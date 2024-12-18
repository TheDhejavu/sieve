# sieve
A real-time data streaming & filtering engine for Ethereum & the superchain.

## Overview
Sieve provides a simple and expressive way to filter blockchain data streams:
- Transactions (confirmed and pending)
- Events (logs)
- Blocks

## Usage

```rust
use sieve::{FilterBuilder, NumericOps, StringOps};

fn main() {
    // Filter for high-value transactions
    let value_filter = FilterBuilder::new()
        .tx(|t| {
            t.value().gt(1000);          // Value > 1000
            t.gas_price().lt(50);        // Gas price < 50
        })
        .build();

    // Filter for Uniswap events
    let event_filter = FilterBuilder::new()
        .event(|e| {
            e.contract().eq("UniswapV2Factory");
        })
        .build();
}
```

## Status
🚧 Experimental - Not ready for production use 