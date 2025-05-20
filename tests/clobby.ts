import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Clobby } from "../target/types/clobby";
import {createMint, getAccount, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_2022_PROGRAM_ID} from "@solana/spl-token";
import { expect } from "chai";

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
    const result = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("market"),
        market.publicKey.toBuffer()
      ],
      PROGRAM_ID,
    );
    return result;
  }

  const getBalanceAccount = (userKey: anchor.web3.PublicKey) => {
    const [balanceAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("balance"),
        userKey.toBuffer()
      ],
      PROGRAM_ID,
    );
    return balanceAccount;
  }

  const userBalanceAccount = getBalanceAccount(keypair.publicKey);
  const [marketAuthority, marketAuthorityBump] = getMarketAuthority();

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
      consumeEventsAuthority: keypair.publicKey,
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

    const marketAcc = await program.account.market.fetch(market.publicKey);

    // create market tests
    expect(marketAcc.asks.toBase58()).to.equal(askAccount.publicKey.toBase58());
    expect(marketAcc.bids.toBase58()).to.equal(bidAccount.publicKey.toBase58());
    expect(marketAcc.baseToken.toBase58()).to.equal(baseToken.publicKey.toBase58());
    expect(marketAcc.quoteToken.toBase58()).to.equal(quoteToken.publicKey.toBase58());
    expect(marketAcc.baseTokenVault.toBase58()).to.equal(baseTokenVault.toBase58());
    expect(marketAcc.quoteTokenVault.toBase58()).to.equal(quoteTokenVault.toBase58());
    expect(marketAcc.name).to.equal("SOL_USDC");
    expect(marketAcc.baseLotSize.toNumber()).to.equal(1000);
    expect(marketAcc.marketAuthority.toBase58()).to.equal(marketAuthority.toBase58());
    expect(marketAcc.marketEvents.toBase58()).to.equal(marketEvent.publicKey.toBase58());
    expect(marketAcc.marketAuthorityBump).to.equal(marketAuthorityBump);
    expect(marketAcc.totalOrders.toNumber()).to.equal(new anchor.BN(0).toNumber());
    expect(marketAcc.consumeEventsAuthority.toBase58()).to.equal(keypair.publicKey.toBase58());

    // init_market_authority_and_event tests
    const marketEventAcc = await program.account.marketEvents.fetch(marketEvent.publicKey);
    const marketAuthorityAcc = await connection.getParsedAccountInfo(marketAuthority);
    //@ts-ignore
    expect(marketAuthorityAcc.value.space).to.equal(130);
    expect(marketAuthorityAcc.value.owner.toBase58()).to.equal(PROGRAM_ID.toBase58());
    expect(marketAuthorityAcc.value.lamports).to.gt(0);
    expect(marketEventAcc.market.toBase58()).to.equal(market.publicKey.toBase58());
    expect(marketEventAcc.eventsToProcess.toNumber()).to.equal(0);
    
    // create_bookside_accounts tests
    const asksAcc = await program.account.bookSide.fetch(askAccount.publicKey);
    const bidsAcc = await program.account.bookSide.fetch(bidAccount.publicKey);

    expect(bidsAcc.side.toNumber()).to.equal(0);
    expect(bidsAcc.orderCount.toNumber()).to.equal(0);
    expect(bidsAcc.marketAccount.toBase58()).to.equal(market.publicKey.toBase58());
    expect(bidsAcc.orders.length).to.equal(1024);

    expect(asksAcc.side.toNumber()).to.equal(1);
    expect(asksAcc.orderCount.toNumber()).to.equal(0);  
    expect(asksAcc.marketAccount.toBase58()).to.equal(market.publicKey.toBase58());
    expect(asksAcc.orders.length).to.equal(1024);


  });

  it("Should create User Balance Account !", async () => {

    const userBalanceAccount = getBalanceAccount(keypair.publicKey);

    const createBalanceSig = await program.methods
    .createUserBalanceAccount()
    .accounts({
      user: keypair.publicKey.toBase58(),
      market: market.publicKey.toBase58(),
    })
    .rpc({commitment: "confirmed"});

    console.log("Create Balance Account Signature is : ", createBalanceSig);

    const balanceAccount = await program.account.userBalance.fetch(userBalanceAccount);
    expect(balanceAccount.baseAmount.toNumber()).to.equal(0);
    expect(balanceAccount.quoteAmount.toNumber()).to.equal(0);
    expect(balanceAccount.user.toBase58()).to.equal(keypair.publicKey.toBase58());
    expect(balanceAccount.market.toBase58()).to.equal(market.publicKey.toBase58());
    expect(balanceAccount.baseToken.toBase58()).to.equal(baseToken.publicKey.toBase58());
    expect(balanceAccount.quoteToken.toBase58()).to.equal(quoteToken.publicKey.toBase58());

  });


  it("Should place a Bid order and sit on the orderbook !", async () => {

    /*
      check whether appropriate quote amount is transferred to the market vault account
      check whether the order is added to the bids account
      check the user onchain balance account is zero
    */

    const userQuoteTokenAccountBefore = await getAccount(connection, userQuoteTokenAccount, undefined, TOKEN_2022_PROGRAM_ID);
    const marketQuoteTokenVaultBefore = await getAccount(connection, quoteTokenVault, undefined, TOKEN_2022_PROGRAM_ID);

    const start = Date.now();

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

    const now = Date.now();

    const timeTaken = now - start;
    console.log(`Time taken to place order is ${timeTaken} ms`);

    const userQuoteTokenAccountAfter = await getAccount(connection, userQuoteTokenAccount, undefined, TOKEN_2022_PROGRAM_ID);
    const marketQuoteTokenVaultAfter = await getAccount(connection, quoteTokenVault, undefined, TOKEN_2022_PROGRAM_ID);

    expect(userQuoteTokenAccountAfter.amount).to.equal(userQuoteTokenAccountBefore.amount - BigInt(1000));
    expect(marketQuoteTokenVaultAfter.amount).to.equal(marketQuoteTokenVaultBefore.amount + BigInt(1000));

    const bidsAcc = await program.account.bookSide.fetch(bidAccount.publicKey);
    const marketAcc = await program.account.market.fetch(market.publicKey);

    expect(marketAcc.totalOrders.toNumber()).to.equal(1);
    expect(bidsAcc.orderCount.toNumber()).to.equal(1);
    expect(bidsAcc.orders[0].orderId.toNumber()).to.equal(1);
    expect(bidsAcc.orders[0].baseAmount.toNumber()).to.equal(marketAcc.baseLotSize.toNumber() * 2);
    expect(bidsAcc.orders[0].quoteAmount.toNumber()).to.equal(1000);
    expect(bidsAcc.orders[0].orderAuthority.toBase58()).to.equal(keypair.publicKey.toBase58());

    console.log("Bid Order Signature is : ", sig);
  });

  it("Should match completely!", async() => {

    const start = Date.now();
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
    const now = Date.now();

    const timeTaken = now - start;
    console.log(`Time taken to Match order is ${timeTaken} ms`);
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

  it("Should be able to cancel an order !", async() => {

    const bidsAccount = await program.account.bookSide.fetch(bidAccount.publicKey);

    let orderId: anchor.BN;

    for (let i = 0; i < bidsAccount.orderCount.toNumber(); i++) {
      if (bidsAccount.orders[i].orderAuthority.toBase58() === keypair.publicKey.toBase58()) {
        orderId = bidsAccount.orders[i].orderId;
        break;
      }
    }

    console.log("Cancelling order with id", orderId.toNumber());

    const cancelOrderSig = await program.methods
    .cancelOrder({
      orderId,
      side: {bid:{}},
    })
    .accounts({
      user: keypair.publicKey.toBase58(),
      booksideAccount: bidAccount.publicKey.toBase58(),
      market: market.publicKey.toBase58(),

    })
    .rpc({commitment: "confirmed"});

    console.log("Cancel Order Signature is : ", cancelOrderSig);

  })


  it("Should be able to consume events", async() => {

    const balanceBefore = await program
    .account
    .userBalance
    .fetch(userBalanceAccount);

    console.log("Base Amount before consume events", balanceBefore.baseAmount.toNumber());
    console.log("Quote Amount before consume events", balanceBefore.quoteAmount.toNumber());

    const eventsBefore = await program.account.marketEvents.fetch(marketEvent.publicKey);

    const remainingAccounts : {
      pubkey:anchor.web3.PublicKey,
      isSigner: boolean,
      isWritable: boolean
    }[] = [];

    for(let i = 0; i < eventsBefore.eventsToProcess.toNumber(); i++) {
      let event = eventsBefore.events[i];
      const balanceAccount = getBalanceAccount(event.maker);
      remainingAccounts.push({
        pubkey: balanceAccount,
        isSigner: false,
        isWritable: true
      });
    }

    const consumeEventsSig = await program.methods
    .consumeEvents()
    .accounts({
      market: market.publicKey.toBase58(),
      consumeEventsAuthority: keypair.publicKey.toBase58(),
      marketEvents: marketEvent.publicKey.toBase58(),
    })
    .remainingAccounts(remainingAccounts)
    .rpc({commitment: "confirmed"});

    console.log("Consume Events Signature is : ", consumeEventsSig);

    const balanceAfter = await program
    .account
    .userBalance
    .fetch(userBalanceAccount);

    console.log("Base Amount after consume events", balanceAfter.baseAmount.toNumber());
    console.log("Quote Amount after consume events", balanceAfter.quoteAmount.toNumber());

  })

  it("Should be able to settle user balance !", async() => {
  
    const beforeBaseAccount = await getAccount(connection, userBaseTokenAccount, undefined, TOKEN_2022_PROGRAM_ID);
    const beforeQuoteAccount = await getAccount(connection, userQuoteTokenAccount, undefined, TOKEN_2022_PROGRAM_ID);

    console.log("Before Base Account Balance", Number(beforeBaseAccount.amount));
    console.log("Before Quote Account Balance", Number(beforeQuoteAccount.amount));

    const settleBalanceSig = await program.methods
    .settleUserBalance()
    .accounts({
      market: market.publicKey.toBase58(),
      userBalanceAccount: userBalanceAccount.toBase58(),
      userBaseTokenAccount: userBaseTokenAccount.toBase58(),
      userQuoteTokenAccount: userQuoteTokenAccount.toBase58(),
      tokenProgam: TOKEN_2022_PROGRAM_ID,
    })
    .rpc({commitment: "confirmed"});

    console.log("Settle Balance Signature is : ", settleBalanceSig);

    const afterBaseAccount = await getAccount(connection, userBaseTokenAccount, undefined, TOKEN_2022_PROGRAM_ID);
    const afterQuoteAccount = await getAccount(connection, userQuoteTokenAccount, undefined, TOKEN_2022_PROGRAM_ID);

    console.log("After Base Account Balance", Number(afterBaseAccount.amount));
    console.log("After Quote Account Balance", Number(afterQuoteAccount.amount));
  })
});

