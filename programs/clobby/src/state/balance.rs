use anchor_lang::prelude::*;


#[derive(InitSpace)]
#[account]
/// This is specially useful when matching the orders
/// we can keep track of the tokens accrued
pub struct UserBalance{
    pub market: Pubkey,
    pub user: Pubkey,
    pub base_token: Pubkey,
    pub quote_token: Pubkey,
    pub base_amount: u64,
    pub quote_amount: u64,
}

pub enum ResetSide{
    Base,
    Quote
}

impl UserBalance{
    pub fn reset_balance(&mut self, side: ResetSide){
        match side {
            ResetSide::Base => {
                self.base_amount = 0;
            },
            ResetSide::Quote => {
                self.quote_amount = 0;
            }
        }
    }
    
}