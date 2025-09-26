# 🦀 Basic Limit Order Book (LOB) Matching Engine

This project is a simple, single-threaded implementation of a **Limit Order Book (LOB)** and **Matching Engine** written in Rust. It demonstrates core exchange functionality, including order submission, cancellation, trade execution, and price-time priority matching.

## 🚀 Features

The engine implements the fundamental components required for processing limit orders:

* **Price-Time Priority:** Uses Rust's `BTreeMap` to ensure orders are matched according to the best price, and within the same price level, according to the time of submission (using a `VecDeque`).

* **Order Management:** Supports adding resting orders and cancelling existing orders by ID.

* **Trade Execution:** Processes incoming orders against the existing book, generating `Trade` records upon a successful match (price crossing).

* **Partial Fills:** Handles complex scenarios where an incoming order partially fills multiple resting orders, or where a resting order is partially filled and remains in the book.

* **Clear I/O:** Includes `Display` implementations for `Order` and `Trade` structs, and robust `print_book` functionality for visualizing the current state of the bids and asks.

## 🛠️ Data Structures

The core functionality revolves around three main data structures:

| Struct | Description | Implementation Detail | 
 | ----- | ----- | ----- | 
| **`Order`** | Represents a single buy or sell order placed by a user. | Includes `order_id`, `symbol`, `side`, `price`, `quantity`, and `timestamp`. | 
| **`Trade`** | Represents an executed match between a buyer and a seller. | Records the `buy_order_id`, `sell_order_id`, executed `price`, and `quantity`. | 
| **`SymbolBook`** | Manages the LOB for a single trading pair (symbol). | Uses `BTreeMap`s for `bids` (Max price first) and `asks` (Min price first) to ensure price ordering. | 
| **`MatchingEngine`** | The public wrapper that manages multiple `SymbolBook` instances (for different trading symbols) and stores the global `trades` history. | Provides the main API for `add_order` and `cancel_order`. | 

## ▶ How to Run the Demonstration

The provided code is a single, self-contained Rust file that includes a comprehensive `main` function to demonstrate the engine's capabilities.

### Prerequisites

You must have Rust installed on your system.

*(Note: You will also need to add `lazy_static = "1.4.0"` and `env_logger = "0.10"` to your `Cargo.toml` file to run this successfully in a proper Rust project.)*

### Demo Flow Highlights

The `main` function executes the following sequence of events:

1. **Initial Setup:** Populates the book with resting Bids and Asks for `BTC-USD`.

2. **Full Fill Match:** An incoming Buy order at the Ask price fully consumes two resting orders.

3. **Price Crossing & Partial Fill:** An aggressive incoming Sell order crosses the Bid/Ask spread, fully filling two existing Bids and partially filling a third.

4. **Cancellation Test:** Attempts to cancel the partially filled remaining order.

5. **Resting Order:** Submits an order far away from the market to confirm it sits in the book without matching.

6. **Trade Summary:** Prints the record of all generated trades.
