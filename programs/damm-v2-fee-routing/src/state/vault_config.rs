// programs/damm-v2-fee-routing/src/state/vault_config.rs

use anchor_lang::prelude::*;

#[account]
pub struct VaultConfig {
    /// The vault this configuration belongs to
    pub vault: Pubkey,
    
    /// DAMM v2 pool address
    pub pool: Pubkey,
    
    /// Quote token mint (fees collected in this token)
    pub quote_mint: Pubkey,
    
    /// Base token mint
    pub base_mint: Pubkey,
    
    /// Honorary position account address
    pub honorary_position: Pubkey,
    
    /// Position NFT mint
    pub position_nft_mint: Pubkey,
    
    /// Position authority PDA bump
    pub position_authority_bump: u8,
    
    /// Total investor allocation minted at TGE (Y0)
    pub y0_total_allocation: u64,
    
    /// Investor fee share in basis points (0-10000)
    pub investor_fee_share_bps: u16,
    
    /// Creator's quote token ATA (receives remainder fees)
    pub creator_quote_ata: Pubkey,
    
    /// VaultConfig PDA bump
    pub bump: u8,
    
    /// Initialization timestamp
    pub initialized_at: i64,
}

impl VaultConfig {
    /// Space required for VaultConfig account
    pub const LEN: usize = 8 +  // discriminator
        32 +  // vault
        32 +  // pool
        32 +  // quote_mint
        32 +  // base_mint
        32 +  // honorary_position
        32 +  // position_nft_mint
        1 +   // position_authority_bump
        8 +   // y0_total_allocation
        2 +   // investor_fee_share_bps
        32 +  // creator_quote_ata
        1 +   // bump
        8;    // initialized_at
}
