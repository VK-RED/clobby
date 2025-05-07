use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};
use crate::state::Market;


/// Initialize the market account as well bids and asks
pub fn create_market(ctx: Context<CreateMarket>, args:CreateMarketArgs) -> Result<()> {

    require_gt!(args.min_base_order_amount, 0);
    require_gt!(args.min_quote_order_amount, 0);

    let accounts = ctx.accounts;

    let market = &mut accounts.market;
    market.asks = accounts.asks.key();
    market.bids = accounts.bids.key();
    market.base_token = accounts.base_token.key();
    market.quote_token = accounts.quote_token.key();
    market.base_token_vault = accounts.base_token_vault.key();
    market.quote_token_vault = accounts.quote_token_vault.key();
    market.name = args.name;
    market.min_base_amount = args.min_base_order_amount;
    market.min_quote_amount = args.min_quote_order_amount;
    market.market_authority_bump = ctx.bumps.market_authority;

    msg!("Market Account has been created Successfully!");

    Ok(())
}

#[derive(Accounts)]
pub struct CreateMarket<'info> {
    #[account(
        mut,
        signer,
    )]
    signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + Market::INIT_SPACE,
    )]
    market: Box<Account<'info, Market>>,

    ///CHECK: THIS IS PDA OF THE MARKET, THAT CAN SEND 
    /// AND RECEIVE TOKENS ON BEHALF OF MARKET
    #[account(
        seeds = [b"market", market.key().as_ref()],
        bump
    )]
    market_authority: UncheckedAccount<'info>,

    /// CHECK: The bids acccount will be created in create_bookside instruction
    bids: UncheckedAccount<'info>,
    /// CHECK: The asks acccount will be created in create_bookside instruction
    asks: UncheckedAccount<'info>,

    #[account(
        constraint = base_token.key() != quote_token.key() 
    )]
    base_token: Box<InterfaceAccount<'info, Mint>>,
    quote_token: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = base_token,
        associated_token::authority = market_authority,
        associated_token::token_program = token_program,
    )]
    base_token_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer=signer,
        associated_token::mint = quote_token,
        associated_token::authority = market_authority,
        associated_token::token_program = token_program,
    )]
    quote_token_vault: InterfaceAccount<'info, TokenAccount>,

    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
    associated_token_program: Program<'info, AssociatedToken>
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateMarketArgs{
    pub name: String, 
    pub min_base_order_amount: u128,
    pub min_quote_order_amount: u128,
}