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

  const initializeOrderSideAccounts = async () => {

    const lamports = await connection.getMinimumBalanceForRentExemption(MAX_ACCOUNT_SPACE);
    
    // const [bidAccount] = anchor.web3.PublicKey.findProgramAddressSync([
    //   Buffer.from("bids"),
    //   market.toBuffer(),
    // ], PROGRAM_ID);

    // const [askAccount] = anchor.web3.PublicKey.findProgramAddressSync([
    //   Buffer.from("asks"),
    //   market.toBuffer(),
    // ], PROGRAM_ID);

    const bidAccount = anchor.web3.Keypair.generate();
    const askAccount = anchor.web3.Keypair.generate();

    const tx = new anchor.web3.Transaction();

    tx.add(
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


    const sig = await connection.sendTransaction(tx, [keypair, bidAccount, askAccount]);
    await connection.confirmTransaction(sig, "confirmed");
    
    console.log("created accounts", bidAccount.publicKey.toBase58(), askAccount.publicKey.toBase58());
    return {bidAccount, askAccount};
  } 


  it("Should create Market !", async () => {


    const {bidAccount, askAccount} = await initializeOrderSideAccounts();

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

    const market = anchor.web3.Keypair.generate();

    await program.methods
    .createMarket({
      name:"SOL_USDC",
      minBaseOrderAmount: new anchor.BN(1000),
      minQuoteOrderAmount: new anchor.BN(1000),
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
    .rpc({commitment: "confirmed"});


    await program.methods
    .createBooksideAccounts()
    .accounts({
      asks: askAccount.publicKey.toBase58(),
      bids: bidAccount.publicKey.toBase58(),
      market: market.publicKey.toBase58(),
    })
    .rpc({commitment: "confirmed"});


    const marketAccount = await program.account.market.fetch(market.publicKey);
    console.log("MARKET ACCOUNT IS : ");
    console.log(marketAccount);

    const bidsAccount = await program.account.bookSide.fetch(bidAccount.publicKey);
    console.log("BIDS ACCOUNT IS : ");
    console.log(bidsAccount);

    const asksAccount = await program.account.bookSide.fetch(askAccount.publicKey);
    console.log("ASKS ACCOUNT IS : ");
    console.log(asksAccount);
  });

  ;
});
