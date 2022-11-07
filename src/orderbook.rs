use serde::Serialize;

use crate::server::order_book::{Summary, Level};
#[derive(Clone, Serialize)]
pub struct OrderBook{
    pub spread: f64, 
    pub asks: Vec<OrderLine>,
    pub bids: Vec<OrderLine>,
}

impl OrderBook{
    pub fn toSummary(self) -> Summary{
        let mut a = Vec::new();
        let mut b = Vec::new();
        for line in self.asks {
            a.push(line.toLevel());
        } 
        for line in self.bids {
            a.push(line.toLevel());
        } 
        Summary { 
            spread: self.spread, 
            bids: b, 
            asks: a 
        }
    }
}

#[derive(Clone, Serialize)]
pub struct OrderLine{
    pub exchange: String,
    pub price: f64,
    pub amount: f64,
}

impl OrderLine{
    pub fn smaller(&self, x: &OrderLine) -> bool{
        if self.amount == 0.0 {
            return true;
        }
        if self.price < x.price{
            return true;
        }
        if self.price > x.price {
            return false;
        }
        if self.amount < x.amount{
            return true;
        }
        return false;
    }

    pub fn greater(&self, x: &OrderLine) -> bool{
        if self.amount == 0.0 {
            return true;
        }
        if self.price > x.price{
            return true;
        }
        if self.price < x.price {
            return false;
        }
        if self.amount < x.amount{
            return true;
        }
        return false;
    }

    pub fn toLevel(self) -> Level{
        Level{
            exchange: self.exchange,
            price: self.price,
            amount: self.amount,
        }
    }
}