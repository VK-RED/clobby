use anchor_lang::prelude::*;

use super::Side;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, InitSpace)]
pub enum EventType{
    Fill,
    Out,
}

#[zero_copy]
pub struct Event{
    pub order_id: u64,
    pub base_amount: u64,
    pub quote_amount: u64,
    pub maker: Pubkey,
    pub side: u64, // 0 -> Bid, 1 -> Ask
    pub event_type : u64, // 0 -> Fill, 1 -> Out
}

#[account(zero_copy)]
pub struct MarketEvents{
    pub market: Pubkey,
    pub events_to_process: u64,
    pub events: [Event; 512],
}

pub struct EventParams{
    pub order_id: u64,
    pub maker: Pubkey,
    pub side: Side,
    pub event_type: EventType,
    pub base_amount: u64,
    pub quote_amount: u64
}

impl MarketEvents {
    
    pub fn add_event(&mut self, event:EventParams) {
        
        let index = self.events_to_process as usize;

        let event_type :u64 = match event.event_type {
            EventType::Fill => 0,
            EventType::Out => 1,
        };

        let order_side: u64 = match event.side {
            Side::Bid => 0,
            Side::Ask => 1,
        };

        self.events[index] = Event{
            base_amount: event.base_amount,
            quote_amount: event.quote_amount,
            maker: event.maker,
            order_id: event.order_id,
            event_type,
            side: order_side,
        };

        self.events_to_process+=1;
    }

    pub fn can_add_event(&self, events_to_add:usize) -> bool{
        // we add 6 as an extra buffer
        events_to_add + 6 < self.events.len()

    }

    // TODO: FIX
    pub fn remove_event(&mut self, index:usize){

        self.events[index] = Event{
            base_amount: 0,
            quote_amount: 0,
            maker: self.market,
            order_id: 0,
            event_type:0,
            side: 0,
        };
    }
}