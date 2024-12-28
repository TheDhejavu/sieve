use alloy_consensus::{Signed, TxEip7702, TxEnvelope};
use alloy_primitives::{ruint::aliases::U256, Address, FixedBytes, PrimitiveSignature, B256};
use alloy_rpc_types::{AccessList, Transaction};
use rand::Rng;
use std::str::FromStr;

#[allow(dead_code)]
pub fn generate_random_transaction(value: u64) -> alloy_rpc_types::Transaction<TxEnvelope> {
    let chain_id = 1;
    let gas_limit: u64 = 10;
    let max_fee_per_gas: u128 = rand::thread_rng()
        .gen_range(20_000_000_000u64..100_000_000_000u64)
        .into();
    let max_priority_fee_per_gas: u128 = rand::thread_rng()
        .gen_range(1_000_000_000u64..10_000_000_000u64)
        .into();
    let to = Address::from_str("0x8ba1f109551bD432803012645Ac136ddd64dBa72").unwrap();
    let value = U256::from(value);
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
        block_number: Some(10),
        transaction_index: Some(0),
        effective_gas_price: Some(20_000_000_000u128),
        from: to,
    }
}
