import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor"; 
import { ClmmCpi } from "../target/types/clmm_cpi";
import { setupInitializeTest, initialize, openPosition, closePosition, decreaseLiquidity } from "./utils";
import { Raydium } from "@raydium-io/raydium-sdk-v2";

describe("close position test", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const owner = anchor.Wallet.local().payer;
  const connection = anchor.getProvider().connection;
  const program = anchor.workspace.ClmmCpi as Program<ClmmCpi>;

  const confirmOptions = {
    skipPreflight: true,
  };

  it("close position", async () => {
    // First create pool and tokens
    const { token0, token0Program, token1, token1Program } =
      await setupInitializeTest(
        connection, 
        owner,
        { transferFeeBasisPoints: 0, MaxFee: 0 },
        confirmOptions
      );

    // Initialize pool
    const { poolAddress } = await initialize(
      program,
      owner, 
      token0,
      token0Program,
      token1,
      token1Program,
      0,
      confirmOptions
    );

    // Get pool info
    const raydium = await Raydium.load({
      owner,
      connection,
    });

    const data = await raydium.clmm.getPoolInfoFromRpc(poolAddress.toString());

    // Define tick range and liquidity
    const tickLowerIndex = -10;
    const tickUpperIndex = 10;
    const liquidity = new anchor.BN(10100000);

    // Open position first
    const { positionNftMint } = await openPosition(
      program,
      owner,
      data.poolKeys,  
      tickLowerIndex,
      tickUpperIndex,
      liquidity,
      new anchor.BN(10100000000),
      new anchor.BN(10100000000),
      confirmOptions
    );

    console.log("Position opened successfully");

    // Decrease liquidity to 0 before closing
    const { tx: decreaseTx } = await decreaseLiquidity(
      program,
      owner,
      data.poolKeys,
      positionNftMint.publicKey,
      liquidity,
      tickLowerIndex,
      tickUpperIndex,
      confirmOptions
    );

    console.log("Liquidity decreased successfully, tx:", decreaseTx);

    // Now close the position
    const { tx: closeTx } = await closePosition(
      program,
      owner,
      positionNftMint.publicKey,
      confirmOptions
    );

    console.log("Position closed successfully, tx:", closeTx);
  });
});