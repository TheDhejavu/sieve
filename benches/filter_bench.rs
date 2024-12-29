use alloy_consensus::{Signed, TxEip7702, TxEnvelope};
use alloy_network::{AnyRpcTransaction, AnyTxEnvelope};
use alloy_primitives::{ruint::aliases::U256, Address, FixedBytes, PrimitiveSignature, B256};
use alloy_rpc_types::{AccessList, Transaction};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{thread_rng, Rng};
use std::{str::FromStr, sync::Arc};

use sieve::prelude::*;

fn generate_random_address() -> String {
    let mut rng = thread_rng();
    let mut address = String::with_capacity(42);
    address.push_str("0x");
    for _ in 0..40 {
        address.push(rng.gen_range('0'..='f'));
    }
    address
}

fn generate_best_case_filter() -> Filter {
    FilterBuilder::new().transaction(|tx| {
        tx.value().gt(U256::from(u64::MAX));
        tx.to().exact("0xdead000000000000000000000000000000000000");
        tx.nonce().eq(u64::MAX);
    })
}

fn generate_worst_case_filter() -> Filter {
    let mut rng = thread_rng();
    // Constructs complex filter definitions for worst-case performance testing.
    // While more complex than typical real-world usage, we use `any_of`/`or`
    // operations to ensure full evaluation in worst-case scenarios. This is because
    // `or` must check all conditions if no true condition is found, preventing
    // short-circuit optimization, unlike `all_of` which can return early on first false.
    FilterBuilder::new().transaction(|f| {
        f.any_of(|tx| {
            tx.any_of(|t| {
                t.value()
                    .gt(U256::from(rng.gen_range(0..1_000_000_000_000_000_000u64)));
                t.to().exact(generate_random_address().as_str());
                t.from().starts_with("0xdead");
            });

            tx.any_of(|t| {
                t.tx_type().eq(2u8);
                t.max_fee_per_gas().lt(100_000_000_000u128);
                t.max_priority_fee()
                    .between(1_000_000_000u128, 10_000_000_000u128);
            });

            tx.any_of(|t| {
                t.nonce().lt(rng.gen_range(0..1000));
            });

            tx.any_of(|t| {
                t.any_of(|inner| {
                    inner.gas_price().gt(50_000_000_000u128);
                    inner.gas().between(21000, 500000);
                });

                t.any_of(|inner| {
                    inner.block_number().gt(1000000);
                    inner.chain_id().eq(1);
                    inner.hash().contains("dead");
                });

                t.access_list().contains(generate_random_address());
            });
        });
    })
}
pub fn generate_random_transaction() -> AnyRpcTransaction {
    let chain_id = 1;
    let gas_limit: u64 = 10;
    let max_fee_per_gas: u128 = rand::thread_rng()
        .gen_range(20_000_000_000u64..100_000_000_000u64)
        .into();
    let max_priority_fee_per_gas: u128 = rand::thread_rng()
        .gen_range(1_000_000_000u64..10_000_000_000u64)
        .into();
    let to = Address::from_str("0x8ba1f109551bD432803012645Ac136ddd64dBa72").unwrap();
    let value = U256::from(rand::thread_rng().gen_range(0..1_000_000_000_000_000_000u64));
    let authorization_list = vec![];
    let input = vec![];

    // Construct the EIP-7702 transaction
    let eip_7702 = TxEip7702 {
        chain_id,
        nonce: 10,
        gas_limit,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        to,
        value,
        access_list: AccessList::default(),
        authorization_list,
        input: input.into(),
    };
    let signature = PrimitiveSignature::new(
        U256::from_str("0xc569c92f176a3be1a6352dd5005bfc751dcb32f57623dd2a23693e64bf4447b0")
            .unwrap(),
        U256::from_str("0x1a891b566d369e79b7a66eecab1e008831e22daa15f91a0a0cf4f9f28f47ee05")
            .unwrap(),
        true,
    );
    let hash = B256::default();
    let tx_envelope = TxEnvelope::Eip7702(Signed::new_unchecked(eip_7702, signature, hash));

    AnyRpcTransaction::new(Transaction {
        inner: alloy_network::AnyTxEnvelope::Ethereum(tx_envelope),
        block_hash: Some(FixedBytes::default()),
        block_number: Some(1),
        transaction_index: Some(0),
        effective_gas_price: Some(20_000_000_000u128),
        from: to,
    })
}

fn bench_filter_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_evaluation");
    let transactions: Vec<_> = (0..300).map(|_| generate_random_transaction()).collect();

    // Test with different numbers of filters
    for num_of_filters in [10, 50, 100, 200].iter() {
        let best_case_filters: Vec<_> = (0..*num_of_filters)
            .map(|_| generate_best_case_filter())
            .collect();

        let worst_case_filters: Vec<_> = (0..*num_of_filters)
            .map(|_| generate_worst_case_filter())
            .collect();

        let engine = FilterEngine::new();

        // Benchmark best case
        group.bench_with_input(
            BenchmarkId::new("best_case", num_of_filters),
            &(worst_case_filters, transactions.clone()),
            |b, (filters, txs)| {
                b.iter(|| {
                    for tx in txs {
                        for filter in filters {
                            criterion::black_box(engine.evaluate_with_context(
                                filter.filter_node().as_ref(),
                                Arc::new(tx.clone()),
                            ));
                        }
                    }
                });
            },
        );

        // Benchmark worst case
        group.bench_with_input(
            BenchmarkId::new("worst_case", num_of_filters),
            &(best_case_filters, transactions.clone()),
            |b, (filters, txs)| {
                b.iter(|| {
                    for tx in txs {
                        for filter in filters {
                            criterion::black_box(engine.evaluate_with_context(
                                filter.filter_node().as_ref(),
                                Arc::new(tx.clone()),
                            ));
                        }
                    }
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_filter_evaluation);
criterion_main!(benches);
