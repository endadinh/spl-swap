import { TokenSwap } from "./../target/types/token_swap";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import {
  SolanaConfigService,
  TestAccountService,
} from "@coin98/solana-support-library/config";
import {
  TokenProgramService,
  TOKEN_PROGRAM_ID,
} from "@coin98/solana-support-library";

describe("test", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.TokenSwap as Program<TokenSwap>;

  it("Is initialized!", async () => {
    // Defined 


    const POOL_SEED_1 = [101, 191, 209, 12, 36, 241, 255, 11];
    const SIGNER_SEED_1 = [240, 191, 125, 12, 36, 152, 255, 16];

    const pool_seeds = Buffer.from(POOL_SEED_1);
    const signer_seeds = Buffer.from(SIGNER_SEED_1);

    const connection = new anchor.web3.Connection(
      // "https://api.devnet.solana.com ",
      "http://localhost:8899",
      "confirmed"
    );


    // Define accounts to test
    const mainAccount = await SolanaConfigService.getDefaultAccount();
    const mintAccountA = await TestAccountService.getAccount(2);
    const mintAccountB = await TestAccountService.getAccount(6);

    const [poolPDA_address, poolPDA_nonce] =
      await anchor.web3.PublicKey.findProgramAddress(
        [pool_seeds],
        program.programId
      );

    const system_program = await anchor.web3.SystemProgram.programId;


    const [poolSignerAddress, nonce] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          signer_seeds,
          poolPDA_address.toBuffer(),
        ],
        program.programId
      );
    // console.log('')

    const tokenMintA = await TokenProgramService.createTokenMint(
      connection,
      mainAccount,
      mintAccountA,
      6,
      mainAccount.publicKey,
      mainAccount.publicKey
    );
    const tokenMintB = await TokenProgramService.createTokenMint(
      connection,
      mainAccount,
      mintAccountB,
      6,
      mainAccount.publicKey,
      mainAccount.publicKey
    );

    /// Create new ATA.

    const ATA_Account_A =
      await TokenProgramService.createAssociatedTokenAccount(
        connection,
        mainAccount,
        mainAccount.publicKey,
        tokenMintA.publicKey
      );

    const ATA_Account_B =
      await TokenProgramService.createAssociatedTokenAccount(
        connection,
        mainAccount,
        mainAccount.publicKey,
        tokenMintB.publicKey
      );

    const ATA_Account_A_POOL =
      await TokenProgramService.createAssociatedTokenAccount(
        connection,
        mainAccount,
        poolSignerAddress,
        tokenMintA.publicKey
      );

    const ATA_Account_B_POOL =
      await TokenProgramService.createAssociatedTokenAccount(
        connection,
        mainAccount,
        poolSignerAddress,
        tokenMintB.publicKey
      );

    //   const ATA_Account_A = TokenProgramService.findAssociatedTokenAddress(
    //           mainAccount.publicKey,
    //           tokenMintA.publicKey

    //   );

    //   const ATA_Account_B = TokenProgramService.findAssociatedTokenAddress(
    //     mainAccount.publicKey,
    //     tokenMintB.publicKey
    // );

    // /// Init new pool ()
        try { 
          const tx = await program.methods.initPool()
          .accounts({
            owner: mainAccount.publicKey,
            poolAccount: poolPDA_address,
            systemProgram: system_program
          })
          .rpc();
          console.log("Your transaction signature", tx);

        }
        catch(error) { 
          console.log('error',error)
        }

    /// Setting rate

    const setting_rate = await program.methods
      .settingRate(new anchor.BN(2))
      .accounts({
        admin: mainAccount.publicKey,
        poolAccount: poolPDA_address,
      })
      .rpc();

    console.log("Your transaction signature", setting_rate);

    // Mint token A,B to pool :

    const mintTokenTx = await program.methods
      .mintToken(new anchor.BN(1000000000))
      .accounts({
        mintAccountA: mintAccountA.publicKey,
        mintAccountB: mintAccountB.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        poolTokenAccountA: ATA_Account_A_POOL,
        poolTokenAccountB: ATA_Account_B_POOL,
        authority: mainAccount.publicKey,
      })
      .signers([mainAccount])
      .rpc();
    console.log("mint transaction hash", mintTokenTx);

    // Mint token A,B to swapper :

    const mintTokenToSwapperTx = await program.methods
      .mintToken(new anchor.BN(1000000000))
      .accounts({
        mintAccountA: mintAccountA.publicKey,
        mintAccountB: mintAccountB.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        poolTokenAccountA: ATA_Account_A,
        poolTokenAccountB: ATA_Account_B,
        authority: mainAccount.publicKey,
      })
      .signers([mainAccount])
      .rpc();
    console.log("mint transaction hash", mintTokenToSwapperTx);

    // Swap token
    try {
      const swapTokenTx = await program.methods
        .swapToken(new anchor.BN(1000000))
        .accounts({
          swapper: mainAccount.publicKey,
          poolAccount: poolPDA_address,
          poolSigner: poolSignerAddress,
          swapperAtaTokenA: ATA_Account_A,
          swapperAtaTokenB: ATA_Account_B,
          tokenA: tokenMintA.publicKey,
          tokenB: tokenMintB.publicKey,
          poolAtaTokenA: ATA_Account_A_POOL,
          poolAtaTokenB: ATA_Account_B_POOL,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([mainAccount])
        .rpc();
      console.log("Swap token hash", swapTokenTx);
    } catch (error) {
      console.log("swap error", error);
    }
  });
});
