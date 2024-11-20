use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use solana_program_test::*;
use solana_sdk::signature::Keypair;

mod utils;
use utils::*;

#[tokio::test]
async fn test_initialize_pool() {
    let mut context = setup_test_context().await;
    let authority = Keypair::new();

    // Create token mints
    let token_0_mint = create_mint(&mut context, &authority, 6).await;
    let token_1_mint = create_mint(&mut context, &authority, 6).await;

    // Test parameters
    let tick_spacing = 10;
    let initial_price = 100_000_000; // 1.0 in fixed point

    // Initialize pool
    let pool = Keypair::new();
    let (pool_token_0, _) = Pubkey::find_program_address(
        &[
            b"token_0",
            pool.pubkey().as_ref(),
        ],
        &crate::ID,
    );
    let (pool_token_1, _) = Pubkey::find_program_address(
        &[
            b"token_1",
            pool.pubkey().as_ref(),
        ],
        &crate::ID,
    );

    let accounts = crate::accounts::InitializePool {
        pool: pool.pubkey(),
        authority: authority.pubkey(),
        token_mint_0: token_0_mint,
        token_mint_1: token_1_mint,
        pool_token_0,
        pool_token_1,
        token_program: Token::id(),
        system_program: System::id(),
        rent: sysvar::rent::id(),
    };

    let result = crate::initialize_pool(
        Context::new(
            &crate::ID,
            accounts,
            &[&pool, &authority],
            context.recent_blockhash,
        ),
        tick_spacing,
        initial_price,
    );

    assert!(result.is_ok());

    // Verify pool state
    let pool_account = context.program.account::<crate::Pool>(pool.pubkey()).await.unwrap();
    assert_eq!(pool_account.token_mint_0, token_0_mint);
    assert_eq!(pool_account.token_mint_1, token_1_mint);
    assert_eq!(pool_account.tick_spacing, tick_spacing);
    assert_eq!(pool_account.sqrt_price, initial_price);
}

#[tokio::test]
async fn test_create_position() {
    let mut context = setup_test_context().await;
    let authority = Keypair::new();

    // Setup pool first
    let (pool, token_0_mint, token_1_mint) = setup_test_pool(&mut context, &authority).await;

    // Create user token accounts
    let user_token_0 = create_token_account(&mut context, &token_0_mint, &authority.pubkey()).await;
    let user_token_1 = create_token_account(&mut context, &token_1_mint, &authority.pubkey()).await;

    // Mint some tokens to user
    mint_tokens(&mut context, &token_0_mint, &user_token_0, &authority, 1_000_000).await;
    mint_tokens(&mut context, &token_1_mint, &user_token_1, &authority, 1_000_000).await;

    // Test parameters
    let tick_lower = -100;
    let tick_upper = 100;
    let liquidity = 1_000_000;
    let amount_0_max = 1_000_000;
    let amount_1_max = 1_000_000;

    // Create position
    let position = Keypair::new();
    let accounts = crate::accounts::CreatePosition {
        pool: pool.pubkey(),
        position: position.pubkey(),
        authority: authority.pubkey(),
        user_token_0,
        user_token_1,
        token_program: Token::id(),
        system_program: System::id(),
        rent: sysvar::rent::id(),
    };

    let result = crate::create_position(
        Context::new(
            &crate::ID,
            accounts,
            &[&position, &authority],
            context.recent_blockhash,
        ),
        tick_lower,
        tick_upper,
        liquidity,
        amount_0_max,
        amount_1_max,
    );

    assert!(result.is_ok());

    // Verify position state
    let position_account = context.program.account::<crate::Position>(position.pubkey()).await.unwrap();
    assert_eq!(position_account.pool, pool.pubkey());
    assert_eq!(position_account.tick_lower_index, tick_lower);
    assert_eq!(position_account.tick_upper_index, tick_upper);
    assert_eq!(position_account.liquidity, liquidity);
}

#[tokio::test]
async fn test_increase_liquidity() {
    let mut context = setup_test_context().await;
    let authority = Keypair::new();

    // Setup pool and position first
    let (pool, position, token_0_mint, token_1_mint) = setup_test_position(&mut context, &authority).await;

    // Create user token accounts
    let user_token_0 = create_token_account(&mut context, &token_0_mint, &authority.pubkey()).await;
    let user_token_1 = create_token_account(&mut context, &token_1_mint, &authority.pubkey()).await;

    // Mint additional tokens to user
    mint_tokens(&mut context, &token_0_mint, &user_token_0, &authority, 500_000).await;
    mint_tokens(&mut context, &token_1_mint, &user_token_1, &authority, 500_000).await;

    // Test parameters
    let liquidity_increase = 500_000;
    let amount_0_max = 500_000;
    let amount_1_max = 500_000;

    let accounts = crate::accounts::ModifyPosition {
        pool: pool.pubkey(),
        position: position.pubkey(),
        authority: authority.pubkey(),
        user_token_0,
        user_token_1,
        token_program: Token::id(),
    };

    let result = crate::increase_liquidity(
        Context::new(
            &crate::ID,
            accounts,
            &[&authority],
            context.recent_blockhash,
        ),
        liquidity_increase,
        amount_0_max,
        amount_1_max,
    );

    assert!(result.is_ok());

    // Verify updated position state
    let position_account = context.program.account::<crate::Position>(position.pubkey()).await.unwrap();
    assert_eq!(position_account.liquidity, 1_500_000); // Original 1M + 500k increase
}

#[tokio::test]
async fn test_error_conditions() {
    let mut context = setup_test_context().await;
    let authority = Keypair::new();

    // Test invalid tick range
    let result = test_create_position_with_params(
        &mut context,
        &authority,
        100, // tick_lower > tick_upper
        0,
        1_000_000,
        1_000_000,
        1_000_000,
    ).await;
    assert!(matches!(result.unwrap_err(), ErrorCode::InvalidTickRange));

    // Test zero liquidity
    let result = test_create_position_with_params(
        &mut context,
        &authority,
        -100,
        100,
        0, // zero liquidity
        1_000_000,
        1_000_000,
    ).await;
    assert!(matches!(result.unwrap_err(), ErrorCode::InvalidLiquidity));

    // Test insufficient balance
    let result = test_create_position_with_params(
        &mut context,
        &authority,
        -100,
        100,
        1_000_000,
        10_000_000_000, // amount larger than minted
        10_000_000_000,
    ).await;
    assert!(matches!(result.unwrap_err(), ErrorCode::InsufficientBalance));
}

// Helper function to setup test pool
async fn setup_test_pool(
    context: &mut TestContext,
    authority: &Keypair,
) -> (Keypair, Pubkey, Pubkey) {
    let token_0_mint = create_mint(context, authority, 6).await;
    let token_1_mint = create_mint(context, authority, 6).await;
    let pool = Keypair::new();

    // Initialize pool here...
    
    (pool, token_0_mint, token_1_mint)
}

// Helper function to setup test position
async fn setup_test_position(
    context: &mut TestContext,
    authority: &Keypair,
) -> (Keypair, Keypair, Pubkey, Pubkey) {
    let (pool, token_0_mint, token_1_mint) = setup_test_pool(context, authority).await;
    let position = Keypair::new();

    // Create position here...
    
    (pool, position, token_0_mint, token_1_mint)
}

// Helper function to test position creation with different parameters
async fn test_create_position_with_params(
    context: &mut TestContext,
    authority: &Keypair,
    tick_lower: i32,
    tick_upper: i32,
    liquidity: u128,
    amount_0_max: u64,
    amount_1_max: u64,
) -> Result<()> {
    // Implementation here...
    Ok(())
}
