use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized: Only owner can perform this action")]
    Unauthorized,
    
    #[msg("BTB price must be greater than 0")]
    ZeroBTBPrice,
    
    #[msg("Vesting price must be greater than 0")]
    ZeroVestingPrice,
    
    #[msg("Invalid token type selected")]
    InvalidTokenType,
    
    #[msg("Calculation overflow occurred")]
    CalculationError,
    
    #[msg("Invalid token mint address")]
    InvalidTokenMint,

    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    
    #[msg("Amount is too small. Minimum amount is 0.001 tokens")]
    AmountTooSmall,
    
    #[msg("Amount exceeds maximum limit")]
    AmountTooLarge,
}