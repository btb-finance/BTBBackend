use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized: Only program owner/deployer can perform this action")]
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

    #[msg("Claiming is not available yet.")]
    ClaimNotAvailableYet,

    #[msg("There is nothing to claim.")]
    NothingToClaim,

    #[msg("Cannot transfer admin to zero address")]
    InvalidNewAdmin,

    #[msg("Sale is not currently active")]
    SaleNotActive,

    #[msg("No tokens available to withdraw")]
    NoTokensToWithdraw,

    #[msg("Unauthorized: Only program deployer can initialize")]
    UnauthorizedDeployer,
}