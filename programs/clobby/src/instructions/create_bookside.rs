use anchor_lang::prelude::*;

use crate::state::{BookSide, Market};

pub fn create_book_side(ctx:Context<CreateBookSide>) -> Result<()> {

    let accounts = ctx.accounts;

    let mut bids = accounts.bids.load_init()?;
    let mut asks = accounts.asks.load_init()?;

    bids.side = 0;
    bids.market_account = accounts.market.key();

    asks.side = 1;
    asks.market_account = accounts.market.key();

    Ok(())
}

#[derive(Accounts)]
pub struct CreateBookSide<'info>{
    #[account(
        zero,
        constraint = bids.key() == market.bids.key(),
    )]
    bids: AccountLoader<'info, BookSide>,

    #[account(
        zero,
        constraint = asks.key() == market.asks.key(),
    )]
    asks: AccountLoader<'info, BookSide>,

    market: Account<'info, Market>,
    
}