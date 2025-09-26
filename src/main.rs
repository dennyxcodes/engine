use std::fmt::{self, Display, Formatter};

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
fn main() {
    print!("Hello, world!");
}