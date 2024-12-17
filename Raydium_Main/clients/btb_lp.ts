import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BtbLp } from "../target/types/btb_lp";
import { 
  PublicKey, 
  SystemProgram, 
  SYSVAR_RENT_PUBKEY,
  ComputeBudgetProgram
} from "@solana/web3.js";
import { 
  createMint, 
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID
} from "@solana/spl-token";
import BN from "bn.js";
import { SqrtPriceMath } from "@raydium-io/raydium-sdk";

describe("btb_lp", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const owner = anchor.Wallet.local().payer;
  const program = anchor.workspace.BtbLp as Program<BtbLp>;

  // Constants
  const CLMM_PROGRAM_ID = new PublicKey("devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH");
  const AMM_CONFIG = new PublicKey("CQYbhr6amxUER4p5SC44C63R4qw4NFc9Z4Db9vF4tZwG");

  // Test state
  let token0: PublicKey;
  let token1: PublicKey;
  let poolState: PublicKey;
  let observationState: PublicKey;
  let token0Vault: PublicKey;
  let token1Vault: PublicKey;
  let tickArrayBitmap: PublicKey;
  let bitmapExtension: PublicKey;

  const confirmOptions = {
    skipPreflight: true,
  };

  it("Create test tokens", async () => {
    try {
      token0 = await createMint(
        provider.connection,
        owner,
        owner.publicKey,
        null,
        9
      );

      token1 = await createMint(
        provider.connection,
        owner,
        owner.publicKey,
        null,
        9
      );

      // Ensure token0 < token1
      if (token0.toBuffer().compare(token1.toBuffer()) > 0) {
        [token0, token1] = [token1, token0];
      }

      console.log("Test tokens created:");
      console.log("Token0:", token0.toString());
      console.log("Token1:", token1.toString());
    } catch (error) {
      console.error("Error creating test tokens:", error);
      throw error;
    }
  });

  
});