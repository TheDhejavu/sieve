use std::str::FromStr;

use alloy_consensus::{Signed, TxEip7702, TxEnvelope};
use alloy_primitives::{ruint::aliases::U256, Address, FixedBytes, PrimitiveSignature, B256};
use alloy_rpc_types::{AccessList, Transaction};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{thread_rng, Rng};

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

fn generate_random_filter() -> FilterNode {
    let mut rng = thread_rng();
    let ops = [
        LogicalOp::And,
        LogicalOp::Or,
        LogicalOp::Not,
        LogicalOp::Xor,
    ];
    let op = ops[rng.gen_range(0..ops.len())].clone();

    // Generate between 2-4 conditions for each filter
    let num_conditions = rng.gen_range(2..=4);
    let conditions = (0..num_conditions)
        .map(|_| {
            let condition_type = rng.gen_range(0..4);
            match condition_type {
                0 => FilterNode {
                    children: None,
                    value: Some(FilterCondition::Transaction(TransactionCondition::To(
                        StringCondition::EqualTo(generate_random_address()),
                    ))),
                },
                1 => FilterNode {
                    children: None,
                    value: Some(FilterCondition::Transaction(TransactionCondition::Value(
                        NumericCondition::GreaterThan(U256::from(rng.gen_range(0..1000000))),
                    ))),
                },
                2 => FilterNode {
                    children: None,
                    value: Some(FilterCondition::Transaction(TransactionCondition::Type(
                        NumericCondition::EqualTo(rng.gen_range(0..3)),
                    ))),
                },
                _ => FilterNode {
                    children: None,
                    value: Some(FilterCondition::Transaction(TransactionCondition::Nonce(
                        NumericCondition::GreaterThan(rng.gen_range(0..1000)),
                    ))),
                },
            }
        })
        .collect::<Vec<_>>();

    FilterNode {
        children: Some((op, conditions)),
        value: None,
    }
}

pub fn generate_random_transaction() -> alloy_rpc_types::Transaction<TxEnvelope> {
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

    Transaction {
        inner: tx_envelope,
        block_hash: Some(FixedBytes::default()),
        block_number: Some(1),
        transaction_index: Some(0),
        effective_gas_price: Some(20_000_000_000u128),
        from: to,
    }
}

fn bench_filter_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_evaluation");
    let transactions: Vec<_> = (0..300).map(|_| generate_random_transaction()).collect();

    // Test with different numbers of filters
    for num_of_filters in [10, 100, 500, 1000].iter() {
        let filters: Vec<_> = (0..*num_of_filters)
            .map(|_| generate_random_filter())
            .collect();

        group.bench_with_input(
            BenchmarkId::new("num_of_filters", num_of_filters),
            &(filters, transactions.clone()),
            |b, (filters, txs)| {
                let engine = FilterEngine::new();
                b.iter(|| {
                    for tx in txs {
                        for filter in filters {
                            criterion::black_box(engine.evaluate_with_context(filter, tx.clone()));
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
