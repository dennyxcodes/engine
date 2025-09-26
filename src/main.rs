use std::{collections::{BTreeMap, HashMap, VecDeque}, fmt::{self, write, Display, Formatter}};

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
    pub price: u64,
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

// Order Book Data Struct 

#[derive(Debug)]
pub struct SymbolBook {
    bids: BTreeMap<u64, VecDeque<Order>>,
    asks: BTreeMap<u64, VecDeque<Order>>,
    order_lookup: HashMap<u64, (Side, u64)>,
}

impl SymbolBook {
    pub fn new() -> Self {
        SymbolBook { 
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_lookup: HashMap::new() 
        }
    }

    fn add_resting_order(&mut self, order: Order) {
        let price = order.price;
        let side = order.side.clone(); // Clone side to avoid partial move
        let order_id = order.order_id;
        
        let book = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };


        let entry = book.entry(price).or_insert_with(VecDeque::new);
        entry.push_back(order);
        self.order_lookup.insert(order_id, (side, price));
    }

    pub fn cancel_order(&mut self, order_id: u64) -> bool {
        if let Some((side, price)) = self.order_lookup.remove(&order_id) {
            let book = match side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };

            if let Some(price_level) = book.get_mut(&price) {
                let initial_len = price_level.len();
                price_level.retain(|order| order.order_id != order_id);
                let is_empty = price_level.is_empty();
                let removed = price_level.len() < initial_len;
                if is_empty {
                    book.remove(&price);
                }

                return removed;
            }
        }
        false
    }

    pub fn process_order(&mut self, mut incoming_order: Order) -> Vec<Trade> {
        let mut trades: Vec<Trade> = Vec::new();

        // Determine which side of the book to match against
        let (incoming_side, target_book) = match incoming_order.side {
            Side::Buy => (Side::Buy, &mut self.asks),
            Side::Sell => (Side::Sell, &mut self.bids),
        };

        while incoming_order.quantity > 0 {
            let best_price_entry = match incoming_side {
                Side::Buy => target_book.keys().next().cloned(),
                Side::Sell => target_book.keys().rev().next().cloned(),
            };

            // If no orders on the target side, break the loop
            let best_price = match best_price_entry {
                Some(p) => p,
                None => break,
            };

            // Check for match condition (Price Crossover)
            let match_found = match incoming_side {
                Side::Buy => incoming_order.price >= best_price,
                Side::Sell => incoming_order.price <= best_price,
            };

            if !match_found {
                break;
            }

            let target_level = target_book.get_mut(&best_price).unwrap();
            let mut resting_order = target_level.pop_front().unwrap();
            let fill_quantity = incoming_order.quantity.min(resting_order.quantity);
            let execution_price = resting_order.price;

            // Create the Trade record
            let (buy_id, sell_id) = match incoming_side {
                Side::Buy => (incoming_order.order_id, resting_order.order_id),
                Side::Sell => (resting_order.order_id, incoming_order.order_id),
            };

            trades.push(Trade {
                buy_order_id: buy_id,
                sell_order_id: sell_id,
                symbol: incoming_order.symbol.clone(),
                price: execution_price,
                quantity: fill_quantity,
            });

            // --- Update quantities and book state ---

            incoming_order.quantity -= fill_quantity;
            resting_order.quantity -= fill_quantity;

            // 1. Handle resting order fill
            if resting_order.quantity > 0 {
                target_level.push_front(resting_order);
            } else {
                self.order_lookup.remove(&resting_order.order_id);
            }

            // 2. Cleanup price level if empty
            if target_level.is_empty() {
                target_book.remove(&best_price);
            }

            // 3. Check if incoming order is fully filled
            if incoming_order.quantity == 0 {
                break;
            }
        }
        trades
    }

}


fn main() {
    print!("Hello, world!");
}