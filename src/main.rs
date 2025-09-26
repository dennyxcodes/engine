use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::fmt::{self, Display, Formatter};
use lazy_static::lazy_static;

lazy_static! {
    static ref ORDER_ID_COUNTER: AtomicU64 = AtomicU64::new(1000);
    static ref TIMESTAMP_COUNTER: AtomicU64 = AtomicU64::new(1_700_000_000_000);
}

// gen a unique, increamenting order ID for demonstration purposes.
fn generate_order_id() -> u64 {
    ORDER_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

// gen a unique, increamenting timestamp (in milliseconds)
fn generate_timestamp() -> u64 {
    TIMESTAMP_COUNTER.fetch_add(10, Ordering::SeqCst)
}

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
        
        if incoming_order.quantity > 0 {
            self.add_resting_order(incoming_order);
        }
        
        trades
    }

    pub fn print_book(&self, symbol: &str) {
        println!("\n--- Order Book for {} ---", symbol);
        println!("\n--- ASKS (Lowest Price) ---");
        if self.asks.is_empty() {
            println!("(Empty)");
        } else {
            for (price, level) in self.asks.iter() {
                let total_qty: u64 = level.iter().map(|o| o.quantity).sum();
                let num_orders = level.len();
                println!("  [Price: {}] Total Qty: {} ({} orders)", price, total_qty, num_orders);
                for order in level.iter() {
                    println!("    -> {}", order);
                }
            }
        }

        println!("\n--- BIDS (Highest Price) ---");
        if self.bids.is_empty() {
            println!("(Empty)");
        } else {
            for (price, level) in self.bids.iter().rev() {
                let total_qty: u64 = level.iter().map(|o| o.quantity).sum();
                let num_orders = level.len();
                println!("  [Price: {}] Total Qty: {} ({} orders)", price, total_qty, num_orders);
                for order in level.iter() {
                    println!("    -> {}", order);
                }
            }
        }
        println!("---------------------------------\n");
    }

}

// Matching Engine Wrapper

pub struct MatchingEngine {
    books: HashMap<String, SymbolBook>,
    trades: Vec<Trade>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        MatchingEngine { 
            books: HashMap::new(),
            trades: Vec::new(),
        }
    }

    fn get_or_create_book(&mut self, symbol: &str) -> &mut SymbolBook {
        self.books.entry(symbol.to_string()).or_insert_with(SymbolBook::new)
    }

    pub fn add_order(&mut self, order: Order) {
        println!("\n--- New Incoming Order ---");
        println!("Processing: {}", order);

        let symbol = order.symbol.clone();
        let book = self.get_or_create_book(&symbol);
        
        let new_trades = book.process_order(order);

        if !new_trades.is_empty() {
            println!("--- Executed Trades ---");
            for trade in &new_trades {
                println!("  {}", trade);
            }
            self.trades.extend(new_trades);
        } else {
            println!("No immediate match found. Order resting in book.");
        }
    }

    pub fn cancel_order(&mut self, order_id: u64, symbol: &str) {
        println!("\n--- Attempting Cancellation (ID: {}) ---", order_id);
        if let Some(book) = self.books.get_mut(symbol) {
            if book.cancel_order(order_id) {
                println!("Cancellation successful for ID {}.", order_id);
            } else {
                println!("Order ID {} not found in book.", order_id);
            }
        } else {
            println!("Symbol {} not found.", symbol);
        }
    }

     pub fn print_book(&self, symbol: &str) {
        if let Some(book) = self.books.get(symbol) {
            book.print_book(symbol);
        } else {
            println!("Order book for {} is empty or does not exist.", symbol);
        }
    }

    pub fn print_all_trades(&self) {
        println!("\n--- All Executed Trades ---");
        if self.trades.is_empty() {
            println!("No trades executed yet.");
        } else {
            for trade in &self.trades {
                println!("{}", trade);
            }
        }
        println!("---------------------------\n");
    }
        
}

fn main() {
    env_logger::init();

    let symbol = "BTC-USD";
    let mut engine = MatchingEngine::new();

    println!("1. Establishing Initial BTC-USD Order Book");

    // Asks (Sell)
    engine.add_order(Order { 
        order_id: generate_order_id(),
        symbol: symbol.to_string(),
        side: Side::Sell,
        price: 50020, 
        quantity: 10, 
        timestamp: generate_timestamp() 
    });

    engine.add_order(Order { 
        order_id: generate_order_id(),
        symbol: symbol.to_string(),
        side: Side::Sell,
        price: 50050, 
        quantity: 5, 
        timestamp: generate_timestamp() 
    });

    engine.add_order(Order { 
        order_id: generate_order_id(),
        symbol: symbol.to_string(),
        side: Side::Sell,
        price: 50020, 
        quantity: 5, 
        timestamp: generate_timestamp() 
    });

    // Bids (Buy)
    engine.add_order(Order { 
        order_id: generate_order_id(),
        symbol: symbol.to_string(),
        side: Side::Buy,
        price: 49980, 
        quantity: 20, 
        timestamp: generate_timestamp() 
    });

    engine.add_order(Order { 
        order_id: generate_order_id(),
        symbol: symbol.to_string(),
        side: Side::Buy,
        price: 49950, 
        quantity: 15, 
        timestamp: generate_timestamp() 
    });

    engine.add_order(Order { 
        order_id: generate_order_id(),
        symbol: symbol.to_string(),
        side: Side::Buy,
        price: 49980, 
        quantity: 10, 
        timestamp: generate_timestamp() 
    });

    engine.print_book(symbol);

    println!("2. Test Matching (Incoming Buy Order)");
    // Incoming Buy order at 50020, Qty 15
    // Matches:
    // 1. Resting Ask ID 1000 (50020 @ 10) - FULL FILL
    // 2. Remaining Qty 5 matches Ask ID 1002 (50020 @ 5) - FULL FILL
    engine.add_order(Order { 
        order_id: generate_order_id(), 
        symbol: symbol.to_string(), 
        side: Side::Buy, 
        price: 50020, 
        quantity: 15, 
        timestamp: generate_timestamp() 
    });

    engine.print_book(symbol);

    println!("3. Test Matching (Incoming Sell Order - Price Crossing)");
    // Incoming Sell order at 49900, Qty 35
    // Matches:
    // 1. Resting Bid ID 1003 (49980 @ 20) - FULL FILL
    // 2. Remaining Qty 15 matches Bid ID 1005 (49980 @ 10) - FULL FILL
    // 3. Remaining Qty 5 matches Bid ID 1004 (49950 @ 15) - PARTIAL FILL (Remaining 10 in book)
    // 4. Remaining Qty 0. Order is done.
    engine.add_order(Order { 
        order_id: generate_order_id(), 
        symbol: symbol.to_string(), 
        side: Side::Sell, 
        price: 49900, 
        quantity: 35, 
        timestamp: generate_timestamp() 
    });

    engine.print_book(symbol);

    println!("4. Test Cancellation of Remaining Order (ID 1004)");
    engine.cancel_order(1004, symbol);

    engine.print_book(symbol);

    println!("5. Test Order with No Match (Resting Order)");
    // This Sell order is far away from the best Bid (49950)
    engine.add_order(Order { 
        order_id: generate_order_id(), 
        symbol: symbol.to_string(), 
        side: Side::Sell, 
        price: 50500, 
        quantity: 50, 
        timestamp: generate_timestamp() 
    });

    engine.print_book(symbol);


    // 6. Final Trade Summary
    engine.print_all_trades();
    
}