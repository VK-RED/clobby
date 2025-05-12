use anchor_lang::prelude::*;

use crate::errors::ClobbyProgramError;

#[derive(AnchorDeserialize, AnchorSerialize, PartialEq, Eq, Clone, InitSpace, Copy)]
pub enum Side{
    Bid, 
    Ask
}

#[zero_copy]
#[derive(
    PartialEq, Eq
)]
pub struct BookSideOrder{
    pub order_id: u64,
    pub base_amount: u64,
    pub quote_amount: u64,
    pub order_authority: Pubkey,
}

#[account(zero_copy)]
pub struct BookSide {
    pub side: u64,  // 0 => Bid, Ask => 1, Ideally this should be an enum ,
    pub order_count: u64,
    pub market_account: Pubkey,
    pub orders: [BookSideOrder;1024]
}

impl BookSide{

    pub fn get_side_in_enum(&self) -> Result<Side> {

        match self.side {
            0 => {
                Ok(Side::Bid)
            },
            1 => {
                Ok(Side::Ask)
            },
            _ => {
                err!(ClobbyProgramError::InvalidBookSide)
            }
        }
    }


    pub fn sort_orders_till_idx(&mut self, till_order_idx: usize) -> Result<()>{
        let side = self.get_side_in_enum()?;
        
        match side {
            Side::Bid => {
                self.orders[..=till_order_idx].sort_by(|a, b| b.quote_amount.cmp(&a.quote_amount));
            },
            Side::Ask => {
                self.orders[..=till_order_idx].sort_by(|a, b| a.quote_amount.cmp(&b.quote_amount));
            }
        }

        Ok(())
    }

    pub fn reposition_orders_after_match(&self){

    }
}