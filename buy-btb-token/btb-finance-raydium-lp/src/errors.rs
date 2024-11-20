use anchor_lang::prelude::*;
use raydium_clmm_sdk::errors::ErrorCode as RaydiumError;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid parameters provided")]
    InvalidParameters,

    #[msg("Tick range is outside of valid bounds")]
    InvalidTickRange,

    #[msg("Liquidity amount must be greater than zero")]
    InvalidLiquidity,

    #[msg("Insufficient token balance for operation")]
    InsufficientBalance,

    #[msg("Slippage tolerance exceeded during operation")]
    SlippageExceeded,

    #[msg("Position with these parameters already exists")]
    PositionExists,

    #[msg("Position not found or has been closed")]
    PositionNotFound,

    #[msg("Pool must be initialized before operation")]
    PoolNotInitialized,

    #[msg("Specified fee tier is not supported")]
    InvalidFeeTier,

    #[msg("Arithmetic overflow occurred during calculation")]
    ArithmeticOverflow,

    #[msg("Invalid token account owner")]
    InvalidOwner,

    #[msg("Reentrancy detected - operation in progress")]
    Reentrancy,

    #[msg("Operation exceeds tick bounds")]
    TickBoundsExceeded,

    #[msg("Invalid price range provided")]
    InvalidPriceRange,

    #[msg("Operation would result in zero liquidity")]
    ZeroLiquidity,

    #[msg("Token accounts must have same mint")]
    TokenMintMismatch,

    #[msg("Insufficient protocol fees collected")]
    InsufficientProtocolFees,

    #[msg("Price limit reached during swap")]
    PriceLimitReached,

    #[msg("Operation would result in invalid tick spacing")]
    InvalidTickSpacing,

    #[msg("Invalid authority for operation")]
    InvalidAuthority,
}

impl From<RaydiumError> for ErrorCode {
    fn from(error: RaydiumError) -> Self {
        match error {
            RaydiumError::InvalidTickRange => ErrorCode::InvalidTickRange,
            RaydiumError::InvalidLiquidity => ErrorCode::InvalidLiquidity,
            _ => ErrorCode::InvalidParameters,
        }
    }
}
