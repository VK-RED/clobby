use anchor_lang::prelude::*;
use anchor_spl::{associated_token::get_associated_token_address, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{errors::ClobbyProgramError, state::{BookSide, BookSideOrder, Market, Side}};

const MAX_ORDERS_TO_MATCH:usize = 10;

struct EditOrders {
    pub order_id : u64,
    pub amount_to_set: u64,
}

pub fn place_order(ctx:Context<PlaceOrder>, args:PlaceOrderArgs) -> Result<()> {

    let accounts = ctx.accounts;

    match args.side {

        Side::Bid => {
             // on the bid side, quote token is used to trade

            require_keys_eq!(accounts.token_to_trade.key(), accounts.market.quote_token);

            require_keys_eq!(accounts.user_token_account.mint,accounts.market.quote_token);

            let expected_token = get_associated_token_address(
                &accounts.market.market_authority, 
                &accounts.token_to_trade.key()
            );

            require_keys_eq!(accounts.vault_token_account.key(), expected_token);
            
            require!(accounts.user_token_account.amount >= args.quote_amount, ClobbyProgramError::InSufficientBalance);

            let mut asks = accounts.asks.load_mut()?;
            let mut bids = accounts.bids.load_mut()?;

            let mut limit = MAX_ORDERS_TO_MATCH;

            let mut orders_to_delete : Vec<u64> = Vec::new();
            let mut orders_to_edit: Vec<EditOrders> = Vec::new();
            let mut remaining_order_amount = args.base_amount;

            let mut orders_matched = 0_usize;

            for opposing_order in asks.orders.iter() {

                if limit == 0 {
                    msg!("Max order limit reached!");
                    break;
                }

                if opposing_order.base_amount < remaining_order_amount {
                    msg!("opposing order becomes greater : {}", opposing_order.base_amount);
                    break;
                }

                let amount_eaten = opposing_order.base_amount.min(remaining_order_amount);

                if amount_eaten == opposing_order.base_amount {
                    orders_to_delete.push(opposing_order.order_id);
                }
                else{
                    orders_to_edit.push(EditOrders { order_id: opposing_order.order_id, amount_to_set: amount_eaten });
                }

                remaining_order_amount -= amount_eaten;
                limit -= 1;
                orders_matched += 1;
            }   

            // if the order is IOC and remaing amount > 0, through partially filled error 1




            // apply the changes to the opposing_order acccounts
            // check if the remaining_order_amount > 0 then add it in orderbook
            for (index, order_id) in orders_to_delete.iter().enumerate() {
                let mut matched_order = asks.orders[index];
                // reset the order
                if matched_order.order_id == *order_id {
                    matched_order.base_amount = 0;
                    matched_order.order_id = 0; // this is important
                    matched_order.quote_amount = 0;
                    matched_order.order_authority = *ctx.program_id;
                }
            }

            let full_orders_matched = orders_to_delete.len();

            for (index, order) in orders_to_edit.iter().enumerate() {
                let order_idx = full_orders_matched + index;
                let mut partial_matched_order = asks.orders[order_idx];

                if partial_matched_order.order_id == order.order_id {
                    partial_matched_order.base_amount = order.amount_to_set;
                }
            }

            // now reposition the array once the matching is done
            // this will push the fully matched orders to the right
            // and orders with base_amount > 0 in asc_order at the front 

            for i in full_orders_matched as u64..asks.order_count {
                let j:u64 = i - full_orders_matched as u64;
                asks.orders.swap(i as usize, j as usize);
            }

            asks.order_count -= orders_matched as u64;

            // now post this in the bids 

            // if the book is full remove the last order
            if bids.order_count == bids.orders.len() as u64{
                let index = bids.order_count - 1;

                bids.orders[index as usize] = BookSideOrder{
                    base_amount:0,
                    order_authority:*ctx.program_id,
                    order_id: 0,
                    quote_amount: 0,
                };

                bids.order_count-=1;
            }

            // Figure out the order_id later
            let order_id = bids.total_order_count + 1;

            let index = bids.order_count as usize;
            bids.orders[index] = BookSideOrder {
                base_amount: remaining_order_amount,
                quote_amount: args.quote_amount,
                order_id,
                order_authority: *accounts.user.key
            };

            bids.order_count += 1;
            bids.total_order_count += 1;
            
            let orders_to_sort = &mut bids.orders[..=index];

            orders_to_sort.sort_by(|a, b| b.base_amount.cmp(&a.base_amount));

            // TODO:
            // increment quote token accrued for the makers

            // have to increment the quote token and decrement the base token for 

        },
        Side::Ask => {

            // on the ask side, base token is used to trase

            require_keys_eq!(accounts.token_to_trade.key(), accounts.market.base_token);

            require_keys_eq!(accounts.user_token_account.mint,accounts.market.base_token);

            let expected_token = get_associated_token_address(
                &accounts.market.market_authority, 
                &accounts.token_to_trade.key()
            );

            require_keys_eq!(accounts.vault_token_account.key(), expected_token);

            require!(accounts.user_token_account.amount >= args.base_amount, ClobbyProgramError::InSufficientBalance);
        }
    }


    // finally transfer the token from the user token account to market vault account

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
        has_one = bids, 
        has_one = asks,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,

    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,   

    #[account(
        mut,
        token::authority = user,
        token::mint = token_to_trade
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_to_trade: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,

}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct PlaceOrderArgs {
    pub side: Side,
    pub base_amount: u64,
    pub quote_amount: u64,
}