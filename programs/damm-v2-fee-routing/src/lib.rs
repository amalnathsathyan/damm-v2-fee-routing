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
    /// This creates an empty DAMM v2 position owned by a program PDA that
    /// accrues fees exclusively in the quote token.
    ///
    /// # Arguments
    /// * `params` - Configuration parameters including Y0 allocation and fee share
    ///
    /// # Validation
    /// - Verifies pool supports quote-only fee collection (collect_fee_mode = 0)
    /// - Validates token order (base < quote lexicographically)
    /// - Checks investor fee share is within bounds (0-10000 bps)
    ///
    /// # Emits
    /// * `HonoraryPositionInitialized` event on success
    pub fn initialize_honorary_position(
        ctx: Context<InitializeHonoraryPosition>,
        params: InitializeHonoraryPositionParams,
    ) -> Result<()> {
        initialize_honorary_position::handler(ctx, params)
    }
}
