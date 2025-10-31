// programs/damm-v2-fee-routing/src/constants.rs

use anchor_lang::prelude::*;

/// Meteora cp-amm program ID (mainnet & devnet)
pub const METEORA_CP_AMM_PROGRAM_ID: Pubkey = 
    pubkey!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

/// PDA seeds
pub const VAULT_SEED: &[u8] = b"vault";
pub const POSITION_AUTHORITY_SEED: &[u8] = b"position_authority";

/// Fee configuration constants
pub const MAX_INVESTOR_FEE_SHARE_BPS: u16 = 10000; // 100%
pub const MIN_PAYOUT_LAMPORTS_DEFAULT: u64 = 1000; // Dust threshold

/// Collect fee mode values (from Meteora)
pub const COLLECT_FEE_MODE_QUOTE_ONLY: u8 = 1;
pub const COLLECT_FEE_MODE_BOTH_TOKENS: u8 = 0;
