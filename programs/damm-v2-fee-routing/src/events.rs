// programs/damm-v2-fee-routing/src/events.rs

use anchor_lang::prelude::*;

/// Emitted when an honorary position is successfully initialized
#[event]
pub struct HonoraryPositionInitialized {
    /// The vault this position belongs to
    pub vault: Pubkey,
    
    /// The DAMM v2 pool address
    pub pool: Pubkey,
    
    /// Quote token mint (fee collection token)
    pub quote_mint: Pubkey,
    
    /// Base token mint
    pub base_mint: Pubkey,
    
    /// The position account address
    pub position: Pubkey,
    
    /// Position NFT mint
    pub position_nft_mint: Pubkey,
    
    /// Position authority (PDA)
    pub position_authority: Pubkey,
    
    /// Total investor allocation at TGE
    pub y0_total_allocation: u64,
    
    /// Investor fee share in basis points
    pub investor_fee_share_bps: u16,
    
    /// Unix timestamp of initialization
    pub timestamp: i64,
}
