use anchor_lang::prelude::*;
use anchor_spl::{associated_token::get_associated_token_address, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{errors::ClobbyProgramError, state::{BookSide, BookSideOrder, EventParams, EventType, Market, MarketEvents, Side, UserBalance}};

const MAX_ORDERS_TO_MATCH:usize = 5;

struct EditOrders {
    pub order_id : u64,
    pub base_amount_to_set: u64,
    pub total_quote_amount: u64
}

pub fn place_order(ctx:Context<PlaceOrder>, args:PlaceOrderArgs) -> Result<()> {

    let accounts = ctx.accounts;

    let market = &mut accounts.market;
    let mut market_events = accounts.market_events.load_mut()?;
    let mut asks = accounts.asks.load_mut()?;
    let mut bids = accounts.bids.load_mut()?;

    let mut orders_to_delete : Vec<(u64, u64)> = Vec::new();
    let mut orders_to_edit: Vec<EditOrders> = Vec::new();
    let mut orders_matched = 0_usize;

    let base_amount:u64 = u64::from(args.base_lots) * market.base_lot_size;

    let mut remaining_order_amount = base_amount;
    
    let mut limit = MAX_ORDERS_TO_MATCH;

    let opposing_side:&mut BookSide;
    let taker_side:&mut BookSide;
    
    let transfer_token_amount: u64;

    match args.side {
        Side::Bid =>{

            // on the bidding side, quote token is used to trade
            require_keys_eq!(accounts.token_to_trade.key(), market.quote_token);
            require!(accounts.user_token_account.amount >= args.quote_amount, ClobbyProgramError::InSufficientBalance);


            transfer_token_amount = args.quote_amount;
            taker_side = &mut bids;
            opposing_side = &mut asks;

        },
        Side::Ask =>{

            // on the asking side, base token is used to trade
            require_keys_eq!(accounts.token_to_trade.key(), market.base_token);
            require!(accounts.user_token_account.amount >= base_amount, ClobbyProgramError::InSufficientBalance);

            transfer_token_amount = base_amount;
            taker_side = &mut asks;
            opposing_side = &mut bids;
        }
    }

    let expected_token_vault = get_associated_token_address(
        &market.market_authority,
        &accounts.token_to_trade.key()
    );

    require_keys_eq!(expected_token_vault.key(), accounts.token_vault.key());

    for opposing_order in opposing_side.orders.iter(){

        if limit == 0 {
            msg!("Max order limit reached!");
            break;
        }

        if remaining_order_amount == 0 {
            msg!("remaining order amount becomes zero");
            break;
        }

        if opposing_order.quote_amount > args.quote_amount {
            msg!("Opposing Quote Amount becomes higher !");
            break;
        }

        let base_amount_eaten = opposing_order.base_amount.min(remaining_order_amount);

        // quote amount used to buy or sell one base lot
        let quote_amount_at = opposing_order.quote_amount.min(args.quote_amount);

        let eaten_base_lots = base_amount_eaten / market.base_lot_size;
        let total_quote_amount = quote_amount_at * eaten_base_lots;

        if base_amount_eaten == opposing_order.base_amount {
            orders_to_delete.push((opposing_order.order_id, total_quote_amount));
        }
        else{
            let base_amount_to_set = opposing_order.base_amount - base_amount_eaten;
            orders_to_edit.push(EditOrders { order_id: opposing_order.order_id, base_amount_to_set,  total_quote_amount});
        }

        remaining_order_amount -= base_amount_eaten;
        limit -= 1;
        orders_matched += 1;
    }

    // if the order is IOC and remaing amount > 0, through partially filled error 1
    if args.ioc && remaining_order_amount > 0 {
        return err!(ClobbyProgramError::OrderFilledPartially);
    }

    // +1 for taker to sit on the orderbook
    if !market_events.can_add_event(orders_matched + 1) {
        return err!(ClobbyProgramError::EventsMaxLimit);
    }

    let event_type_ops_side = opposing_side.get_side_in_enum()?;

    // apply the changes to the opposing_order acccounts
    // check if the remaining_order_amount > 0 then add it in orderbook
    for (index, (order_id, total_quote_amount)) in orders_to_delete.iter().enumerate() {
        let mut matched_order = opposing_side.orders[index];

        // to match the makers
        market_events.add_event(EventParams{
            order_id: matched_order.order_id,
            maker: matched_order.order_authority,
            base_amount: matched_order.base_amount,
            quote_amount: *total_quote_amount,
            event_type: EventType::Fill,
            side: event_type_ops_side.clone() ,
        });

        // reset the order
        if matched_order.order_id == *order_id {
            matched_order.base_amount = 0;
            matched_order.order_id = 0; // this is important
            matched_order.quote_amount = 0;
            matched_order.order_authority = market.key();
        }
    }

    let full_orders_matched = orders_to_delete.len();

    for (index, order) in orders_to_edit.iter().enumerate() {
        let order_idx = full_orders_matched + index;
        let mut partial_matched_order = opposing_side.orders[order_idx];

        if partial_matched_order.order_id == order.order_id {
            partial_matched_order.base_amount = order.base_amount_to_set;

            market_events.add_event(EventParams{
                base_amount: order.base_amount_to_set,
                order_id: partial_matched_order.order_id,
                maker: partial_matched_order.order_authority,
                quote_amount: order.total_quote_amount,
                event_type: EventType::Fill,
                side: event_type_ops_side,
            });
        }
    }

    /*
        Consider something like this: (we want to bring the valid base_amounts at first!)

        Order{order_id: 0, base_amount:0, quote_amount: 0},
        Order{order_id: 0, base_amount:0, quote_amount: 0},
        Order{order_id: 3, base_amount:1000, quote_amount: 1000},
        Order{order_id: 4, base_amount:2000, quote_amount: 2000},
        Order{order_id: 0, base_amount:0, quote_amount: 0},
    */

    // now reposition the array

    for i in full_orders_matched as u64..opposing_side.order_count {
        let j:u64 = i - full_orders_matched as u64;
        opposing_side.orders.swap(i as usize, j as usize);
    }

    opposing_side.order_count -= orders_matched as u64;

    if remaining_order_amount == 0 {
        msg!("successfully executed orders without sitting on orderbook !");
    }
    else{

        // if the book is full remove the last order
        if orders_matched > 0 && taker_side.order_count == taker_side.orders.len() as u64{

            let index = (taker_side.order_count - 1) as usize;

            let removed_order = taker_side.orders[index];

            // record the removed order in the market event, as we need to pay them back 

            market_events.add_event(EventParams { 
                order_id: removed_order.order_id, 
                maker: removed_order.order_authority, 
                side: args.side, 
                event_type: EventType::Out, 
                base_amount: removed_order.base_amount, 
                quote_amount: removed_order.quote_amount, 
            });

            taker_side.orders[index] = BookSideOrder{
                base_amount:0,
                order_authority:market.key(),
                order_id: 0,
                quote_amount: 0,
            };

            taker_side.order_count-=1;
        }

        market.total_orders += 1;

        let order_id = market.total_orders;

        let index = taker_side.order_count as usize;

        taker_side.orders[index] = BookSideOrder {
            base_amount: remaining_order_amount,
            quote_amount: args.quote_amount,
            order_id,
            order_authority: *accounts.user.key
        };

        taker_side.order_count += 1;
        
        // sort it base on the side, once the order is added
        taker_side.sort_orders_till_idx(index)?;

        msg!("Successfully executed the orders! and placed the remaining or orderbook");

    }

    // finally transfer the token from the user token account to market vault account

    let decimals = accounts.token_to_trade.decimals;

    let cpi_accounts = TransferChecked {
        mint: accounts.token_to_trade.to_account_info(),
        from: accounts.user_token_account.to_account_info(),
        to: accounts.token_vault.to_account_info(),
        authority: accounts.user.to_account_info(),
    };

    let cpi_program = accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
    transfer_checked(cpi_context, transfer_token_amount, decimals)?;
    
    Ok(())

}

#[derive(Accounts)]
#[instruction(args: PlaceOrderArgs)]
pub struct PlaceOrder<'info>{

    #[account(
        mut,
        signer,
    )]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserBalance::INIT_SPACE,
        seeds = [b"balance", user.key().as_ref()],
        bump,
    )]
    pub user_balance_account: Account<'info, UserBalance>,

    #[account(
        has_one = bids, 
        has_one = asks,
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

    #[account(mut)]
    pub market_events: AccountLoader<'info, MarketEvents>,

    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,

    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,   

    #[account(
        mut,
        token::authority = user,
        token::mint = token_to_trade,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::authority = market_authority,
    )]
    pub token_vault: InterfaceAccount<'info, TokenAccount>,

    pub token_to_trade: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}


#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct PlaceOrderArgs {
    pub side: Side,
    pub base_lots: u16, // Number of base lots to buy or sell
    pub quote_amount: u64,
    pub ioc: bool, // ImmediateOrCancel 
}