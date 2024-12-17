use filter::{ArrayOps, FilterBuilder, NumericOps, StringOps};

// ! Sieve is a real-time data streaming and filtering engine for ethereum & the superchain
mod filter;

fn main() {
    //===============================================================================================
    //                                     1. SIMPLE OR FILTER
    //===============================================================================================
    let simple_or_filter = FilterBuilder::new()
        .any_of(|f| {
            f.tx(|t| t.value().gt(1000)); // Value > 1000
            f.tx(|t| t.gas_price().lt(50)); // OR Gas price < 50
            f.tx(|t| t.nonce().eq(5)); // OR Nonce = 5
        })
        .build();

    //===============================================================================================
    //                         2. TRANSACTION AND EVENT FILTER COMBINATION
    //===============================================================================================
    let combined_filter = FilterBuilder::new()
        .and(|f| {
            f.tx(|t| {
                t.value().gt(100); // Value > 100
                t.gas_price().lt(200); // AND Gas price < 200
                t.transfer().amount().lt(100)
            });
            f.event(|e| {
                e.contract().eq("UniswapV2Factory"); // AND Contract = UniswapV2Factory
            });
        })
        .build();

    //===============================================================================================
    //                                  3. DEFI NESTED FILTERS
    //===============================================================================================
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

    //===============================================================================================
    //                              4. TRANSACTION PATTERN FILTER
    //===============================================================================================
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

    //===============================================================================================
    //                             5. MULTI-PROTOCOL MONITORING
    //===============================================================================================
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

    //===============================================================================================
    //                            6. COMPREHENSIVE TRANSACTION FILTER
    //===============================================================================================
    let comprehensive_filter = FilterBuilder::new()
        .any_of(|f| {
            // Basic transaction numeric fields
            f.tx(|t| {
                t.value().gt(1000000);
                t.gas_price().lt(50_000_000_000);
                t.gas().between(21000, 100000);
                t.nonce().eq(5);

                t.access_list()
                    .contains("0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string());
            });

            // EIP-1559 fields
            f.tx(|t| {
                t.max_fee_per_gas().lt(100_000_000_000);
                t.max_priority_fee().lt(2_000_000_000);
                t.tx_type().eq(2);
            });

            // Block and chain fields
            f.tx(|t| {
                t.block_number().gt(1000000);
                t.index().lt(100);
                t.chain_id().eq(1);
            });

            // Address and hash fields
            f.tx(|t| {
                t.from().starts_with("0xdead");
                t.to().eq("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
                t.hash().contains("abc");
                t.block_hash().starts_with("0x0");
            });

            // Transfer-specific fields
            f.tx(|t| {
                t.transfer().method().eq("transfer");
                t.transfer().amount().gt(1000);
                t.transfer().to().contains("dead");
                t.transfer().from().starts_with("0x");
                t.transfer()
                    .spender()
                    .eq("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");
            });
        })
        .build();
}
