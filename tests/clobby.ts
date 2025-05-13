import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Clobby } from "../target/types/clobby";
import {createMint, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_2022_PROGRAM_ID} from "@solana/spl-token";

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
  const market = anchor.web3.Keypair.generate();
  const baseToken = anchor.web3.Keypair.generate();
  const quoteToken = anchor.web3.Keypair.generate();

  const getMarketAuthority = () => {
    const [marketAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("market"),
        market.publicKey.toBuffer()
      ],
      PROGRAM_ID,
    );
    return marketAuthority;
  }

  const getBalanceAccount = () => {
    const [balanceAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("balance"),
        keypair.publicKey.toBuffer()
      ],
      PROGRAM_ID,
    );
    return balanceAccount;
  }

  const userBalanceAccount = getBalanceAccount();
  const marketAuthority = getMarketAuthority();

  const baseTokenVault = getAssociatedTokenAddressSync(baseToken.publicKey, marketAuthority, true, TOKEN_2022_PROGRAM_ID);
  const quoteTokenVault= getAssociatedTokenAddressSync(quoteToken.publicKey, marketAuthority, true, TOKEN_2022_PROGRAM_ID);

  let userBaseTokenAccount: anchor.web3.PublicKey;
  let userQuoteTokenAccount: anchor.web3.PublicKey;


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

    return {bidAccount, askAccount};
  } 


  it("Should create Market !", async () => {

    await Promise.all([
      createMint(
        connection,
        keypair,
        keypair.publicKey,
        null,
        9,
        baseToken,
        {commitment:"confirmed"},
        TOKEN_2022_PROGRAM_ID
      ),
      createMint(
        connection,
        keypair,
        keypair.publicKey,
        null,
        6,
        quoteToken,
        {commitment:"confirmed"},
        TOKEN_2022_PROGRAM_ID
      )

    ])

    const [baseTokenResult, quoteTokenResult] = await Promise.all([
      getOrCreateAssociatedTokenAccount(
        connection,
        keypair,
        baseToken.publicKey,
        keypair.publicKey,
        undefined,
        undefined,
        {commitment:"confirmed"},
        TOKEN_2022_PROGRAM_ID
      ),
      getOrCreateAssociatedTokenAccount(
        connection,
        keypair,
        quoteToken.publicKey,
        keypair.publicKey,
        undefined,
        undefined,
        {commitment:"confirmed"},
        TOKEN_2022_PROGRAM_ID
      )
    ])

    userBaseTokenAccount = baseTokenResult.address;
    userQuoteTokenAccount = quoteTokenResult.address;

    await Promise.all([
      mintTo(
        connection,
        keypair,
        baseToken.publicKey,
        userBaseTokenAccount,
        keypair,
        10_000_000_000,
        undefined,
        undefined,
        TOKEN_2022_PROGRAM_ID
      ),
      mintTo(
        connection,
        keypair,
        quoteToken.publicKey,
        userQuoteTokenAccount,
        keypair,
        10_000_000_000,
        undefined,
        undefined,
        TOKEN_2022_PROGRAM_ID
      )
    ]) 
  
    console.log("Base Token : ", baseToken.publicKey.toBase58());
    console.log("Quote Token : ", quoteToken.publicKey.toBase58());
    console.log("Market Account", market.publicKey.toBase58());
    console.log("Market Event Account", marketEvent.publicKey.toBase58());
    console.log("Market Authority Account", marketAuthority.toBase58());
    
    console.log("Base Token Vault : ", baseTokenVault.toBase58());
    console.log("Quote Token Vault : ", quoteTokenVault.toBase58());


    const {bidAccount, askAccount} = await initOrderSideAndEventAccounts();

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
      baseToken: baseToken.publicKey.toBase58(),
      quoteToken: quoteToken.publicKey.toBase58(),
      marketEvents: marketEvent.publicKey.toBase58(),
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

    const sig = await anchor.web3.sendAndConfirmTransaction(connection, tx, [keypair, market], {commitment: "confirmed", skipPreflight: true});
    console.log("Create Market Signature is : ", sig);
  });

  it("Should place a Bid order and sit on the orderbook !", async () => {

    const sig = await program.methods
    .placeOrder({
      baseLots: 2, // Buy two base lots
      ioc: false, // Non-immediate-or-cancel order
      quoteAmount: new anchor.BN(1000), // Buy 1000 quote tokens
      side: {bid:{}},
    })
    .accounts({
      user: keypair.publicKey.toBase58(),
      userTokenAccount: userQuoteTokenAccount.toBase58(),
      market: market.publicKey.toBase58(),
      tokenToTrade: quoteToken.publicKey.toBase58(),
      tokenVault: quoteTokenVault.toBase58(),
      bids: bidAccount.publicKey.toBase58(),
      asks: askAccount.publicKey.toBase58(),   
      marketEvents: marketEvent.publicKey.toBase58(),
      tokenProgram: TOKEN_2022_PROGRAM_ID,
    })
    .rpc({commitment: "confirmed"});

    console.log("Bid Order Signature is : ", sig);
  });

  it("Should match completely!", async() => {

    const ix1 = await program.methods
    .placeOrder({
      baseLots: 2, // Buy two base lots
      ioc: false, // Non-immediate-or-cancel order
      quoteAmount: new anchor.BN(1000), // Buy 1000 quote tokens
      side: {bid:{}},
    })
    .accounts({
      user: keypair.publicKey.toBase58(),
      userTokenAccount: userQuoteTokenAccount.toBase58(),
      market: market.publicKey.toBase58(),
      tokenToTrade: quoteToken.publicKey.toBase58(),
      tokenVault: quoteTokenVault.toBase58(),
      bids: bidAccount.publicKey.toBase58(),
      asks: askAccount.publicKey.toBase58(),   
      marketEvents: marketEvent.publicKey.toBase58(),
      tokenProgram: TOKEN_2022_PROGRAM_ID,
    })
    .instruction();

    const ix2 = await program.methods
    .placeOrder({
      baseLots: 2, // Sell two base lots
      ioc: false, // Non-immediate-or-cancel order
      quoteAmount: new anchor.BN(1000), // Sell each at the price 1000 quote tokens
      side: {ask:{}},
    })
    .accounts({
      user: keypair.publicKey.toBase58(),
      userTokenAccount: userBaseTokenAccount.toBase58(),
      market: market.publicKey.toBase58(),
      tokenToTrade: baseToken.publicKey.toBase58(),
      tokenVault: baseTokenVault.toBase58(),
      bids: bidAccount.publicKey.toBase58(),
      asks: askAccount.publicKey.toBase58(),   
      marketEvents: marketEvent.publicKey.toBase58(),
      tokenProgram: TOKEN_2022_PROGRAM_ID,
    })
    .instruction();

    const tx = new anchor.web3.Transaction();

    tx.add(
      ix1,
      ix2
    );

    const sig = await anchor.web3.sendAndConfirmTransaction(connection, tx, [keypair], {commitment: "confirmed", skipPreflight: true});
    console.log("Complete Match Signature is : ", sig);


  });

  it("Should match partially and sit on the orderbook", async () => {
    const ix1 = await program.methods
    .placeOrder({
      baseLots: 1, // Buy two base lots
      ioc: false, // Non-immediate-or-cancel order
      quoteAmount: new anchor.BN(1000), // Buy 1000 quote tokens
      side: {bid:{}},
    })
    .accounts({
      user: keypair.publicKey.toBase58(),
      userTokenAccount: userQuoteTokenAccount.toBase58(),
      market: market.publicKey.toBase58(),
      tokenToTrade: quoteToken.publicKey.toBase58(),
      tokenVault: quoteTokenVault.toBase58(),
      bids: bidAccount.publicKey.toBase58(),
      asks: askAccount.publicKey.toBase58(),   
      marketEvents: marketEvent.publicKey.toBase58(),
      tokenProgram: TOKEN_2022_PROGRAM_ID,
    })
    .instruction();

    const ix2 = await program.methods
    .placeOrder({
      baseLots: 2, // Sell two base lots
      ioc: false, // Non-immediate-or-cancel order
      quoteAmount: new anchor.BN(1000), // Sell each at the price 1000 quote tokens
      side: {ask:{}},
    })
    .accounts({
      user: keypair.publicKey.toBase58(),
      userTokenAccount: userBaseTokenAccount.toBase58(),
      market: market.publicKey.toBase58(),
      tokenToTrade: baseToken.publicKey.toBase58(),
      tokenVault: baseTokenVault.toBase58(),
      bids: bidAccount.publicKey.toBase58(),
      asks: askAccount.publicKey.toBase58(),   
      marketEvents: marketEvent.publicKey.toBase58(),
      tokenProgram: TOKEN_2022_PROGRAM_ID,
    })
    .instruction();

    const tx = new anchor.web3.Transaction();

    tx.add(
      ix1,
      ix2
    );

    const sig = await anchor.web3.sendAndConfirmTransaction(connection, tx, [keypair], {commitment: "confirmed", skipPreflight: true});
    console.log("PARTIAL ORDER SIG", sig);

  })

});
