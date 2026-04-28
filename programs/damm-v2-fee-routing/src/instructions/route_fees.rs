use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022, Transfer};
use anchor_spl::token_interface::TokenAccount as TokenAccountInterface;

use crate::{
    constants::*,
    error::FeeRoutingError,
    events::FeesRouted,
    state::VaultConfig,
};

#[derive(Accounts)]
pub struct RouteFees<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub vault: UncheckedAccount<'info>,

    #[account(
        seeds = [b"vault", vault.key().as_ref()],
        bump = vault_config.bump
    )]
    pub vault_config: Account<'info, VaultConfig>,

    /// CHECK: Position authority PDA (signer for token transfers)
    #[account(
        seeds = [b"vault", vault.key().as_ref(), b"position_authority"],
        bump = vault_config.position_authority_bump
    )]
    pub position_authority: UncheckedAccount<'info>,

    /// Quote token account owned by position_authority (source of claimed fees)
    #[account(
        mut,
        constraint = quote_token_account.owner == position_authority.key() @ FeeRoutingError::InvalidCreatorAta
    )]
    pub quote_token_account: InterfaceAccount<'info, TokenAccountInterface>,

    /// Creator's quote ATA (receives remainder share)
    #[account(
        mut,
        address = vault_config.creator_quote_ata
    )]
    pub creator_quote_ata: InterfaceAccount<'info, TokenAccountInterface>,

    /// Vault's quote ATA (receives investor share)
    #[account(mut)]
    pub vault_quote_ata: InterfaceAccount<'info, TokenAccountInterface>,

    pub token_program: Program<'info, Token2022>,
}

pub fn route_handler(ctx: Context<RouteFees>) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    let balance = ctx.accounts.quote_token_account.amount;

    require!(balance > 0, FeeRoutingError::AmountIsZero);

    // Compute split based on investor_fee_share_bps
    let investor_share = balance
        .checked_mul(ctx.accounts.vault_config.investor_fee_share_bps as u64)
        .ok_or(FeeRoutingError::MathOverflow)?
        .checked_div(MAX_INVESTOR_FEE_SHARE_BPS as u64)
        .ok_or(FeeRoutingError::MathOverflow)?;

    let creator_share = balance
        .checked_sub(investor_share)
        .ok_or(FeeRoutingError::MathOverflow)?;

    let position_authority_bump = ctx.accounts.vault_config.position_authority_bump;
    let signer_seeds: &[&[u8]] = &[
        b"vault",
        vault_key.as_ref(),
        b"position_authority",
        &[position_authority_bump],
    ];

    // Transfer investor share to vault
    if investor_share > 0 {
        token_2022::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.quote_token_account.to_account_info(),
                    to: ctx.accounts.vault_quote_ata.to_account_info(),
                    authority: ctx.accounts.position_authority.to_account_info(),
                },
                &[signer_seeds],
            ),
            investor_share,
        )?;
    }

    // Transfer creator share to creator ATA
    if creator_share > 0 {
        token_2022::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.quote_token_account.to_account_info(),
                    to: ctx.accounts.creator_quote_ata.to_account_info(),
                    authority: ctx.accounts.position_authority.to_account_info(),
                },
                &[signer_seeds],
            ),
            creator_share,
        )?;
    }

    let clock = Clock::get()?;

    emit!(FeesRouted {
        vault: ctx.accounts.vault.key(),
        total_amount: balance,
        investor_share,
        creator_share,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
