// scripts/createPool.ts
import { 
    Connection, 
    Keypair, 
    PublicKey, 
    Transaction,
    sendAndConfirmTransaction,
    SystemProgram,
    LAMPORTS_PER_SOL
} from '@solana/web3.js';
import { 
    TOKEN_PROGRAM_ID, 
    createMint,
    getAssociatedTokenAddress,
    createAssociatedTokenAccountInstruction,
    createInitializeMintInstruction,
    MINT_SIZE
} from '@solana/spl-token';

async function createDevnetPool() {
    try {
        const connection = new Connection("https://api.devnet.solana.com", "confirmed");
        
        // Create a new keypair for the pool
        const poolAuthority = Keypair.generate();
        
        // Get some SOL for the pool authority
        const airdropSig = await connection.requestAirdrop(
            poolAuthority.publicKey,
            2 * LAMPORTS_PER_SOL
        );
        await connection.confirmTransaction(airdropSig);
        
        // Constants
        const RAYDIUM_PROGRAM_ID = new PublicKey("devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH");
        const SOL_MINT = new PublicKey("So11111111111111111111111111111111111111112");
        const USDC_MINT = new PublicKey("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");

        // Create LP token mint
        const lpMintKeypair = Keypair.generate();
        const mintRent = await connection.getMinimumBalanceForRentExemption(MINT_SIZE);

        const createAccountTx = new Transaction().add(
            SystemProgram.createAccount({
                fromPubkey: poolAuthority.publicKey,
                newAccountPubkey: lpMintKeypair.publicKey,
                space: MINT_SIZE,
                lamports: mintRent,
                programId: TOKEN_PROGRAM_ID
            }),
            createInitializeMintInstruction(
                lpMintKeypair.publicKey,
                9, // decimals
                poolAuthority.publicKey,
                poolAuthority.publicKey
            )
        );

        await sendAndConfirmTransaction(
            connection,
            createAccountTx,
            [poolAuthority, lpMintKeypair]
        );

        // Create vault accounts
        const vault0 = await getAssociatedTokenAddress(
            SOL_MINT,
            poolAuthority.publicKey,
            true
        );

        const vault1 = await getAssociatedTokenAddress(
            USDC_MINT,
            poolAuthority.publicKey,
            true
        );

        const createVaultsTx = new Transaction().add(
            createAssociatedTokenAccountInstruction(
                poolAuthority.publicKey,
                vault0,
                poolAuthority.publicKey,
                SOL_MINT
            ),
            createAssociatedTokenAccountInstruction(
                poolAuthority.publicKey,
                vault1,
                poolAuthority.publicKey,
                USDC_MINT
            )
        );

        await sendAndConfirmTransaction(
            connection,
            createVaultsTx,
            [poolAuthority]
        );

        // Log all created accounts
        console.log("\nPool Information:");
        console.log("Pool Authority:", poolAuthority.publicKey.toString());
        console.log("LP Token Mint:", lpMintKeypair.publicKey.toString());
        console.log("SOL Vault:", vault0.toString());
        console.log("USDC Vault:", vault1.toString());
        
        // Save pool authority secret key for later use
        console.log("\nPool Authority Secret Key (save this):");
        console.log(JSON.stringify(Array.from(poolAuthority.secretKey)));

        return {
            poolAuthority: poolAuthority.publicKey,
            lpMint: lpMintKeypair.publicKey,
            vault0,
            vault1
        };

    } catch (error) {
        console.error("Error creating pool:", error);
        throw error;
    }
}

// Run the pool creation
createDevnetPool().then(pool => {
    console.log("\nPool creation completed!");
}).catch(error => {
    console.error("Failed to create pool:", error);
});