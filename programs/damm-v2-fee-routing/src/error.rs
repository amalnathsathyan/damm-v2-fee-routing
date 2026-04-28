// programs/damm-v2-fee-routing/src/error.rs

use anchor_lang::prelude::*;

#[error_code]
pub enum FeeRoutingError {
    #[msg("Pool fee collection mode is not quote-only (must be 0 or 1)")]
    PoolNotQuoteOnlyCompatible,

    #[msg("Invalid token order - base mint must be lexicographically less than quote mint")]
    InvalidTokenOrder,

    #[msg("Base mint does not match pool configuration")]
    BaseMintMismatch,

    #[msg("Quote mint does not match pool configuration")]
    QuoteMintMismatch,

    #[msg("Pool account is invalid or not properly initialized")]
    InvalidPoolAccount,

    #[msg("Pool discriminator does not match expected DAMM v2 pool")]
    InvalidPoolDiscriminator,

    #[msg("Investor fee share exceeds maximum allowed (10000 bps)")]
    InvalidInvestorFeeShare,

    #[msg("Total allocation (Y0) cannot be zero")]
    InvalidTotalAllocation,

    #[msg("Creator quote ATA is invalid or not for the correct mint")]
    InvalidCreatorAta,

    #[msg("Fee mode not quote only")]
    FeeModeNotQuoteOnly,

    #[msg("Position authority mismatch")]
    InvalidPositionAuthority,

    #[msg("Vault configuration already exists")]
    VaultConfigAlreadyExists,

    #[msg("Invalid program ID - must be Meteora cp-amm")]
    InvalidProgramId,

    #[msg("Amount is zero")]
    AmountIsZero,

    #[msg("Math operation overflow")]
    MathOverflow,
}
