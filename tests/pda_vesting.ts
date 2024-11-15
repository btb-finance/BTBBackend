import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PdaVesting } from "../target/types/pda_vesting";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";

describe("pda-vesting", () => {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PdaVesting as Program<PdaVesting>;
  
  // Basic addresses we'll use
  const btbMint = new anchor.web3.PublicKey("btbVv5dmAjutpRRSr6DKwBPyPyfKiJw4eXU11BPuTCK");
  
  const usdtMint = new anchor.web3.PublicKey("utK7s5CmT6vvkd3JpTg5CfMaqAS8uVMwnqZjPZvcLkD");
  const usdcMint = new anchor.web3.PublicKey("ucKymLwwPxrPaUDMtYGx5uoho91SfE3Qs2VuXf9dDZB");
  const paypalMint = new anchor.web3.PublicKey("pa3x7zKXd2yvPNM8NxJUp1tu1j8xeXyRb6Y65yqPvtQ");
  const receiveWallet = new anchor.web3.PublicKey("te6eqhHuXFuhP1bjBfPs17VS84dR1M725FR9txASuCS");

  it("Initialize BTB Sale", async () => {
    // Get PDA
    const [btbSaleAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("btb-sale-account"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    // Create associated token account for PDA to hold BTB tokens
    const btbSaleTokenAccount = await getAssociatedTokenAddress(btbMint, btbSaleAccount, true);

    console.log('btbSaleTokenAccount' , btbSaleTokenAccount)

    // Initialize the program
    // const tx = await program.methods
    //   .initialize(
    //     btbMint,
    //     usdtMint,
    //     usdcMint,
    //     paypalMint,
    //     receiveWallet,
    //     new anchor.BN(1000),
    //     new anchor.BN(8.08)
    //   )
    //   .accounts({
    //     btbSaleAccount: btbSaleAccount,
    //     btbSaleTokenAccount: btbSaleTokenAccount,
    //     btbMintAccount: btbMint,
    //     signer: provider.wallet.publicKey,
    //     systemProgram: anchor.web3.SystemProgram.programId,
    //     tokenProgram: TOKEN_PROGRAM_ID,
    //     associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    //   })
    //   .rpc();

    // expect(tx).to.be.a("string");
  });

  it("Update Initialize Parameters", async () => {
    // Get PDA
    const [btbSaleAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("btb-sale-account"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    // Update the initialization parameters
    const tx = await program.methods
      .updateInitialize(
        btbMint,
        usdtMint,
        usdcMint,
        paypalMint,
        receiveWallet,
        new anchor.BN(1200),
        new anchor.BN(9.99)
      )
      .accounts({
        btbSaleAccount: btbSaleAccount,
        signer: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    expect(tx).to.be.a("string");
  });

  it("Toggle Sale Status", async () => {
    // Get PDA
    const [btbSaleAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("btb-sale-account"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    // Toggle sale status
    const tx = await program.methods
      .toggleSale()
      .accounts({
        btbSaleAccount: btbSaleAccount,
        signer: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    expect(tx).to.be.a("string");
  });

  // it("Buy Token", async () => {
  //   // Get PDA and required accounts
  //   const [btbSaleAccount] = await anchor.web3.PublicKey.findProgramAddress(
  //     [Buffer.from("btb-sale-account"), provider.wallet.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   // Get btbSaleTokenAccount
  //   const btbSaleTokenAccount = await getAssociatedTokenAddress(btbMint, btbSaleAccount, true);

  //   // Get user's USDT token account
  //   const userTokenAccount = await getAssociatedTokenAddress(
  //     usdtMint,
  //     provider.wallet.publicKey,
  //     false
  //   );

  //   // Get user's BTB token account
  //   const userBtbAccount = await getAssociatedTokenAddress(
  //     btbMint,
  //     provider.wallet.publicKey,
  //     false
  //   );

  //   // Buy tokens
  //   const tx = await program.methods
  //     .buyToken(
  //       new anchor.BN(100), // Amount to buy
  //       0 // Token type (0 for USDT)
  //     )
  //     .accounts({
  //       btbSaleAccount: btbSaleAccount,
  //       userTokenAccount: userTokenAccount,
  //       ownerTokenAccount: receiveWallet,
  //       btbSaleTokenAccount: btbSaleTokenAccount,
  //       userBtbAccount: userBtbAccount,
  //       btbMintAccount: btbMint,
  //       user: provider.wallet.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //     })
  //     .rpc();

  //   expect(tx).to.be.a("string");
  // });
});