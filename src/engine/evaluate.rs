use super::{state::CacheKey, DecodedData};
use crate::filter::conditions::{
    BlockHeaderCondition, EventCondition, FilterCondition, PoolCondition, TransactionCondition,
};
use crate::{filter::evaluate::Evaluable, utils::decoder::EventDefinition};
use alloy_consensus::{BlockHeader, Transaction, Typed2718};
use alloy_primitives::{keccak256, Selector};
use alloy_rpc_types::{Header, Log, Transaction as RpcTransaction};
use std::sync::Arc;

pub(crate) trait EvaluableData {
    // Check if we should proceed with full evaluation
    fn pre_evaluate(&self, _condition: &FilterCondition) -> bool {
        true
    }

    fn cache_key(&self) -> CacheKey;

    fn evaluate(&self, condition: &FilterCondition, decoded_data: Option<Arc<DecodedData>>)
        -> bool;

    fn decode_data(&self, condition: &FilterCondition) -> Option<Arc<DecodedData>>;
}

impl EvaluableData for RpcTransaction {
    fn cache_key(&self) -> CacheKey {
        CacheKey::ContractCall(self.inner.tx_hash().to_string())
    }

    fn evaluate(
        &self,
        filter_condition: &FilterCondition,
        _decoded_data: Option<Arc<DecodedData>>,
    ) -> bool {
        match filter_condition {
            FilterCondition::Transaction(condition) => match condition {
                TransactionCondition::Value(condition) => condition.evaluate(&self.value()),
                TransactionCondition::GasPrice(condition) => {
                    condition.evaluate(&self.gas_price().unwrap_or_default())
                }
                TransactionCondition::From(condition) => condition.evaluate(&self.from.to_string()),
                TransactionCondition::MaxFeePerGas(condition) => {
                    condition.evaluate(&self.max_fee_per_gas())
                }
                TransactionCondition::MaxPriorityFee(condition) => {
                    condition.evaluate(&self.max_priority_fee_per_gas().unwrap_or_default())
                }
                TransactionCondition::BlockNumber(condition) => {
                    condition.evaluate(&self.block_number.unwrap_or_default())
                }
                TransactionCondition::BlockHash(condition) => {
                    condition.evaluate(&self.block_hash.unwrap_or_default().to_string())
                }
                TransactionCondition::ChainId(condition) => {
                    condition.evaluate(&self.chain_id().unwrap_or_default())
                }
                TransactionCondition::To(condition) => {
                    condition.evaluate(&self.to().unwrap_or_default().to_string())
                }
                TransactionCondition::Nonce(condition) => condition.evaluate(&self.nonce()),
                TransactionCondition::Type(condition) => condition.evaluate(&self.ty()),
                TransactionCondition::TransactionIndex(condition) => {
                    condition.evaluate(&self.transaction_index.unwrap_or_default())
                }
                TransactionCondition::Hash(condition) => {
                    condition.evaluate(&self.inner.tx_hash().to_string())
                }
                TransactionCondition::DynField(dyn_condition) => {
                    let json_value = serde_json::to_value(self).unwrap_or_default();
                    dyn_condition.evaluate(&json_value)
                }
                _ => false,
            },
            FilterCondition::Pool(pool_condition) => match pool_condition {
                PoolCondition::Value(condition) => condition.evaluate(&self.value()),
                PoolCondition::GasPrice(condition) => {
                    condition.evaluate(&self.gas_price().unwrap_or_default())
                }
                PoolCondition::From(condition) => condition.evaluate(&self.from.to_string()),
                PoolCondition::Nonce(condition) => condition.evaluate(&self.nonce()),
                PoolCondition::GasLimit(condition) => condition.evaluate(&self.gas_limit()),
                PoolCondition::Hash(condition) => {
                    condition.evaluate(&self.inner.tx_hash().to_string())
                }
                PoolCondition::To(condition) => {
                    condition.evaluate(&self.to().unwrap_or_default().to_string())
                }
            },
            FilterCondition::DynField(dyn_condition) => {
                let json_value = serde_json::to_value(self).unwrap_or_default();
                dyn_condition.evaluate(&json_value)
            }
            _ => false,
        }
    }

    fn decode_data(&self, _condition: &FilterCondition) -> Option<Arc<DecodedData>> {
        unimplemented!()
    }

    fn pre_evaluate(&self, condition: &FilterCondition) -> bool {
        match condition {
            FilterCondition::Transaction(TransactionCondition::CallData {
                method_selector,
                ..
            }) => {
                let input = self.input();
                if input.len() >= 4 {
                    let actual_selector = Selector::from_slice(&input[0..4]);
                    return &actual_selector == method_selector;
                }
                false
            }
            _ => true,
        }
    }
}

impl EvaluableData for Header {
    fn cache_key(&self) -> CacheKey {
        CacheKey::ContractCall(self.hash.to_string())
    }

    fn evaluate(
        &self,
        condition: &FilterCondition,
        _decoded_data: Option<Arc<DecodedData>>,
    ) -> bool {
        match condition {
            FilterCondition::BlockHeader(condition) => match condition {
                BlockHeaderCondition::BaseFee(condition) => {
                    condition.evaluate(&self.base_fee_per_gas().unwrap_or_default())
                }
                BlockHeaderCondition::Number(condition) => condition.evaluate(&self.number),
                BlockHeaderCondition::Timestamp(condition) => condition.evaluate(&self.timestamp),
                BlockHeaderCondition::Size(condition) => {
                    condition.evaluate(&self.size.unwrap_or_default())
                }
                BlockHeaderCondition::GasUsed(condition) => condition.evaluate(&self.gas_used),
                BlockHeaderCondition::GasLimit(condition) => condition.evaluate(&self.gas_limit),
                BlockHeaderCondition::Hash(condition) => condition.evaluate(&self.hash.to_string()),
                BlockHeaderCondition::ParentHash(condition) => {
                    condition.evaluate(&self.parent_hash.to_string())
                }
                BlockHeaderCondition::StateRoot(condition) => {
                    condition.evaluate(&self.state_root.to_string())
                }
                BlockHeaderCondition::ReceiptsRoot(condition) => {
                    condition.evaluate(&self.receipts_root.to_string())
                }
                BlockHeaderCondition::TransactionsRoot(condition) => {
                    condition.evaluate(&self.transactions_root.to_string())
                }
                BlockHeaderCondition::DynField(dyn_condition) => {
                    let json_value = serde_json::to_value(self).unwrap_or_default();
                    dyn_condition.evaluate(&json_value)
                }
            },
            _ => false,
        }
    }

    fn decode_data(&self, _condition: &FilterCondition) -> Option<Arc<DecodedData>> {
        unimplemented!()
    }
}

impl EvaluableData for Log {
    fn cache_key(&self) -> CacheKey {
        CacheKey::Event(format!("{:?}", self.log_index))
    }

    fn evaluate(
        &self,
        condition: &FilterCondition,
        decoded_data: Option<Arc<DecodedData>>,
    ) -> bool {
        match condition {
            FilterCondition::Event(event_condition) => match event_condition {
                EventCondition::Contract(condition) => {
                    condition.evaluate(&self.address().to_string())
                }
                EventCondition::BlockHash(condition) => {
                    condition.evaluate(&self.block_hash.unwrap_or_default().to_string())
                }
                EventCondition::TxHash(condition) => {
                    condition.evaluate(&self.transaction_hash.unwrap_or_default().to_string())
                }
                EventCondition::LogIndex(condition) => {
                    condition.evaluate(&self.log_index.unwrap_or_default())
                }
                EventCondition::BlockNumber(condition) => {
                    condition.evaluate(&self.block_number.unwrap_or_default())
                }
                EventCondition::TxIndex(condition) => {
                    condition.evaluate(&self.transaction_index.unwrap_or_default())
                }
                EventCondition::Topics(condition) => {
                    let topics: Vec<String> = self
                        .topics()
                        .iter()
                        .map(|topic| topic.to_string())
                        .collect();
                    condition.evaluate(&topics)
                }
                EventCondition::EventData { parameters, .. } => match decoded_data {
                    Some(data) => {
                        if let DecodedData::Event(decoded_log) = data.as_ref() {
                            parameters.iter().all(|(param, condition)| {
                                decoded_log
                                    .get_parameter(param)
                                    .map_or(false, |value| condition.evaluate(value))
                            })
                        } else {
                            false
                        }
                    }
                    None => false,
                },
                EventCondition::DynField(dyn_condition) => {
                    let json_value = serde_json::to_value(self).unwrap_or_default();
                    dyn_condition.evaluate(&json_value)
                }
            },
            _ => false,
        }
    }
    fn pre_evaluate(&self, condition: &FilterCondition) -> bool {
        match condition {
            FilterCondition::Event(EventCondition::EventData { signature, .. }) => {
                let data = self.data();
                let topic_0 = data.topics()[0];
                // Calculate the method selector from the signature
                let method_selector = Selector::from_slice(&keccak256(signature.as_bytes())[0..4]);
                let topic_selector = &topic_0[0..4];
                // Compare the selector with the first 4 bytes of topic0
                topic_selector == method_selector.as_slice()
            }
            _ => false,
        }
    }

    fn decode_data(&self, condition: &FilterCondition) -> Option<Arc<DecodedData>> {
        let FilterCondition::Event(EventCondition::EventData { signature, .. }) = condition else {
            return None;
        };

        // TODO: Verify method signature hash against topic[0] to determine if
        // this log is valid enough to be decoded.

        let event = EventDefinition::from_signature(signature).ok()?;
        let event_log = event.decode_log(&self.inner.data).ok()?;

        Some(Arc::new(DecodedData::Event(event_log)))
    }
}
