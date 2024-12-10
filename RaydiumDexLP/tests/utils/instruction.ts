import { Program, BN } from "@coral-xyz/anchor";
import { ClmmCpi } from "../../target/types/clmm_cpi";
import {
  Connection,
  ConfirmOptions,
  PublicKey,
  Keypair,
  Signer,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";

import { ClmmProgram, configAddress } from "../config";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { ClmmKeys, TickUtils, SqrtPriceMath } from "@raydium-io/raydium-sdk-v2";
import { createTokenMintAndAssociatedTokenAccount } from "./util";
import {
  getNftMetadataAddress,
  getOrcleAccountAddress,
  getPersonalPositionAddress,
  getPoolAddress,
  getPoolVaultAddress,
  getProtocolPositionAddress,
  getTickArrayAddress,
  getTickArrayBitmapAddress,
} from "./pda";

export async function setupInitializeTest(
  connection: Connection,
  owner: Signer,
  transferFeeConfig: { transferFeeBasisPoints: number; MaxFee: number } = {
    transferFeeBasisPoints: 0,
    MaxFee: 0,
  },
  confirmOptions?: ConfirmOptions
) {
  const [{ token0, token0Program }, { token1, token1Program }] =
    await createTokenMintAndAssociatedTokenAccount(
      connection,
      owner,
      new Keypair(),
      transferFeeConfig
    );
  return {
    token0,
    token0Program,
    token1,
    token1Program,
  };
}

export async function initialize(
  program: Program<ClmmCpi>,
  creator: Signer,
  token0: PublicKey,
  token0Program: PublicKey,
  token1: PublicKey,
  token1Program: PublicKey,
  initTick: number,
  confirmOptions?: ConfirmOptions
) {
  const [poolAddress, _bump1] = await getPoolAddress(
    configAddress,
    token0,
    token1,
    ClmmProgram
  );
  const [vault0, _bump2] = await getPoolVaultAddress(
    poolAddress,
    token0,
    ClmmProgram
  );
  const [vault1, _bump3] = await getPoolVaultAddress(
    poolAddress,
    token1,
    ClmmProgram
  );

  const [tick_array_bitmap, _bump4] = await getTickArrayBitmapAddress(
    poolAddress,
    ClmmProgram
  );

  const [observation, _bump5] = await getOrcleAccountAddress(
    poolAddress,
    ClmmProgram
  );

  const [bitmapExtension, _bump111] = await getTickArrayBitmapAddress(
    poolAddress,
    ClmmProgram
  );

  const tx = await program.methods
    .proxyInitialize(SqrtPriceMath.getSqrtPriceX64FromTick(initTick), new BN(0))
    .accounts({
      clmmProgram: ClmmProgram,
      poolCreator: creator.publicKey,
      ammConfig: configAddress,
      poolState: poolAddress,
      tokenMint0: token0,
      tokenMint1: token1,
      tokenVault0: vault0,
      tokenVault1: vault1,
      observationState: observation,
      tickArrayBitmap: tick_array_bitmap,
      tokenProgram0: token0Program,
      tokenProgram1: token1Program,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .remainingAccounts([
      { pubkey: bitmapExtension, isSigner: false, isWritable: true },
    ])
    .preInstructions([
      ComputeBudgetProgram.setComputeUnitLimit({ units: 400000 }),
    ])
    .rpc(confirmOptions);

  return { poolAddress, tx };
}

export async function openPosition(
  program: Program<ClmmCpi>,
  owner: Signer,
  poolKeys: ClmmKeys,
  tickLowerIndex: number,
  tickUpperIndex: number,
  liquidity: BN,
  amount0Max: BN,
  amount1Max: BN,
  confirmOptions?: ConfirmOptions
) {
  // prepare tickArray
  const tickArrayLowerStartIndex = TickUtils.getTickArrayStartIndexByTick(
    tickLowerIndex,
    poolKeys.config.tickSpacing
  );
  const [tickArrayLower] = await getTickArrayAddress(
    new PublicKey(poolKeys.id),
    ClmmProgram,
    tickArrayLowerStartIndex
  );
  const tickArrayUpperStartIndex = TickUtils.getTickArrayStartIndexByTick(
    tickUpperIndex,
    poolKeys.config.tickSpacing
  );
  const [tickArrayUpper] = await getTickArrayAddress(
    new PublicKey(poolKeys.id),
    ClmmProgram,
    tickArrayUpperStartIndex
  );
  const positionNftMint = Keypair.generate();
  const positionANftAccount = getAssociatedTokenAddressSync(
    positionNftMint.publicKey,
    owner.publicKey
  );

  const metadataAccount = (
    await getNftMetadataAddress(positionNftMint.publicKey)
  )[0];

  const [personalPosition] = await getPersonalPositionAddress(
    positionNftMint.publicKey,
    ClmmProgram
  );

  const [protocolPosition] = await getProtocolPositionAddress(
    new PublicKey(poolKeys.id),
    ClmmProgram,
    tickLowerIndex,
    tickUpperIndex
  );

  const token0Account = getAssociatedTokenAddressSync(
    new PublicKey(poolKeys.mintA.address),
    owner.publicKey,
    false,
    new PublicKey(poolKeys.mintA.programId)
  );

  const token1Account = getAssociatedTokenAddressSync(
    new PublicKey(poolKeys.mintB.address),
    owner.publicKey,
    false,
    new PublicKey(poolKeys.mintB.programId)
  );

  const [bitmapExtension, _bump111] = await getTickArrayBitmapAddress(
    new PublicKey(poolKeys.id),
    ClmmProgram
  );

  const tx = await program.methods
    .proxyOpenPosition(
      tickLowerIndex,
      tickUpperIndex,
      tickArrayLowerStartIndex,
      tickArrayUpperStartIndex,
      liquidity,
      amount0Max,
      amount1Max,
      true
    )
    .accounts({
      clmmProgram: ClmmProgram,
      payer: owner.publicKey,
      positionNftOwner: owner.publicKey,
      positionNftMint: positionNftMint.publicKey,
      positionNftAccount: positionANftAccount,
      metadataAccount,
      poolState: new PublicKey(poolKeys.id),
      protocolPosition,
      tickArrayLower,
      tickArrayUpper,
      tokenAccount0: token0Account,
      tokenAccount1: token1Account,
      tokenVault0: new PublicKey(poolKeys.vault.A),
      tokenVault1: new PublicKey(poolKeys.vault.B),
      vault0Mint: new PublicKey(poolKeys.mintA.address),
      vault1Mint: new PublicKey(poolKeys.mintB.address),
      personalPosition,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      tokenProgram2022: TOKEN_2022_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      metadataProgram: new PublicKey(
        "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
      ),
    })
    .remainingAccounts([
      { pubkey: bitmapExtension, isSigner: false, isWritable: true },
    ])
    .preInstructions([
      ComputeBudgetProgram.setComputeUnitLimit({ units: 400000 }),
    ])
    .signers([positionNftMint])
    .rpc(confirmOptions);

  return { positionNftMint, personalPosition, protocolPosition, tx };
}


export async function increaseLiquidity(
  program: Program<ClmmCpi>,
  owner: Signer,
  poolKeys: ClmmKeys,
  positionNftMint: PublicKey,
  tickLowerIndex: number,  // Added parameter
  tickUpperIndex: number,  // Added parameter 
  liquidity: BN,
  amount0Max: BN,
  amount1Max: BN,
  baseFlagOpt: boolean = true,
  confirmOptions?: ConfirmOptions
 ) {
  // Get position nft account
  const positionNftAccount = getAssociatedTokenAddressSync(
    positionNftMint,
    owner.publicKey,
    false,
    TOKEN_PROGRAM_ID
  );
 
  // Get personal position
  const [personalPosition] = await getPersonalPositionAddress(
    positionNftMint,
    ClmmProgram
  );
 
  // Get protocol position
  const [protocolPosition] = await getProtocolPositionAddress(
    new PublicKey(poolKeys.id),
    ClmmProgram,
    tickLowerIndex,
    tickUpperIndex
  );
 
  // Get tick arrays
  const tickLowerArrayStartIndex = TickUtils.getTickArrayStartIndexByTick(
    tickLowerIndex,
    poolKeys.config.tickSpacing
  );
 
  const tickUpperArrayStartIndex = TickUtils.getTickArrayStartIndexByTick(
    tickUpperIndex,  
    poolKeys.config.tickSpacing
  );
 
  const [tickArrayLower] = await getTickArrayAddress(
    new PublicKey(poolKeys.id),
    ClmmProgram,
    tickLowerArrayStartIndex
  );
 
  const [tickArrayUpper] = await getTickArrayAddress( 
    new PublicKey(poolKeys.id),
    ClmmProgram,
    tickUpperArrayStartIndex
  );
 
  // Get token accounts
  const tokenAccount0 = getAssociatedTokenAddressSync(
    new PublicKey(poolKeys.mintA.address),
    owner.publicKey,
    false,
    new PublicKey(poolKeys.mintA.programId)
  );
 
  const tokenAccount1 = getAssociatedTokenAddressSync(
    new PublicKey(poolKeys.mintB.address),
    owner.publicKey,
    false,
    new PublicKey(poolKeys.mintB.programId)
  );
 
  const [bitmapExtension] = await getTickArrayBitmapAddress(
    new PublicKey(poolKeys.id),
    ClmmProgram
  );
 
  const tx = await program.methods
    .proxyIncreaseLiquidity(
      liquidity,
      amount0Max,
      amount1Max,
      baseFlagOpt
    )
    .accounts({
      clmmProgram: ClmmProgram,
      nftOwner: owner.publicKey,
      nftAccount: positionNftAccount,
      poolState: new PublicKey(poolKeys.id),
      protocolPosition: protocolPosition,
      personalPosition: personalPosition,
      tickArrayLower: tickArrayLower,
      tickArrayUpper: tickArrayUpper,
      tokenAccount0: tokenAccount0,
      tokenAccount1: tokenAccount1,
      tokenVault0: new PublicKey(poolKeys.vault.A),
      tokenVault1: new PublicKey(poolKeys.vault.B),
      tokenProgram: TOKEN_PROGRAM_ID,
      tokenProgram2022: TOKEN_2022_PROGRAM_ID,
      vault0Mint: new PublicKey(poolKeys.mintA.address),
      vault1Mint: new PublicKey(poolKeys.mintB.address)
    })
    .remainingAccounts([
      { pubkey: bitmapExtension, isSigner: false, isWritable: true },
    ])
    .preInstructions([
      ComputeBudgetProgram.setComputeUnitLimit({ units: 400000 }),
    ])
    .rpc(confirmOptions);
 
  return { tx };
 }

export async function decreaseLiquidity(
  program: Program<ClmmCpi>,
  owner: Signer,
  poolKeys: ClmmKeys,
  positionNftMint: PublicKey, 
  liquidity: BN,
  tickLowerIndex: number,
  tickUpperIndex: number,
  confirmOptions?: ConfirmOptions
) {
  const positionNftAccount = getAssociatedTokenAddressSync(
    positionNftMint,
    owner.publicKey
  );

  const [personalPosition] = await getPersonalPositionAddress(
    positionNftMint,
    ClmmProgram
  );

  const tickLowerArrayStartIndex = TickUtils.getTickArrayStartIndexByTick(
    tickLowerIndex,
    poolKeys.config.tickSpacing
  );

  const tickUpperArrayStartIndex = TickUtils.getTickArrayStartIndexByTick(
    tickUpperIndex,  
    poolKeys.config.tickSpacing
  );

  const [tickArrayLower] = await getTickArrayAddress(
    new PublicKey(poolKeys.id),
    ClmmProgram,
    tickLowerArrayStartIndex
  );

  const [tickArrayUpper] = await getTickArrayAddress( 
    new PublicKey(poolKeys.id),
    ClmmProgram,
    tickUpperArrayStartIndex
  );

  const [protocolPosition] = await getProtocolPositionAddress(
    new PublicKey(poolKeys.id),
    ClmmProgram,
    tickLowerIndex,
    tickUpperIndex
  );

  const recipientToken0Account = getAssociatedTokenAddressSync(
    new PublicKey(poolKeys.mintA.address),
    owner.publicKey,
    false,
    new PublicKey(poolKeys.mintA.programId)
  );

  const recipientToken1Account = getAssociatedTokenAddressSync(
    new PublicKey(poolKeys.mintB.address),
    owner.publicKey,
    false,
    new PublicKey(poolKeys.mintB.programId)
  );

  const [bitmapExtension] = await getTickArrayBitmapAddress(
    new PublicKey(poolKeys.id),
    ClmmProgram
  );

  const tx = await program.methods
    .proxyDecreaseLiquidity(liquidity, new BN(0), new BN(0))
    .accounts({
      clmmProgram: ClmmProgram,
      nftOwner: owner.publicKey,
      nftAccount: positionNftAccount,
      personalPosition: personalPosition,
      poolState: new PublicKey(poolKeys.id),
      protocolPosition: protocolPosition,
      tokenVault0: new PublicKey(poolKeys.vault.A),
      tokenVault1: new PublicKey(poolKeys.vault.B),
      tickArrayLower: tickArrayLower,
      tickArrayUpper: tickArrayUpper,
      recipientTokenAccount0: recipientToken0Account,
      recipientTokenAccount1: recipientToken1Account,
      tokenProgram: TOKEN_PROGRAM_ID,
      tokenProgram2022: TOKEN_2022_PROGRAM_ID,
      memoProgram: new PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
      vault0Mint: new PublicKey(poolKeys.mintA.address),
      vault1Mint: new PublicKey(poolKeys.mintB.address)
    })
    .remainingAccounts([
      { pubkey: bitmapExtension, isSigner: false, isWritable: true },
    ])
    .preInstructions([
      ComputeBudgetProgram.setComputeUnitLimit({ units: 400000 }),
    ])
    .rpc(confirmOptions);

  return { tx };
}

export async function closePosition(
  program: Program<ClmmCpi>,
  owner: Signer,
  positionNftMint: PublicKey,
  confirmOptions?: ConfirmOptions 
) {
  const positionNftAccount = getAssociatedTokenAddressSync(
    positionNftMint, 
    owner.publicKey,
    false,
    TOKEN_PROGRAM_ID
  );

  const [personalPosition] = await getPersonalPositionAddress(
    positionNftMint,
    ClmmProgram
  );

  const tx = await program.methods
    .proxyClosePosition()
    .accounts({
      clmmProgram: ClmmProgram,
      nftOwner: owner.publicKey,
      positionNftMint: positionNftMint,
      positionNftAccount: positionNftAccount,
      personalPosition: personalPosition,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .preInstructions([
      ComputeBudgetProgram.setComputeUnitLimit({ units: 400000 }),
    ])
    .rpc(confirmOptions);

  return { tx };
}

