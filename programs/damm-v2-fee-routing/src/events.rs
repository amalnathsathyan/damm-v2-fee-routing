// programs/damm-v2-fee-routing/src/events.rs

use anchor_lang::prelude::*;

/// Emitted when an honorary position is successfully initialized
#[event]
pub struct HonoraryPositionInitialized {
    pub vault: Pubkey,
    pub pool: Pubkey,
    pub quote_mint: Pubkey,
    pub base_mint: Pubkey,
    pub position: Pubkey,
    pub position_nft_mint: Pubkey,
    pub position_authority: Pubkey,
    pub y0_total_allocation: u64,
    pub investor_fee_share_bps: u16,
    pub timestamp: i64,
}

/// Emitted when fees are successfully claimed from the honorary position
#[event]
pub struct FeesClaimed {
    pub vault: Pubkey,
    pub pool: Pubkey,
    pub position: Pubkey,
    pub max_amount_a: u64,
    pub max_amount_b: u64,
    pub timestamp: i64,
}

/// Emitted when claimed fees are routed between investor and creator
#[event]
pub struct FeesRouted {
    pub vault: Pubkey,
    pub total_amount: u64,
    pub investor_share: u64,
    pub creator_share: u64,
    pub timestamp: i64,
}

/// Emitted when a position is closed and vault config is reclaimed
#[event]
pub struct PositionClosed {
    pub vault: Pubkey,
    pub pool: Pubkey,
    pub position: Pubkey,
    pub timestamp: i64,
}
