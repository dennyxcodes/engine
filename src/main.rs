use std::fmt::{self, write, Display, Formatter};

// For market order
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Side {
    Buy, // Bid
    Sell, // Ask
}

impl Display for Side {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Single Order placing
#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: u64,
    pub symbol: String,
    pub side: Side,
    pub price: f64,
    pub quantity: u64,
    pub timestamp: u64,
}

impl Display for Order {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "ID: {}, {} {} @ {} (Qty: {}) | TS: {}",
            self.order_id,
            self.side,
            self.symbol,
            self.price,
            self.quantity,
            self.timestamp
        )
    }
    
}

#[derive(Debug)]
pub struct Trade {
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub symbol: String,
    pub price: u64,
    pub quantity: u64,
}

impl Display for Trade {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} | Executed {} @ {} | Buy ID: {}, Sell ID: {}",
            self.symbol, self.quantity,
            self.price, self.buy_order_id,
            self.sell_order_id
        )
    }
}

fn main() {
    print!("Hello, world!");
}