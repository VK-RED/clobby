use anchor_lang::prelude::*;

#[zero_copy]
pub struct BookSideOrder{
    pub order_id: u64,
    pub base_amount: u64,
    pub quote_amount: u64,
    pub order_authority: Pubkey,
}

#[account(zero_copy)]
pub struct BookSide {
    pub side: u64,  // 0 => Bid, Ask => 1, Ideally this should be an enum 
    pub market_account: Pubkey,
    pub orders: [BookSideOrder;1024]
}