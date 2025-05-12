import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Clobby } from "../target/types/clobby";
import {makeTokenMint} from "@solana-developers/helpers";
import {TOKEN_2022_PROGRAM_ID} from "@solana/spl-token";

describe("clobby", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const program = anchor.workspace.Clobby as Program<Clobby>;
  const PROGRAM_ID = program.programId;

  const keypair = anchor.Wallet.local().payer;
  const connection = provider.connection;

  const MAX_ACCOUNT_SPACE = 10_485_760; 

  const bidAccount = anchor.web3.Keypair.generate();
  const askAccount = anchor.web3.Keypair.generate();
  const marketEvent = anchor.web3.Keypair.generate();

  const initOrderSideAndEventAccounts = async () => {

    const lamports = await connection.getMinimumBalanceForRentExemption(MAX_ACCOUNT_SPACE);
    
    // const [bidAccount] = anchor.web3.PublicKey.findProgramAddressSync([
    //   Buffer.from("bids"),
    //   market.toBuffer(),
    // ], PROGRAM_ID);

    // const [askAccount] = anchor.web3.PublicKey.findProgramAddressSync([
    //   Buffer.from("asks"),
    //   market.toBuffer(),
    // ], PROGRAM_ID);

    const tx1 = new anchor.web3.Transaction();

    tx1.add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: keypair.publicKey,
        newAccountPubkey: bidAccount.publicKey,
        space: MAX_ACCOUNT_SPACE,
        lamports,
        programId: PROGRAM_ID,
      }),
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: keypair.publicKey,
        newAccountPubkey: askAccount.publicKey,
        space: MAX_ACCOUNT_SPACE,
        lamports,
        programId: PROGRAM_ID,
      }),
    );


    const sig = await connection.sendTransaction(tx1, [keypair, bidAccount, askAccount]);
    await connection.confirmTransaction(sig, "confirmed");
    
    console.log("created bids and asks accounts", bidAccount.publicKey.toBase58(), askAccount.publicKey.toBase58());

    const tx2 = new anchor.web3.Transaction();

    tx2.add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: keypair.publicKey,
        newAccountPubkey: marketEvent.publicKey,
        space: MAX_ACCOUNT_SPACE,
        lamports,
        programId: PROGRAM_ID,
      }),
    );

    const sig2 = await connection.sendTransaction(tx2, [keypair, marketEvent]);
    await connection.confirmTransaction(sig2, "confirmed");
    
    console.log("created market_event accounts", marketEvent.publicKey.toBase58());

    return {bidAccount, askAccount, marketEvent};
  } 


  it("Should create Market !", async () => {
    const market = anchor.web3.Keypair.generate();

    const {bidAccount, askAccount, marketEvent} = await initOrderSideAndEventAccounts();

    const baseToken = await makeTokenMint(
      connection,
      keypair,
      "TRUMP",
      "TRMP",
      6,
      "https://www.google.com"
    );

    const quoteToken = await makeTokenMint(
      connection,
      keypair,
      "WSOL",
      "WSOL",
      9,
      "https://www.solana.com",
      
    )

    const initAuthorityAndEventIx = await program.methods
    .initMarketAuthorityAndEvent()
    .accounts({
      market: market.publicKey.toBase58(),
      marketEvent: marketEvent.publicKey.toBase58(),
      user: keypair.publicKey.toBase58(),
    })
    .signers([keypair])
    .instruction();


    const createBooksideAccountsIx = await program.methods
    .createBooksideAccounts()
    .accounts({
      asks: askAccount.publicKey.toBase58(),
      bids: bidAccount.publicKey.toBase58(),
      market: market.publicKey.toBase58(),
    })
    .instruction();

    const createMarketIx = await program.methods
    .createMarket({
      name:"SOL_USDC",
      baseLotSize: new anchor.BN(1000),
    })
    .accounts({
      market: market.publicKey.toBase58(),
      bids: bidAccount.publicKey.toBase58(),
      asks: askAccount.publicKey.toBase58(),      
      signer: keypair.publicKey.toBase58(),
      baseToken: baseToken.toBase58(),
      quoteToken: quoteToken.toBase58(),
      tokenProgram: TOKEN_2022_PROGRAM_ID,
    })
    .signers([market])
    .instruction();

    const tx = new anchor.web3.Transaction();

    tx.add(
      createMarketIx,
      initAuthorityAndEventIx, 
      createBooksideAccountsIx
    );

    const sig = await anchor.web3.sendAndConfirmTransaction(connection, tx, [keypair, market], {commitment: "confirmed"});
    console.log("Signature is : ", sig);
  });

  ;
});
