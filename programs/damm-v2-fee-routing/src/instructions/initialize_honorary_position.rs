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

    /// CHECK: PDA derived from vault, will be the position owner
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

    /// CHECK: Validated against the pool
    pub pool_authority: UncheckedAccount<'info>,

    /// CHECK: cp-amm program
    pub cp_amm_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<InitializeHonoraryPosition>,
    params: InitializeHonoraryPositionParams,
) -> Result<()> {
    // ✅ FIX 1: Use correct constant name
    require!(
        params.investor_fee_share_bps <= MAX_INVESTOR_FEE_SHARE_BPS,
        FeeRoutingError::InvalidInvestorFeeShare
    );

    // Read and validate pool state
    let pool_data = ctx.accounts.pool.try_borrow_data()?;

    // ✅ FIX 2: Need >= 233 to read byte at offset 232, use correct error variant
    require!(pool_data.len() >= 233, FeeRoutingError::InvalidPoolAccount);

    // ✅ FIX 3: Use correct error variant InvalidPoolAccount
    let token_a_mint = Pubkey::try_from(&pool_data[168..200])
        .map_err(|_| FeeRoutingError::InvalidPoolAccount)?;

    // ✅ FIX 4: Use correct error variant InvalidPoolAccount
    let token_b_mint = Pubkey::try_from(&pool_data[200..232])
        .map_err(|_| FeeRoutingError::InvalidPoolAccount)?;

    // Read collect_fee_mode (u8 at offset 232)
    let collect_fee_mode = pool_data[72];


    //collect_fee_mode: 1 = OnlyB for swap (A->B),
    //collect_fee_mode: 0 = OnlyB for swap (B->A), where B is Quote Mint.
    //ref:https://github.com/MeteoraAg/damm-v2/blob/b6453348b87105d143bd1d98a1dbdd0a3774bd50/programs/cp-amm/src/state/pool.rs#L41
    //ref:https://github.com/MeteoraAg/damm-v2/blob/main/programs/cp-amm/src/state/fee.rs#L351
    require!(
        collect_fee_mode == 0 || collect_fee_mode == 1,
        FeeRoutingError::PoolNotQuoteOnlyCompatible
    );

    // ✅ FIX 6: Validate provided mints match pool
    require!(
        token_a_mint == ctx.accounts.base_mint.key(),
        FeeRoutingError::BaseMintMismatch
    );
    require!(
        token_b_mint == ctx.accounts.quote_mint.key(),
        FeeRoutingError::QuoteMintMismatch
    );

    drop(pool_data);

    // Derive position authority PDA seeds
    let vault_key = ctx.accounts.vault.key();
    let position_authority_seeds = &[
        b"position_authority",
        vault_key.as_ref(),
        &[ctx.bumps.vault_config],
    ];

    // ✅ CORRECTED DISCRIMINATOR from cp_amm.json IDL
    let instruction_data = vec![
        48, 215, 197, 153, 96, 203, 180, 133  // create_position discriminator
    ];

    // Build account metas for CPI call (must match exact order from IDL)
    let account_metas = vec![
        AccountMeta::new_readonly(ctx.accounts.position_authority.key(), false),  // owner
        AccountMeta::new(ctx.accounts.position_nft_mint.key(), true),             // position_nft_mint (mut, signer)
        AccountMeta::new(ctx.accounts.position_nft_account.key(), false),         // position_nft_account (mut)
        AccountMeta::new(ctx.accounts.pool.key(), false),                         // pool (mut)
        AccountMeta::new(ctx.accounts.position.key(), false),                     // position (mut) - NOT a signer, cp-amm creates it
        AccountMeta::new_readonly(ctx.accounts.pool_authority.key(), false),      // pool_authority
        AccountMeta::new(ctx.accounts.payer.key(), true),                         // payer (mut, signer)
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),       // token_program
        AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),      // system_program
        AccountMeta::new_readonly(ctx.accounts.cp_amm_program.key(), false),      // event_authority
        AccountMeta::new_readonly(ctx.accounts.cp_amm_program.key(), false),      // program
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
            ctx.accounts.cp_amm_program.to_account_info(),
            ctx.accounts.cp_amm_program.to_account_info(),
        ],
        &[position_authority_seeds],
    )?;

    let clock = Clock::get()?;

    // ✅ FIX 7: Initialize ALL VaultConfig fields
    let vault_config = &mut ctx.accounts.vault_config;
    vault_config.vault = ctx.accounts.vault.key();
    vault_config.pool = ctx.accounts.pool.key();
    vault_config.quote_mint = ctx.accounts.quote_mint.key();
    vault_config.base_mint = ctx.accounts.base_mint.key();
    vault_config.honorary_position = ctx.accounts.position.key();
    vault_config.position_nft_mint = ctx.accounts.position_nft_mint.key();
    vault_config.position_authority_bump = ctx.bumps.vault_config;
    vault_config.y0_total_allocation = params.y0_total_allocation;
    vault_config.investor_fee_share_bps = params.investor_fee_share_bps;
    vault_config.creator_quote_ata = ctx.accounts.creator_quote_ata.key();
    vault_config.bump = ctx.bumps.vault_config;
    vault_config.initialized_at = clock.unix_timestamp;

    // ✅ FIX 8: Emit event with ALL required fields
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
