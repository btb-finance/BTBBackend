import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Btbdefipool } from "../target/types/btbdefipool";

describe("btbdefipool", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Btbdefipool as Program<Btbdefipool>;

  it("Is initialized!", async () => {
    // Add your test here.
    const text = "First Investor";
    const tx = await program.methods.initialize(text).rpc();
    console.log("Your transaction signature", tx);
  });
});
