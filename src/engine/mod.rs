//! Filter engine handles concurrent filter evaluations using Rayon workers for parallel processing.
//! While data decoding can be intensive, results are cached globally to prevent redundant operations.
//! The parallel design enables efficient processing of multiple simultaneous filter conditions.
use std::sync::Arc;

use context::EvaluationContext;
use dashmap::DashMap;
use evaluate::EvaluableData;
use rayon::prelude::*;
use state::State;

use crate::filter::conditions::{FilterNode, LogicalOp};
mod context;
mod evaluate;
mod state;

pub(crate) use state::DecodedData;

#[allow(dead_code)]
#[derive(Clone)]
pub struct FilterEngine {
    state: State,
}

#[allow(dead_code)]
impl Default for FilterEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterEngine {
    pub fn new() -> Self {
        Self {
            state: State {
                decoded_data: Arc::new(DashMap::new()),
            },
        }
    }

    fn evaluate<D>(filter: &FilterNode, ctx: &EvaluationContext<D>) -> bool
    where
        D: EvaluableData + Send + Sync,
    {
        match &filter.value {
            Some(condition) => ctx.evaluate(condition),
            None => filter.children.as_ref().map_or(false, |(op, nodes)| {
                let parallel_iter = nodes.par_iter();

                match op {
                    LogicalOp::And => parallel_iter.all(|node| Self::evaluate(node, ctx)),
                    LogicalOp::Or => parallel_iter.any(|node| Self::evaluate(node, ctx)),
                    LogicalOp::Not => !parallel_iter.all(|node| Self::evaluate(node, ctx)),
                    LogicalOp::Xor => {
                        let count = parallel_iter
                            .filter(|node| Self::evaluate(node, ctx))
                            .count();
                        count == 1
                    }
                }
            }),
        }
    }

    pub fn evaluate_with_context<D>(&self, filter: &FilterNode, data: Arc<D>) -> bool
    where
        D: EvaluableData + Send + Sync,
    {
        let ctx = EvaluationContext::new(data, &self.state);
        Self::evaluate(filter, &ctx)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::conditions::{
        FilterCondition, FilterNode, LogicalOp, NumericCondition, StringCondition,
        TransactionCondition,
    };
    use alloy_network::{AnyRpcTransaction, AnyTxEnvelope};
    use alloy_primitives::U256;
    use alloy_rpc_types::Transaction as RpcTransaction;

    fn create_test_transaction() -> RpcTransaction<AnyTxEnvelope> {
        let rpc_tx = r#"{
            "blockHash": "0x883f974b17ca7b28cb970798d1c80f4d4bb427473dc6d39b2a7fe24edc02902d",
            "blockNumber": "0xe26e6d",
            "hash": "0x0e07d8b53ed3d91314c80e53cf25bcde02084939395845cbb625b029d568135c",
            "accessList": [],
            "transactionIndex": "0xad",
            "type": "0x2",
            "nonce": "0x16d",
            "input": "0x5ae401dc00000000000000000000000000000000000000000000000000000000628ced5b000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000000e442712a6700000000000000000000000000000000000000000000b3ff1489674e11c40000000000000000000000000000000000000000000000000000004a6ed55bbcc18000000000000000000000000000000000000000000000000000000000000000800000000000000000000000003cf412d970474804623bb4e3a42de13f9bca54360000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000003a75941763f31c930b19c041b709742b0b31ebb600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000412210e8a00000000000000000000000000000000000000000000000000000000",
            "r": "0x7f2153019a74025d83a73effdd91503ceecefac7e35dd933adc1901c875539aa",
            "s": "0x334ab2f714796d13c825fddf12aad01438db3a8152b2fe3ef7827707c25ecab3",
            "chainId": "0x1",
            "v": "0x0",
            "gas": "0x46a02",
            "maxPriorityFeePerGas": "0x59682f00",
            "from": "0x3cf412d970474804623bb4e3a42de13f9bca5436",
            "to": "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45",
            "maxFeePerGas": "0x7fc1a20a8",
            "value": "0x4a6ed55bbcc180",
            "gasPrice": "0x50101df3a"
        }"#;

        serde_json::from_str::<RpcTransaction<AnyTxEnvelope>>(rpc_tx).unwrap()
    }

    #[test]
    fn test_matching_complex_condition() {
        let engine = FilterEngine::new();
        let tx = create_test_transaction();

        // Complex matching condition:
        // (to == uniswap_router AND type == 2) OR (nonce > 300 AND maxPriorityFeePerGas > 1 gwei)
        let filter = FilterNode {
            children: Some((
                LogicalOp::Or,
                vec![
                    FilterNode {
                        children: Some((
                            LogicalOp::And,
                            vec![
                                FilterNode {
                                    children: None,
                                    value: Some(FilterCondition::Transaction(
                                        TransactionCondition::To(StringCondition::EqualTo(
                                            "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45"
                                                .to_string(),
                                        )),
                                    )),
                                },
                                FilterNode {
                                    children: None,
                                    value: Some(FilterCondition::Transaction(
                                        TransactionCondition::Type(NumericCondition::EqualTo(2u8)),
                                    )),
                                },
                            ],
                        )),
                        value: None,
                    },
                    FilterNode {
                        children: Some((
                            LogicalOp::And,
                            vec![
                                FilterNode {
                                    children: None,
                                    value: Some(FilterCondition::Transaction(
                                        TransactionCondition::Nonce(NumericCondition::GreaterThan(
                                            300u64,
                                        )),
                                    )),
                                },
                                FilterNode {
                                    children: None,
                                    value: Some(FilterCondition::Transaction(
                                        TransactionCondition::MaxPriorityFee(
                                            NumericCondition::GreaterThan(1_000_000_000u128),
                                        ),
                                    )),
                                },
                            ],
                        )),
                        value: None,
                    },
                ],
            )),
            value: None,
        };

        let result = engine.evaluate_with_context(&filter, Arc::new(AnyRpcTransaction::new(tx)));
        assert!(result, "Transaction should match the complex conditions");
    }

    #[test]
    fn test_unmatching_transaction() {
        let engine = FilterEngine::new();
        let tx = create_test_transaction();

        // Create a filter that won't match the transaction:
        // (to == different_router) AND (value > 0) AND (type == 2)
        let filter = FilterNode {
            children: Some((
                LogicalOp::And,
                vec![
                    FilterNode {
                        children: None,
                        value: Some(FilterCondition::Transaction(TransactionCondition::To(
                            StringCondition::EqualTo(
                                "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45".to_string(),
                            ),
                        ))),
                    },
                    FilterNode {
                        children: None,
                        value: Some(FilterCondition::Transaction(TransactionCondition::Value(
                            NumericCondition::GreaterThan(U256::ZERO),
                        ))),
                    },
                    FilterNode {
                        children: None,
                        value: Some(FilterCondition::Transaction(TransactionCondition::Type(
                            NumericCondition::EqualTo(2u8),
                        ))),
                    },
                ],
            )),
            value: None,
        };

        let result = engine.evaluate_with_context(&filter, Arc::new(AnyRpcTransaction::new(tx)));
        assert!(
            !result,
            "Transaction should not match Uniswap Router criteria"
        );
    }
}
