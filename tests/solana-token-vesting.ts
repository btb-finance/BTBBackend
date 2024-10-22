// tests/solana-token-vesting.ts

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaTokenVesting } from "../target/types/solana_token_vesting";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { expect } from "chai";

describe("solana-token-vesting", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaTokenVesting as Program<SolanaTokenVesting>;
  
  let mint: PublicKey;
  let vestingAccount: PublicKey;
  let vestingTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let authority: Keypair;

  before(async () => {
    authority = Keypair.generate();
    await provider.connection.requestAirdrop(authority.publicKey, 10 * LAMPORTS_PER_SOL);

    mint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      9
    );

    userTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      authority,
      mint,
      authority.publicKey
    );

    [vestingAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("vesting_account")],
      program.programId
    );

    vestingTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      authority,
      mint,
      vestingAccount,
      true // allowOwnerOffCurve
    );

    await mintTo(
      provider.connection,
      authority,
      mint,
      vestingTokenAccount,
      authority.publicKey,
      1000000000 // 1000 tokens
    );
  });

  it("Initializes the vesting account", async () => {
    const devaiConfig = {
      model: "gpt-3.5-turbo",
      temperature: 0.7,
      maxTokens: 100,
    };

    await program.methods
      .initialize(devaiConfig)
      .accounts({
        vestingAccount,
        authority: authority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    const account = await program.account.vestingAccount.fetch(vestingAccount);
    expect(account.authority.toString()).to.equal(authority.publicKey.toString());
    expect(account.devaiConfig).to.deep.equal(devaiConfig);
  });

  it("Creates a vesting schedule", async () => {
    const now = Math.floor(Date.now() / 1000);
    const startTime = now + 60; // Start in 1 minute
    const endTime = now + 3660; // End in 1 hour and 1 minute
    const amount = new anchor.BN(1000000000); // 1000 tokens
    const periods = new anchor.BN(60); // 60 periods (1 per minute)

    await program.methods
      .createVestingSchedule(amount, new anchor.BN(startTime), new anchor.BN(endTime), periods)
      .accounts({
        vestingAccount,
        vestingSchedule: vestingAccount,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const account = await program.account.vestingAccount.fetch(vestingAccount);
    expect(account.totalAmount.toString()).to.equal(amount.toString());
    expect(account.startTime.toString()).to.equal(startTime.toString());
    expect(account.endTime.toString()).to.equal(endTime.toString());
    expect(account.periods.toString()).to.equal(periods.toString());
  });

  it("Claims vested tokens", async () => {
    // Fast-forward time by 30 minutes
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(authority.publicKey, 1),
      "confirmed"
    );
    
    await program.methods
      .claimVesting()
      .accounts({
        vestingAccount,
        vestingTokenAccount,
        userTokenAccount,
        authority: authority.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();

    const userBalance = await provider.connection.getTokenAccountBalance(userTokenAccount);
    expect(parseInt(userBalance.value.amount)).to.be.above(0);
    expect(parseInt(userBalance.value.amount)).to.be.below(1000000000);
  });

  it("Retrieves vesting information", async () => {
    const tx = await program.methods
      .getVestingInfo()
      .accounts({
        vestingAccount,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const txResult = await provider.connection.getTransaction(tx, { commitment: "confirmed" });
    expect(txResult.meta.logMessages.some(log => log.includes("Vesting Info:"))).to.be.true;
  });
});