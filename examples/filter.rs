extern crate sieve;

use alloy_primitives::U256;
use sieve::prelude::*;

#[allow(dead_code)]
fn main() {
    //===============================================================================================
    //                                     1. SIMPLE OR FILTER
    //===============================================================================================
    let _simple_or_filter = FilterBuilder::new().transaction(|f| {
        f.or(|tx| {
            tx.value().gt(U256::from(1000)); // Value > 1000
            tx.gas_price().lt(50000); // OR Gas price < 50
            tx.nonce().eq(5); // OR Nonce = 5
        });
    });

    //===============================================================================================
    //                              2. TRANSACTION PATTERN FILTER
    //===============================================================================================
    let _pattern_filter = FilterBuilder::new().transaction(|f| {
        f.value().gt(U256::from(100));

        f.all_of(|f| {
            f.gas_price().between(50, 150);
        });

        f.or(|t| {
            t.gas().gt(500000);
            t.value().eq(U256::from(100));
        });
    });

    //===============================================================================================
    //                             3. MULTI-PROTOCOL MONITORING
    //===============================================================================================
    let _monitoring_filter = FilterBuilder::new().event(|f| {
        // Monitor multiple tokens & DEX
        f.any_of(|e| {
            e.contract().exact("TetherToken");
            e.contract().exact("UniswapV2Factory");
            e.contract().exact("FiatTokenProxy");
        });

        // Monitor lending protocols
        f.any_of(|e| {
            e.contract().exact("Comp");
            e.contract().exact("InitializableAdminUpgradeabilityProxy");
            e.contract()
                .exact("0xdAC17F958D2ee523a2206206994597C13D831ec7");
            e.topics().contains(
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(),
            );

            e.signature("Transfer(address indexed from,address indexed to,uint256 value)")
                .params("value1")
                .gt(100_u128);
        });
    });

    //===============================================================================================
    //                            4. COMPREHENSIVE TRANSACTION FILTER
    //===============================================================================================
    let _comprehensive_filter = FilterBuilder::new().transaction(|f| {
        // Basic transaction numeric fields
        f.any_of(|t| {
            t.value().gt(U256::from(1000000));
            t.gas_price().lt(50_000_000_000);
            t.gas().between(21000, 100000);
            t.nonce().eq(5);

            t.access_list()
                .contains("0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string());
        });

        // EIP-1559 fields
        f.any_of(|t| {
            t.max_fee_per_gas().lt(100_000_000_000);
            t.max_priority_fee().lt(2_000_000_000);
            t.tx_type().eq(2);
        });

        // Block and chain fields
        f.any_of(|t| {
            t.block_number().gt(1000000);
            t.index().lt(100);
            t.chain_id().eq(1);
        });

        // Address and hash fields
        f.or(|t| {
            t.from().starts_with("0xdead");
            t.to().exact("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
            t.hash().contains("abc");
            t.block_hash().starts_with("0x0");
        });
    });

    //===============================================================================================
    //                            5. POOL FILTER
    //===============================================================================================
    let _pool_filter = FilterBuilder::new().pool(|f| {
        f.any_of(|p| {
            // High value pending transaction
            p.value().gt(U256::from(1000000000000000000u64));
            // Specific sender/receiver
            p.from().starts_with("0xdead");
            p.to().exact("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
        });
    });

    //===============================================================================================
    //                            6. BLOCK HEADER FILTER
    //===============================================================================================
    let _block_filter = FilterBuilder::new().block_header(|f| {
        f.or(|b| {
            b.gas_limit().gt(100);
            b.hash().exact("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
            b.state_root()
                .contains("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");

            b.receipts_root().starts_with("0xdead");
            b.base_fee().gt(100);

            b.gas_used().lt(1000);
        });
    });

    //===============================================================================================
    //                            7. L2 FILTER
    //===============================================================================================
    let _filter = FilterBuilder::new()
        .chain(Chain::Optimism)
        .transaction(|op_tx| {
            //
            op_tx.value().gt(U256::from(1000000000000000000u64));

            // NOTE: This is currently not supported yet.
            op_tx.field("l1BlockNumber").gt(1000000000000000000u128);

            op_tx.field("l1TxOrigin").starts_with("0x");
            op_tx.field("queueIndex").lt(100u64);

            // L2 block fields
            op_tx.field("sequenceNumber").gt(500u64);
            op_tx.field("prevTotalElements").between(1000u64, 2000u64);

            op_tx.any_of(|f| {
                f.field("l1BlockNumber").gt(1000000000000000000u128);
                f.field("l1TxOrigin").starts_with("0x");
            });

            op_tx.all_of(|f| {
                f.field("sequenceNumber").gt(500u64);
                f.field("batch.index").gt(100u128);
            });
        });
}
