use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{Instruction, AccountMeta};
use anchor_lang::solana_program::program::invoke_signed;

use crate::{
    constants::*,
    error::FeeRoutingError,
    events::FeesClaimed,
    state::VaultConfig,
};

#[derive(Accounts)]
pub struct ClaimFee<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub vault: UncheckedAccount<'info>,

    #[account(
        seeds = [b"vault", vault.key().as_ref()],
        bump = vault_config.bump
    )]
    pub vault_config: Account<'info, VaultConfig>,

    /// CHECK: Derived from vault, signs for CPI to cp-amm
    #[account(
        seeds = [b"vault", vault.key().as_ref(), b"position_authority"],
        bump = vault_config.position_authority_bump
    )]
    pub position_authority: UncheckedAccount<'info>,

    /// CHECK: Meteora pool authority (fixed)
    #[account(address = METEORA_POOL_AUTHORITY)]
    pub pool_authority: UncheckedAccount<'info>,

    /// CHECK: Validated by vault_config.pool
    #[account(mut, address = vault_config.pool)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: Validated by vault_config.honorary_position
    #[account(mut, address = vault_config.honorary_position)]
    pub position: UncheckedAccount<'info>,

    /// CHECK: Treasury token A account, validated by cp-amm
    #[account(mut)]
    pub token_a_treasury: UncheckedAccount<'info>,

    /// CHECK: Treasury token B account, validated by cp-amm
    #[account(mut)]
    pub token_b_treasury: UncheckedAccount<'info>,

    /// CHECK: Pool token A vault, validated by cp-amm
    #[account(mut)]
    pub token_a_vault: UncheckedAccount<'info>,

    /// CHECK: Pool token B vault, validated by cp-amm
    #[account(mut)]
    pub token_b_vault: UncheckedAccount<'info>,

    /// CHECK: Validated by vault_config
    #[account(address = vault_config.base_mint)]
    pub base_mint: UncheckedAccount<'info>,

    /// CHECK: Validated by vault_config
    #[account(address = vault_config.quote_mint)]
    pub quote_mint: UncheckedAccount<'info>,

    /// CHECK: Position NFT ATA for position_authority
    pub position_nft_account: UncheckedAccount<'info>,

    /// CHECK: Token program for token A
    pub token_a_program: UncheckedAccount<'info>,

    /// CHECK: Token program for token B
    pub token_b_program: UncheckedAccount<'info>,

    /// CHECK: cp-amm program
    #[account(address = METEORA_CP_AMM_PROGRAM_ID)]
    pub cp_amm_program: UncheckedAccount<'info>,

    /// CHECK: cp-amm event authority PDA
    pub event_authority: UncheckedAccount<'info>,
}

pub fn claim_handler(ctx: Context<ClaimFee>) -> Result<()> {
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

    // Read pool data to get token program types
    let pool_data = ctx.accounts.pool.try_borrow_data()?;
    require!(pool_data.len() >= 485, FeeRoutingError::InvalidPoolAccount);

    // Derive position authority seeds for signing
    let position_authority_seeds = &[
        b"vault",
        vault_key.as_ref(),
        b"position_authority",
        &[ctx.accounts.vault_config.position_authority_bump],
    ];
    drop(pool_data);

    // claim_position_fee discriminator from cp_amm.json
    let instruction_data = vec![
        180, 38, 154, 17, 133, 33, 162, 211
    ];

    let account_metas = vec![
        AccountMeta::new_readonly(ctx.accounts.pool_authority.key(), false),
        AccountMeta::new_readonly(ctx.accounts.pool.key(), false),
        AccountMeta::new(ctx.accounts.position.key(), false),
        AccountMeta::new(ctx.accounts.token_a_treasury.key(), false),
        AccountMeta::new(ctx.accounts.token_b_treasury.key(), false),
        AccountMeta::new(ctx.accounts.token_a_vault.key(), false),
        AccountMeta::new(ctx.accounts.token_b_vault.key(), false),
        AccountMeta::new_readonly(ctx.accounts.base_mint.key(), false),
        AccountMeta::new_readonly(ctx.accounts.quote_mint.key(), false),
        AccountMeta::new_readonly(ctx.accounts.position_nft_account.key(), false),
        AccountMeta::new_readonly(ctx.accounts.position_authority.key(), true),
        AccountMeta::new_readonly(ctx.accounts.token_a_program.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_b_program.key(), false),
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
            ctx.accounts.pool_authority.to_account_info(),
            ctx.accounts.pool.to_account_info(),
            ctx.accounts.position.to_account_info(),
            ctx.accounts.token_a_treasury.to_account_info(),
            ctx.accounts.token_b_treasury.to_account_info(),
            ctx.accounts.token_a_vault.to_account_info(),
            ctx.accounts.token_b_vault.to_account_info(),
            ctx.accounts.base_mint.to_account_info(),
            ctx.accounts.quote_mint.to_account_info(),
            ctx.accounts.position_nft_account.to_account_info(),
            ctx.accounts.position_authority.to_account_info(),
            ctx.accounts.token_a_program.to_account_info(),
            ctx.accounts.token_b_program.to_account_info(),
            ctx.accounts.event_authority.to_account_info(),
            ctx.accounts.cp_amm_program.to_account_info(),
        ],
        &[position_authority_seeds],
    )?;

    let clock = Clock::get()?;

    emit!(FeesClaimed {
        vault: ctx.accounts.vault.key(),
        pool: ctx.accounts.pool.key(),
        position: ctx.accounts.position.key(),
        max_amount_a: 0,
        max_amount_b: 0,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
