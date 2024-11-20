# PDA Vesting Contract

A Solana program for managing token sales and vesting with support for multiple payment tokens (USDT, USDC, and PayPal USD). The contract enables secure token purchases with predefined pricing and owner-controlled parameters.

## Features

- Multi-token payment support (USDT, USDC, PayPal USD)
- Configurable token pricing
- Owner-controlled parameter updates
- Secure PDA-based account management
- Associated Token Account integration
- Minimum purchase amount validation
- Automated price calculations

## Contract Address
```
teyk2AYGjb6SGPtXxqr6EW1bqQNtthfjaQggMYDSSXv
```

## Account Structure

### InitializeDataAccount
Main storage account that holds configuration data:
- BTB token address
- USDT token address
- USDC token address
- PayPal USD token address
- Owner token receive wallet
- Owner initialize wallet
- BTB token price
- Vesting price

## Instructions

### 1. Initialize
Initializes the contract with base configuration:

```rust
initialize(
    btb: Pubkey,
    usdt: Pubkey,
    usdc: Pubkey,
    paypal_usd: Pubkey,
    owner_token_receive_wallet: Pubkey,
    btb_price: u64,
    vesting_price: u64
)
```

### 2. Update Initialize
Updates the contract configuration (owner-only):

```rust
update_initialize(
    btb: Pubkey,
    usdt: Pubkey,
    usdc: Pubkey,
    paypal_usd: Pubkey,
    owner_token_receive_wallet: Pubkey,
    btb_price: u64,
    vesting_price: u64
)
```

### 3. Buy Token
Executes token purchase:

```rust
buy_token(
    amount: u64,
    token_type: u8
)
```

Token types:
- 1: USDT
- 2: USDC
- 3: PayPal USD

## Error Codes

| Error | Description |
|-------|-------------|
| Unauthorized | Only owner can perform this action |
| ZeroBTBPrice | BTB price must be greater than 0 |
| ZeroVestingPrice | Vesting price must be greater than 0 |
| InvalidTokenType | Invalid token type selected |
| CalculationError | Calculation overflow occurred |
| InvalidTokenMint | Invalid token mint address |
| InvalidAmount | Amount must be greater than zero |
| AmountTooSmall | Amount is too small (minimum 0.001 tokens) |
| AmountTooLarge | Amount exceeds maximum limit |

## Usage Example

1. Initialize the contract:
```typescript
await program.methods
  .initialize(
    btbMint,
    usdtMint,
    usdcMint,
    paypalUsdMint,
    ownerTokenReceiveWallet,
    new BN(1000), // BTB price (1 USDT = 1000 units)
    new BN(1000)  // Vesting price
  )
  .accounts({
    btbSaleAccount: pdaAddress,
    btbSaleTokenAccount: btbSaleAta,
    btbMintAccount: btbMint,
    signer: wallet.publicKey,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  })
  .rpc();
```

2. Buy tokens:
```typescript
await program.methods
  .buyToken(
    new BN(1000000), // Amount in smallest units
    1                // Token type (1 for USDT)
  )
  .accounts({
    btbSaleAccount: pdaAddress,
    userTokenAccount: userUsdtAta,
    ownerTokenAccount: ownerUsdtAta,
    btbSaleTokenAccount: btbSaleAta,
    userBtbAccount: userBtbAta,
    btbMintAccount: btbMint,
    user: wallet.publicKey,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  })
  .rpc();
```

## Security Considerations

1. The contract uses PDA for secure account management
2. Owner-only functions are properly validated
3. Minimum amount checks prevent dust attacks
4. Token account validation ensures correct token types
5. Math operations include overflow checks

## Installation

1. Clone the repository
2. Install dependencies:
```bash
yarn install
```

3. Build the program:
```bash
anchor build
```

4. Deploy:
```bash
anchor deploy
```

## Testing

Run the test suite:
```bash
anchor test
```

## License

[Add your license information here]