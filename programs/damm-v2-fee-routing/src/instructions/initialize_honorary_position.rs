use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{Instruction, AccountMeta};
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_2022::Token2022;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    constants::*,
    error::FeeRoutingError,
    events::HonoraryPositionInitialized,
    state::VaultConfig,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeHonoraryPositionParams {
    pub y0_total_allocation: u64,
    pub investor_fee_share_bps: u16,
}

#[derive(Accounts)]
#[instruction(params: InitializeHonoraryPositionParams)]
pub struct InitializeHonoraryPosition<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Used only as a seed for PDA derivation
    pub vault: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = VaultConfig::LEN,
        seeds = [b"vault", vault.key().as_ref()],
        bump
    )]
    pub vault_config: Account<'info, VaultConfig>,

    /// CHECK: Validated by reading pool data
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: Base token mint from the pool
    pub base_mint: UncheckedAccount<'info>,

    /// CHECK: Quote token mint from the pool
    pub quote_mint: UncheckedAccount<'info>,

    /// CHECK: PDA derived from vault, signs for CPI to cp-amm
    #[account(
        seeds = [b"vault", vault.key().as_ref(), b"position_authority"],
        bump
    )]
    pub position_authority: UncheckedAccount<'info>,

    /// CHECK: Will be initialized by cp-amm program
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// CHECK: Will be initialized by cp-amm program as NFT mint
    #[account(mut)]
    pub position_nft_mint: Signer<'info>,

    /// CHECK: Will be initialized by cp-amm program as NFT holder
    #[account(mut)]
    pub position_nft_account: UncheckedAccount<'info>,

    /// CHECK: Creator's quote token ATA
    pub creator_quote_ata: UncheckedAccount<'info>,

    /// CHECK: Validated against the known pool authority
    #[account(
        address = METEORA_POOL_AUTHORITY
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// CHECK: Validated against known cp-amm program ID
    #[account(
        address = METEORA_CP_AMM_PROGRAM_ID
    )]
    pub cp_amm_program: UncheckedAccount<'info>,

    /// CHECK: cp-amm event authority PDA, validated in handler
    pub event_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn init_handler(
    ctx: Context<InitializeHonoraryPosition>,
    params: InitializeHonoraryPositionParams,
) -> Result<()> {
    require!(
        params.investor_fee_share_bps <= MAX_INVESTOR_FEE_SHARE_BPS,
        FeeRoutingError::InvalidInvestorFeeShare
    );

    // Read and validate pool state
    let pool_data = ctx.accounts.pool.try_borrow_data()?;

    // Pool struct is 1104 bytes (zero_copy, repr(C)) + 8-byte Anchor discriminator
    // collect_fee_mode is at byte 484, so need at least 485 bytes
    require!(pool_data.len() >= 485, FeeRoutingError::InvalidPoolAccount);

    let token_a_mint = Pubkey::try_from(&pool_data[168..200])
        .map_err(|_| FeeRoutingError::InvalidPoolAccount)?;

    let token_b_mint = Pubkey::try_from(&pool_data[200..232])
        .map_err(|_| FeeRoutingError::InvalidPoolAccount)?;

    // collect_fee_mode at byte 484 in Anchor-serialized Pool account
    // Meteora enum: 0 = BothToken, 1 = OnlyB
    // Ref: programs/cp-amm/src/state/pool.rs in damm-v2 repo
    let collect_fee_mode = pool_data[484];

    require!(
        collect_fee_mode == COLLECT_FEE_MODE_BOTH_TOKEN || collect_fee_mode == COLLECT_FEE_MODE_ONLY_B,
        FeeRoutingError::PoolNotQuoteOnlyCompatible
    );

    // Validate provided mints match pool
    require!(
        token_a_mint == ctx.accounts.base_mint.key(),
        FeeRoutingError::BaseMintMismatch
    );
    require!(
        token_b_mint == ctx.accounts.quote_mint.key(),
        FeeRoutingError::QuoteMintMismatch
    );

    // Validate event_authority is the correct PDA for cp-amm
    let (expected_event_authority, _) = Pubkey::find_program_address(
        &[b"__event_authority"],
        &ctx.accounts.cp_amm_program.key(),
    );
    require!(
        ctx.accounts.event_authority.key() == expected_event_authority,
        FeeRoutingError::InvalidProgramId
    );

    drop(pool_data);

    // Derive position authority PDA seeds
    let vault_key = ctx.accounts.vault.key();
    let position_authority_bump = ctx.bumps.position_authority;
    let position_authority_seeds = &[
        b"vault",
        vault_key.as_ref(),
        b"position_authority",
        &[position_authority_bump],
    ];

    // create_position discriminator from cp_amm.json IDL
    let instruction_data = vec![
        48, 215, 197, 153, 96, 203, 180, 133
    ];

    // CPI account order MUST match cp-amm IDL exactly:
    // owner, position_nft_mint, position_nft_account, pool, position,
    // pool_authority, payer, token_program, system_program, event_authority, program
    let account_metas = vec![
        AccountMeta::new_readonly(ctx.accounts.position_authority.key(), false),
        AccountMeta::new(ctx.accounts.position_nft_mint.key(), true),
        AccountMeta::new(ctx.accounts.position_nft_account.key(), false),
        AccountMeta::new(ctx.accounts.pool.key(), false),
        AccountMeta::new(ctx.accounts.position.key(), false),
        AccountMeta::new_readonly(ctx.accounts.pool_authority.key(), false),
        AccountMeta::new(ctx.accounts.payer.key(), true),
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
        AccountMeta::new_readonly(expected_event_authority, false),
        AccountMeta::new_readonly(ctx.accounts.cp_amm_program.key(), false),
    ];

    let instruction = Instruction {
        program_id: ctx.accounts.cp_amm_program.key(),
        accounts: account_metas,
        data: instruction_data,
    };

    // Execute CPI with position authority as signer
    invoke_signed(
        &instruction,
        &[
            ctx.accounts.position_authority.to_account_info(),
            ctx.accounts.position_nft_mint.to_account_info(),
            ctx.accounts.position_nft_account.to_account_info(),
            ctx.accounts.pool.to_account_info(),
            ctx.accounts.position.to_account_info(),
            ctx.accounts.pool_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.event_authority.to_account_info(),
            ctx.accounts.cp_amm_program.to_account_info(),
        ],
        &[position_authority_seeds],
    )?;

    let clock = Clock::get()?;

    // Initialize all VaultConfig fields
    let vault_config = &mut ctx.accounts.vault_config;
    vault_config.vault = ctx.accounts.vault.key();
    vault_config.pool = ctx.accounts.pool.key();
    vault_config.quote_mint = ctx.accounts.quote_mint.key();
    vault_config.base_mint = ctx.accounts.base_mint.key();
    vault_config.honorary_position = ctx.accounts.position.key();
    vault_config.position_nft_mint = ctx.accounts.position_nft_mint.key();
    vault_config.position_authority_bump = position_authority_bump;
    vault_config.y0_total_allocation = params.y0_total_allocation;
    vault_config.investor_fee_share_bps = params.investor_fee_share_bps;
    vault_config.creator_quote_ata = ctx.accounts.creator_quote_ata.key();
    vault_config.bump = ctx.bumps.vault_config;
    vault_config.initialized_at = clock.unix_timestamp;

    emit!(HonoraryPositionInitialized {
        vault: ctx.accounts.vault.key(),
        pool: ctx.accounts.pool.key(),
        quote_mint: ctx.accounts.quote_mint.key(),
        base_mint: ctx.accounts.base_mint.key(),
        position: ctx.accounts.position.key(),
        position_nft_mint: ctx.accounts.position_nft_mint.key(),
        position_authority: ctx.accounts.position_authority.key(),
        y0_total_allocation: params.y0_total_allocation,
        investor_fee_share_bps: params.investor_fee_share_bps,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
