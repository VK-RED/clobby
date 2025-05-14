use anchor_lang::prelude::*;

use crate::state::{Market, MarketEvents, UserBalance, Side, EventType};

const MAX_EVENTS_TO_CONSUME:usize = 7;

pub fn consume_events(ctx:Context<ConsumeEvents>) -> Result<()> {
    let accounts = ctx.accounts;
    // makers balance account should be passed here
    let remaining_accounts = ctx.remaining_accounts;
    let mut market_events = accounts.market_events.load_mut()?;

    let mut consumed_count: usize = 0;

    for event in market_events.events.iter_mut(){

        if event.id == 0 && event.order_id == 0 {
            msg!("no events left to consume!");
            break;
        }

        if consumed_count == MAX_EVENTS_TO_CONSUME {
            msg!("maximum limit reached");
            break;
        }

        let maker = event.maker;

        let (balance_account, _bump) = Pubkey::find_program_address(
            &[b"balance", maker.as_ref()], 
            ctx.program_id,
        );

        let maker_balance_acc = remaining_accounts
        .iter()
        .find(|account| *account.key == balance_account);

        let mut consumed = false;

        // else case will not occur most probably
        if let Some(account_info) = maker_balance_acc {
            let mut maker_balance_account = UserBalance::try_from_slice(&mut account_info.data.borrow_mut())?;

            match event.get_event_in_enum()? {
                EventType::Fill => {
                    match event.get_side_in_enum()? {
                        Side::Bid => {
                            maker_balance_account.base_amount += event.base_amount;
                        },
                        Side::Ask => {
                            maker_balance_account.quote_amount += event.quote_amount;
                        }
                    }
                },
                EventType::Out => {
                    match event.get_side_in_enum()? {
                        Side::Bid => {
                            maker_balance_account.quote_amount += event.quote_amount;
                        },
                        Side::Ask => {
                            maker_balance_account.base_amount += event.base_amount;
                        }
                    }
                }
            }
            consumed = true;
            consumed_count+=1;
        }

        if consumed {
            event.remove(accounts.market.key());
        }   
        
    }

    if consumed_count > 0 {
        for i in consumed_count..market_events.events_to_process as usize{
            let j = i - consumed_count;
            market_events.events.swap(i, j);
        }
    
        market_events.events_to_process -= consumed_count as u64;
    }
    
    msg!("successfully consumed {} events", consumed_count);

    Ok(())
}


#[derive(Accounts)]
pub struct ConsumeEvents<'info>{

    #[account(
        mut,
        signer,
        constraint = market.consume_events_authority == consume_events_authority.key(),
    )]
    pub consume_events_authority: Signer<'info>,

    pub market: Account<'info, Market>,

    #[account(
        constraint = market.market_authority.key() == market_authority.key(),
        seeds = [b"market", market.key().as_ref()],
        bump
    )]
    /// CHECK: THIS IS PDA OF THE MARKET, THAT CAN SEND TOKENS
    pub market_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = market.market_events.key() == market_events.key(),
    )]
    pub market_events: AccountLoader<'info, MarketEvents>,
}