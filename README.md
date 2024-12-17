# sieve
Experimental real-time data streaming and filtering engine for ethereum &amp; the superchain


## Example
```rust 
use sieve::{FilterBuilder, NumericOps, StringOps};

fn main() {
    // Example 1: Simple transaction filters combined with OR
    let simple_or_filter = FilterBuilder::new()
        .any_of(|f| {
            f.tx(|t| t.value().gt(1000)); // Value > 1000
            f.tx(|t| t.gas_price().lt(50)); // OR Gas price < 50
            f.tx(|t| t.nonce().eq(5)); // OR Nonce = 5
        })
        .build();

    // Example 2: Combined transaction and event filters with AND
    let combined_filter = FilterBuilder::new()
        .and(|f| {
            f.tx(|t| {
                t.value().gt(100); // Value > 100
                t.gas_price().lt(200); // AND Gas price < 200
            });
            f.event(|e| {
                e.contract().eq("UniswapV2Factory"); // AND Contract = UniswapV2Factory
            });
        })
        .build();
}

```