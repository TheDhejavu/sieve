/// ! Sieve is a real-time data streaming and filtering engine for ethereum & the superchain
use alloy_primitives::U256;
use filter::{ArrayOps, FilterBuilder, NumericOps, StringOps};

mod config;
mod engine;
mod filter;
mod utils;

#[allow(dead_code)]
fn main() {
    //===============================================================================================
    //                                     1. SIMPLE OR FILTER
    //===============================================================================================
    let _simple_or_filter = FilterBuilder::new()
        .any_of(|f| {
            f.tx(|t| t.value().gt(U256::from(1000))); // Value > 1000
            f.tx(|t| t.gas_price().lt(50000)); // OR Gas price < 50
            f.tx(|t| t.nonce().eq(5)); // OR Nonce = 5
        })
        .build();

    //===============================================================================================
    //                         2. TRANSACTION AND EVENT FILTER COMBINATION
    //===============================================================================================
    let _combined_filter = FilterBuilder::new()
        .and(|f| {
            f.tx(|t| {
                t.value().gt(U256::from(100)); // Value > 100
                t.gas_price().lt(200); // AND Gas price < 200
            });
            f.event(|e| {
                e.contract().exact("UniswapV2Factory"); // AND Contract = UniswapV2Factory
            });
        })
        .build();

    //===============================================================================================
    //                                  3. DEFI NESTED FILTERS
    //===============================================================================================
    let _defi_filter = FilterBuilder::new()
        .any_of(|f| {
            f.all_of(|f| {
                f.event(|e| e.contract().exact("UniswapV2Factory"));
                f.tx(|t| {
                    t.value().gt(U256::from(100));
                    t.gas_price().lt(150);
                });
            });

            f.all_of(|f| {
                f.event(|e| e.contract().exact("TetherToken"));
                f.tx(|t| t.value().gt(U256::from(100)));
            });
        })
        .build();

    //===============================================================================================
    //                              4. TRANSACTION PATTERN FILTER
    //===============================================================================================
    let _pattern_filter = FilterBuilder::new()
        .any_of(|f| {
            f.tx(|t| t.value().gt(U256::from(100)));

            f.and(|f| {
                f.tx(|t| t.gas_price().between(50, 150));
                f.event(|e| {
                    e.contract().starts_with("0xDex");
                    e.topics().contains("Transfer".to_string());
                    // e.param("amount").exact("1000");
                    // e.param("from").starts_with("0xa1b2...");
                });
            });

            f.tx(|t| {
                t.gas().gt(500000);
                t.value().eq(U256::from(100));
            });
        })
        .build();

    //===============================================================================================
    //                             5. MULTI-PROTOCOL MONITORING
    //===============================================================================================
    let _monitoring_filter = FilterBuilder::new()
        .any_of(|f| {
            // Monitor multiple tokens & DEX
            f.any_of(|f| {
                f.event(|e| e.contract().exact("TetherToken"));
                f.event(|e| e.contract().exact("UniswapV2Factory"));
                f.event(|e| e.contract().exact("FiatTokenProxy"));
            });

            // Monitor lending protocols
            f.any_of(|f| {
                f.event(|e| e.contract().exact("Comp"));
                f.event(|e| e.contract().exact("InitializableAdminUpgradeabilityProxy"));
                f.event(|e| {
                    e.contract()
                        .exact("0xdAC17F958D2ee523a2206206994597C13D831ec7");
                    e.topics().contains(
                        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
                            .to_string(),
                    );

                    e.signature("Transfer(address indexed from,address indexed to,uint256 value)")
                        .params("value")
                        .gt(100_u128);
                });
            });
        })
        .and(|f| {
            // But only high-value transactions
            f.tx(|t| t.value().gt(U256::from(1000000)));
        })
        .build();

    //===============================================================================================
    //                            6. COMPREHENSIVE TRANSACTION FILTER
    //===============================================================================================
    let _comprehensive_filter = FilterBuilder::new()
        .any_of(|f| {
            // Basic transaction numeric fields
            f.tx(|t| {
                t.value().gt(U256::from(1000000));
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
                t.to().exact("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
                t.hash().contains("abc");
                t.block_hash().starts_with("0x0");
            });

            // Contract-specific calls fields
            f.tx(|t| {
                t.contract().method().exact("transfer");
                t.contract().params("tokenIn").gt(100);

                t.contract().path("tokenIn").starts_with("0x8");
            });
        })
        .build();

    //===============================================================================================
    //                            6. POOL FILTER
    //===============================================================================================
    let _pool_filter = FilterBuilder::new()
        .any_of(|f| {
            f.pool(|p| {
                // High value pending transaction
                p.value().gt(U256::from(1000000000000000000u64));
                // Specific sender/receiver
                p.from().starts_with("0xdead");
                p.to().exact("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
            });
        })
        .unless(|f| {
            f.block_header(|b| {
                b.gas_used().lt(1000);
                b.field("l1BlockNumber").gt(100_u128);
            });
        })
        .build();

    //===============================================================================================
    //                            6. BLOCK HEADER FILTER
    //===============================================================================================
    let _block_filter = FilterBuilder::new()
        .any_of(|f| {
            f.block_header(|b| {
                b.gas_limit().gt(100);
                b.hash().exact("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
                b.state_root()
                    .contains("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");

                b.receipts_root().starts_with("0xdead");
                b.base_fee().gt(100);

                b.gas_used().lt(1000);
            });
        })
        .build();

    //===============================================================================================
    //                            7. L2 FILTER
    //===============================================================================================
    let _filter = FilterBuilder::new()
        .optimism(|op| {
            op.field("l1BlockNumber").gt(1000000000000000000u128);

            op.field("l1TxOrigin").starts_with("0x");
            op.field("queueIndex").lt(100u64);

            // L2 block fields
            op.field("sequenceNumber").gt(500u64);
            op.field("prevTotalElements").between(1000u64, 2000u64);

            op.any_of(|f| {
                f.field("l1BlockNumber").gt(1000000000000000000u128);
                f.field("l1TxOrigin").starts_with("0x");
            });

            op.all_of(|f| {
                f.field("sequenceNumber").gt(500u64);
                f.field("batch.index").gt(100u128);
            });
        })
        .build();
}
