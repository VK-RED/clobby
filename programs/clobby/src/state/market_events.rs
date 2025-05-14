use anchor_lang::prelude::*;

use crate::errors::ClobbyProgramError;

use super::Side;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, InitSpace)]
pub enum EventType{
    Fill,
    Out,
}

#[zero_copy]
pub struct Event{
    pub id: u64,
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
    pub total_events_count: u64,
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

impl Event {
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

    pub fn get_event_in_enum(&self) -> Result<EventType> {
        match self.event_type {
            0 => {
                Ok(EventType::Fill)
            },
            1 => {
                Ok(EventType::Out)
            }
            _ => {
                err!(ClobbyProgramError::InvalidEventType)
            }
        }
    }

    pub fn remove(&mut self, market:Pubkey){
        self.base_amount =  0;
        self.quote_amount = 0;
        self.maker = market;
        self.order_id = 0;
        self.event_type = 0;
        self.side = 0;
        self.id = 0;
    }
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

        let event_id = self.total_events_count+1;

        self.events[index] = Event{
            base_amount: event.base_amount,
            quote_amount: event.quote_amount,
            maker: event.maker,
            order_id: event_id,
            id: self.total_events_count+1,
            event_type,
            side: order_side,
        };

        self.events_to_process+=1;
        self.total_events_count+=1;
    }

    pub fn can_add_event(&self, events_to_add:usize) -> bool{
        // we add 6 as an extra buffer
        events_to_add + 6 < self.events.len()

    }

}