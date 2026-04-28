use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{Instruction, AccountMeta};
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_2022::Token2022;

use crate::{
    constants::*,
    error::FeeRoutingError,
    events::PositionClosed,
    state::VaultConfig,
};

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub vault: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault.key().as_ref()],
        bump = vault_config.bump,
        close = payer
    )]
    pub vault_config: Account<'info, VaultConfig>,

    /// CHECK: Derived from vault, signs for CPI to cp-amm
    #[account(
        seeds = [b"vault", vault.key().as_ref(), b"position_authority"],
        bump = vault_config.position_authority_bump
    )]
    pub position_authority: UncheckedAccount<'info>,

    /// CHECK: Meteora position_nft_mint, validated by cp-amm
    #[account(mut, address = vault_config.position_nft_mint)]
    pub position_nft_mint: UncheckedAccount<'info>,

    /// CHECK: Position NFT ATA, validated by cp-amm
    #[account(mut)]
    pub position_nft_account: UncheckedAccount<'info>,

    /// CHECK: Pool address, validated by cp-amm
    #[account(mut, address = vault_config.pool)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: Honorary position, validated by cp-amm
    #[account(mut, address = vault_config.honorary_position)]
    pub position: UncheckedAccount<'info>,

    /// CHECK: Meteora pool authority (fixed)
    #[account(address = METEORA_POOL_AUTHORITY)]
    pub pool_authority: UncheckedAccount<'info>,

    /// CHECK: Rent receiver, receives rent SOL
    #[account(mut)]
    pub rent_receiver: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,

    /// CHECK: cp-amm program
    #[account(address = METEORA_CP_AMM_PROGRAM_ID)]
    pub cp_amm_program: UncheckedAccount<'info>,

    /// CHECK: cp-amm event authority PDA
    pub event_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn close_handler(ctx: Context<ClosePosition>) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();

    // Validate event_authority
    let (expected_event_authority, _) = Pubkey::find_program_address(
        &[b"__event_authority"],
        &ctx.accounts.cp_amm_program.key(),
    );
    require!(
        ctx.accounts.event_authority.key() == expected_event_authority,
        FeeRoutingError::InvalidProgramId
    );

    // Derive position authority seeds for signing
    let position_authority_bump = ctx.accounts.vault_config.position_authority_bump;
    let position_authority_seeds = &[
        b"vault",
        vault_key.as_ref(),
        b"position_authority",
        &[position_authority_bump],
    ];

    // close_position discriminator from cp_amm.json
    let instruction_data = vec![
        123, 134, 81, 0, 49, 68, 98, 98
    ];

    let account_metas = vec![
        AccountMeta::new(ctx.accounts.position_nft_mint.key(), false),
        AccountMeta::new(ctx.accounts.position_nft_account.key(), false),
        AccountMeta::new(ctx.accounts.pool.key(), false),
        AccountMeta::new(ctx.accounts.position.key(), false),
        AccountMeta::new_readonly(ctx.accounts.pool_authority.key(), false),
        AccountMeta::new(ctx.accounts.rent_receiver.key(), false),
        AccountMeta::new_readonly(ctx.accounts.position_authority.key(), true),
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        AccountMeta::new_readonly(expected_event_authority, false),
        AccountMeta::new_readonly(ctx.accounts.cp_amm_program.key(), false),
    ];

    let instruction = Instruction {
        program_id: ctx.accounts.cp_amm_program.key(),
        accounts: account_metas,
        data: instruction_data,
    };

    invoke_signed(
        &instruction,
        &[
            ctx.accounts.position_nft_mint.to_account_info(),
            ctx.accounts.position_nft_account.to_account_info(),
            ctx.accounts.pool.to_account_info(),
            ctx.accounts.position.to_account_info(),
            ctx.accounts.pool_authority.to_account_info(),
            ctx.accounts.rent_receiver.to_account_info(),
            ctx.accounts.position_authority.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.event_authority.to_account_info(),
            ctx.accounts.cp_amm_program.to_account_info(),
        ],
        &[position_authority_seeds],
    )?;

    let clock = Clock::get()?;

    emit!(PositionClosed {
        vault: ctx.accounts.vault.key(),
        pool: ctx.accounts.pool.key(),
        position: ctx.accounts.position.key(),
        timestamp: clock.unix_timestamp,
    });

    // vault_config is closed automatically by Anchor (close = payer)

    Ok(())
}
