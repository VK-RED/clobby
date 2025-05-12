use anchor_lang::prelude::*;

use crate::{errors::ClobbyProgramError, state::{BookSide, EventParams, Market, MarketEvents, Side}};
use crate::state::EventType;

pub fn cancel_order(ctx:Context<CancelOrder>, args: CancelOrderArgs) -> Result<()>{
    
    let accounts= ctx.accounts;

    let bookside_account = &accounts.bookside_account;
    let mut bookside = bookside_account.load_mut()?;
    let mut market_event = accounts.market_events.load_mut()?;

    let expected_bookside : Pubkey;

    match args.side {
        Side::Bid => {
            expected_bookside = accounts.market.bids.key();
        },
        Side::Ask => {
            expected_bookside = accounts.market.asks.key();
        }
    }

    require_keys_eq!(expected_bookside, bookside_account.key());

    let (target_index, target_order) = bookside.orders
    .iter_mut()
    .enumerate()
    .find(|(_index, order)| order.order_id == args.market_id)
    .ok_or(ClobbyProgramError::InvalidOrderId)?;

    // Check only the order_authority can cancel !
    require_keys_eq!(target_order.order_authority, accounts.user.key());

    let can_add_event = market_event.can_add_event(1);

    if !can_add_event {
        return err!(ClobbyProgramError::EventsMaxLimit);
    }

    market_event.add_event(
        EventParams {
            base_amount: target_order.base_amount,
            order_id: target_order.order_id,
            maker: target_order.order_authority,
            quote_amount: target_order.quote_amount,
            side: args.side,
            event_type: EventType::Out,
        }
    );

    // reset the target order
    target_order.base_amount = 0;
    target_order.quote_amount = 0;
    target_order.order_id = 0;
    target_order.order_authority = accounts.market.key();

    /*
        Consider the bookside like
        [
            {id: 4, base_Amount: 3000, quote_Amount: 3000},
            {id: 0, base_Amount: 0, quote_Amount: 0},
            {id: 6, base_Amount: 5000, quote_Amount: 5000},
            {id: 7, base_Amount: 6000, quote_Amount: 6000}
        ]
        in this case, target_index = 1, (i starts from 2 and j starts from 1)
        we want to bring the id:0 to the last as it becomes invalid !!
    */

    for i in target_index+1..bookside.order_count as usize {
        let j = i-1;
        bookside.orders.swap(i, j);
    } 

    bookside.order_count -= 1;

    Ok(())
}

#[derive(Accounts)]
pub struct CancelOrder<'info>{

    #[account(
        mut,
        signer,
    )]
    pub user: Signer<'info>,

    #[account(
        has_one = market_events,
        has_one = market_authority,
    )]
    pub market: Box<Account<'info, Market>>,

    ///CHECK: THIS IS PDA OF THE MARKET, THAT CAN SEND 
    /// AND RECEIVE TOKENS ON BEHALF OF MARKET
    #[account(
        seeds = [b"market", market.key().as_ref()],
        bump
    )]
    pub market_authority: UncheckedAccount<'info>,

    #[account(
        mut
    )]
    pub market_events: AccountLoader<'info, MarketEvents>,

    #[account(mut)]
    pub bookside_account: AccountLoader<'info, BookSide>,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CancelOrderArgs{
    pub market_id: u64,
    pub side: Side,
}