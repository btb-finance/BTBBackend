# BTB Sale Account Client Documentation

### Overview
This project provides scripts for initializing and interacting with a BTB (Buy-to-Build) sale account on Solana devnet. It manages PDAs and token accounts for BTB token sales with USDT, USDC, and PayPal USD payment options.

### Available Scripts

1. Initialize BTB Sale Account:
```bash
npx tsx buy_token.ts
```

2. Buy BTB Tokens:
```bash
npx tsx buy-token.ts
```

### Script Outputs

#### Initialize Script Output
```bash
BTB Sale Account (PDA): ABC123...XYZ
BTB Sale Token Account: DEF456...UVW
PDA initialized. Transaction signature: xxxxx...
```

#### Buy Token Script Output
```bash
BTB Sale Account: ABC123...XYZ
User USDT Account: DEF456...UVW
Owner USDT Account: GHI789...RST
BTB Sale Token Account: JKL012...MNO
User BTB Account: PQR345...XYZ
Buy Token Transaction Signature: xxxxx...
```

### Important Program Addresses
```typescript
Program ID: "teyk2AYGjb6SGPtXxqr6EW1bqQNtthfjaQggMYDSSXv"
BTB Mint: "btbjSLvBfKFf94VTYbze6TtCXYaeBgCadTcLfvoZp9d"
USDT: "usddpqpxr3LAu2HL95YJ4JJ4LFGFumAv7iaUhHYbmiQ"
Owner Initialize Wallet: "sibxc42SdHMtovWeFzHihDMyENg9Dzf3vLWjxpt1xHo"
Owner Token Receive: "te6eqhHuXFuhP1bjBfPs17VS84dR1M725FR9txASuCS"
```

### Required Files
1. `initialize.ts`: Initialization script
2. `buy-token.ts`: Token purchase script
3. `user_wallet.json`: User wallet keypair for testing

### Setting Up Buy Token Script

1. **Import Required Dependencies**
```typescript
import idl from "../../target/idl/pda_vesting.json";
import { PdaVesting } from "../../target/types/pda_vesting";
import { Program, Idl, AnchorProvider, setProvider, web3, Wallet, BN } from "@coral-xyz/anchor";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token";
```

2. **Initialize Connection and Wallet**
```typescript
const connection = new web3.Connection("https://api.devnet.solana.com");
const userKeypair = loadWalletKey("user_wallet.json");
const userWallet = new Wallet(userKeypair);
```

3. **Configure Token Purchase**
```typescript
const amount = new BN(1000000); // Amount to buy
const tokenType = 1; // 1 for USDT
```

4. **Account Structure**
```typescript
{
    btbSaleAccount: btbSaleAccount,
    userTokenAccount: userUsdtAccount,
    ownerTokenAccount: ownerUsdtAccount,
    btbSaleTokenAccount: btbSaleTokenAccount,
    userBtbAccount: userBtbAccount,
    btbMintAccount: btbMint,
    user: userKeypair.publicKey,
    systemProgram: web3.SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
}
```

### Token Purchase Parameters
- Amount: 1000000 (represents 1000 tokens with decimals)
- Token Type: 1 (USDT)
- Transaction fees: ~0.00001 SOL

### Prerequisites for Buying Tokens
1. Initialize BTB sale account first
2. User wallet must have:
   - Sufficient USDT balance
   - Minimum 0.01 SOL for transaction fees
3. BTB sale account must have sufficient BTB tokens

### Common Errors and Solutions

1. "Program Error: Invalid Token Type"
   - Ensure token_type is 1 for USDT
   - Verify token accounts match the selected token type

2. "Program Error: Invalid Amount"
   - Amount must be greater than 0
   - Check decimal places (amount = tokens * 10^6)

3. "Token Account Not Found"
   - Run initialize script first
   - Check if user's token accounts exist
   - Verify PDA derivation is correct

4. "Insufficient Balance"
   - Add USDT to user wallet
   - Add SOL for transaction fees
   - Verify BTB sale account has enough tokens

5. "Invalid Owner"
   - Check owner_initialize_wallet matches initialized state
   - Verify owner_token_receive_wallet is correct

### Testing Instructions

1. **Prepare Environment**
   ```bash
   # Install dependencies
   npm install
   ```

2. **Create User Wallet**
   - Generate `user_wallet.json` with Solana CLI
   - Fund wallet with devnet SOL
   - Add test USDT tokens

3. **Run Test**
   ```bash
   npx tsx buy-token.ts
   ```

4. **Verify Transaction**
   - Check console output for transaction signature
   - Verify token balances updated correctly
   - Check for any error messages

Note: All transactions are on Solana devnet. Make sure your wallet has sufficient devnet SOL and USDT for testing.

### Development Notes
- Token amounts use 6 decimal places
- USDT amount calculation: tokens * 10^6
- Program requires owner signature for initialization
- Associated Token Accounts are created automatically if needed
- Error messages are detailed in console output

### Security Considerations
- Keep wallet keypair files secure
- Verify all addresses before transactions
- Double-check token amounts and decimals
- Monitor transaction logs for errors

For more information about the Solana program structure and deployment, refer to the smart contract documentation.