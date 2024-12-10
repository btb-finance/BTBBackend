import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ClmmCpi } from "../target/types/clmm_cpi";
import { setupInitializeTest, initialize, openPosition, increaseLiquidity } from "./utils";
import { Raydium } from "@raydium-io/raydium-sdk-v2";

describe("increase position test", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const owner = anchor.Wallet.local().payer;
  const connection = anchor.getProvider().connection;
  const program = anchor.workspace.ClmmCpi as Program<ClmmCpi>;

  const confirmOptions = {
    skipPreflight: true,
  };

  it("increase position", async () => {
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

    // Define tick ranges and liquidity
    const tickLowerIndex = -10;
    const tickUpperIndex = 10;
    const initialLiquidity = new anchor.BN(10100000);

    // Open position first
    const { positionNftMint } = await openPosition(
      program,
      owner,
      data.poolKeys,
      tickLowerIndex,
      tickUpperIndex,
      initialLiquidity,
      new anchor.BN(10100000000),
      new anchor.BN(10100000000),
      confirmOptions
    );

    console.log("Position opened successfully");

    // Increase liquidity in position
    const additionalLiquidity = new anchor.BN(5000000);

    const { tx } = await increaseLiquidity(
      program,
      owner,
      data.poolKeys,
      positionNftMint.publicKey,
      tickLowerIndex,
      tickUpperIndex,
      additionalLiquidity,
      new anchor.BN(5000000000),
      new anchor.BN(5000000000),
      true, // baseFlag added
      confirmOptions
    );

    console.log("increase liquidity tx:", tx);
  });
});