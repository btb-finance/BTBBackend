# BTB Sale Account Client Documentation

### Overview
This script initializes a BTB (Buy-to-Build) sale account on Solana devnet. It creates necessary PDAs and token accounts for BTB token sales with USDT, USDC, and PayPal USD payment options.

### How to Run
```bash
npx tsx update_initialize.ts
```

### Script Output
When you run the script, it will display:
1. BTB Sale Account (PDA) address
2. BTB Sale Token Account address
3. Transaction signature after successful initialization

Example output:
```bash
BTB Sale Account (PDA): ABC123...XYZ
BTB Sale Token Account: DEF456...UVW
PDA initialized. Transaction signature: xxxxx...
```

### Client Code Explanation
1. **Wallet Loading & Connection**
```typescript
const connection = new web3.Connection("https://api.devnet.solana.com");
const initializerKeypair = loadWalletKey("owner_signer_wallet.json");
```
- Establishes connection to Solana devnet
- Loads owner wallet from JSON file

2. **PDA Creation**
```typescript
const [btbSaleAccount] = await web3.PublicKey.findProgramAddress(
    [Buffer.from("btb-sale-account"), initializerKeypair.publicKey.toBuffer()],
    program.programId
);
```
- Creates PDA (Program Derived Address) for BTB sale account
- Uses "btb-sale-account" and owner's public key as seeds

3. **Token Account Setup**
```typescript
const btbMint = new web3.PublicKey("btbjSLvBfKFf94VTYbze6TtCXYaeBgCadTcLfvoZp9d");
const btbSaleTokenAccount = await getAssociatedTokenAddress(btbMint, btbSaleAccount, true);
```
- Sets up BTB token mint address
- Creates associated token account for PDA

4. **Payment Token Addresses**
```typescript
const usdt = new web3.PublicKey("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const usdc = new web3.PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const paypal_usd = new web3.PublicKey("2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo");
```
- USDT address on devnet
- USDC address on devnet
- PayPal USD address on devnet

### Important Program Addresses
```typescript
BTB Mint: "btbjSLvBfKFf94VTYbze6TtCXYaeBgCadTcLfvoZp9d"
USDT: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
USDC: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
PayPal USD: "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo"
Owner Token Receive: "muf1hVTPSP4NfbtqEjbSdZiJwg5KvX81q6wPmafMfz5"
```

### Required Files
1. `initialize.ts`: Main initialization script
2. `owner_signer_wallet.json`: Owner wallet keypair (you need to create this)

### Token Prices
- BTB Price: 0.02 units
- Vesting Price: 0.01 units

### Common Errors and Solutions
1. "Wallet not found"
   - Make sure owner_signer_wallet.json exists in the correct directory
   - Check file permissions

2. "Insufficient balance"
   - Add more SOL to wallet for transaction fees
   - Make sure wallet has enough tokens for initialization

3. "Invalid PDA"
   - Verify program ID is correct
   - Check seed phrase is "btb-sale-account"

4. "Token account not found"
   - Ensure BTB mint address is correct
   - Verify PDA derivation is successful

Note: All transactions are on Solana devnet. Make sure your wallet has sufficient devnet SOL for transactions.