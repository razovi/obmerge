use serde::Serialize;

use crate::proto::order_book::{Summary, Level};
#[derive(Clone, Serialize)]
pub struct OrderBook{
    pub spread: f64, 
    pub asks: Vec<OrderLine>,
    pub bids: Vec<OrderLine>,
}

impl OrderBook{
    pub fn to_summary(self) -> Summary{
        let mut a = Vec::new();
        let mut b = Vec::new();
        for line in self.asks {
            a.push(line.to_level());
        } 
        for line in self.bids {
            b.push(line.to_level());
        } 
        Summary { 
            spread: self.spread, 
            bids: b, 
            asks: a,
        }
    }
    pub fn from_summary(x: Summary) -> Self{
        let mut a = Vec::new();
        let mut b = Vec::new();
        for line in x.asks {
            a.push(OrderLine::from_level(line));
        } 
        for line in x.bids {
            b.push(OrderLine::from_level(line));
        } 
        OrderBook { 
            spread: x.spread, 
            bids: b, 
            asks: a,
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

    pub fn to_level(self) -> Level{
        Level{
            exchange: self.exchange,
            price: self.price,
            amount: self.amount,
        }
    }

    pub fn from_level(x: Level) -> Self{
        OrderLine{
            exchange: x.exchange,
            price: x.price,
            amount: x.amount,
        }
    }
}