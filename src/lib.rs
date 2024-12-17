use filter::{FilterBuilder, NumericOps, StringOps};

// ! Sieve is a real-time data streaming and filtering engine for ethereum & the superchain
mod filter;

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

    // Example 3: Complex nested filters for DeFi transactions
    let defi_filter = FilterBuilder::new()
        .any_of(|f| {
            f.all_of(|f| {
                f.event(|e| e.contract().eq("UniswapV2Factory"));
                f.tx(|t| {
                    t.value().gt(1000);
                    t.gas_price().lt(150);
                });
            });

            f.all_of(|f| {
                f.event(|e| e.contract().eq("TetherToken"));
                f.tx(|t| t.value().gt(5000));
            });
        })
        .build();

    // Example 4: Filtering specific transaction patterns
    let pattern_filter = FilterBuilder::new()
        .any_of(|f| {
            f.tx(|t| t.value().gt(10000));

            f.and(|f| {
                f.tx(|t| t.gas_price().between(50, 150));
                f.event(|e| e.contract().starts_with("0xDex"));
            });

            f.tx(|t| {
                t.gas().gt(500000);
                t.value().eq(0);
            });
        })
        .build();

    // Example 5: Multi-protocol monitoring
    let monitoring_filter = FilterBuilder::new()
        .any_of(|f| {
            // Monitor multiple tokens & DEX
            f.any_of(|f| {
                f.event(|e| e.contract().eq("TetherToken"));
                f.event(|e| e.contract().eq("UniswapV2Factory"));
                f.event(|e| e.contract().eq("FiatTokenProxy"));
            });

            // Monitor lending protocols
            f.any_of(|f| {
                f.event(|e| e.contract().eq("Comp"));
                f.event(|e| e.contract().eq("InitializableAdminUpgradeabilityProxy"));
            });
        })
        .and(|f| {
            // But only high-value transactions
            f.tx(|t| t.value().gt(50000));
        })
        .build();
}
