// programs/damm-v2-fee-routing/src/lib.rs

use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use error::*;
pub use events::*;
pub use instructions::*;
pub use state::*;

declare_id!("HuuZH7f52k6mrjdqyV3vaK6xoY2FWwirdbqBeJu8qfFc");

#[program]
pub mod damm_v2_fee_routing {
    use super::*;

    /// Initialize an honorary fee position for quote-only fee collection
    ///
    /// Creates an empty DAMM v2 position owned by a program PDA that
    /// accrues fees exclusively in the quote token.
    ///
    /// # Arguments
    /// * `params` - Configuration parameters including Y0 allocation and fee share
    pub fn initialize_honorary_position(
        ctx: Context<InitializeHonoraryPosition>,
        params: InitializeHonoraryPositionParams,
    ) -> Result<()> {
        initialize_honorary_position::init_handler(ctx, params)
    }

    /// Claim accrued fees from the honorary position
    ///
    /// CPIs to Meteora's claim_position_fee to collect pending fees
    /// from the honorary position into treasury token accounts.
    pub fn claim_fee(ctx: Context<ClaimFee>) -> Result<()> {
        claim_fee::claim_handler(ctx)
    }

    /// Route claimed fees between investor and creator
    ///
    /// Splits claimed fees according to investor_fee_share_bps stored
    /// in vault_config. Transfers tokens from position_authority's
    /// token account to vault and creator ATAs.
    pub fn route_fees(ctx: Context<RouteFees>) -> Result<()> {
        route_fees::route_handler(ctx)
    }

    /// Close the honorary position and reclaim rent
    ///
    /// CPIs to Meteora's close_position to close the position,
    /// then closes the vault_config PDA and returns rent to payer.
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        close_position::close_handler(ctx)
    }
}
