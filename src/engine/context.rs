use super::{
    state::{CacheKey, State},
    DecodedData,
};
use crate::filter::evaluate::Evaluable;
use crate::{
    filter::conditions::{
        BlockHeaderCondition, EventCondition, FilterCondition, PoolCondition, TransactionCondition,
    },
    utils::decoder::parse_event_signature,
};
use alloy_consensus::{BlockHeader, Transaction, Typed2718};
use alloy_rpc_types::{Header, Log, Transaction as RpcTransaction};
use std::sync::Arc;

pub(crate) trait EvaluableData {
    // Check if we should proceed with full evaluation
    fn pre_evaluate(&self, _condition: &FilterCondition) -> bool {
        // Default implementation always returns true
        // Individual implementations will override this
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
        decoded_data: Option<Arc<DecodedData>>,
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
                TransactionCondition::Method(condition) => {
                    if let Some(DecodedData::ContractCall(decoded)) =
                        decoded_data.as_ref().map(|arc| arc.as_ref())
                    {
                        let method = decoded.get_method();
                        return condition.evaluate(&method.to_string());
                    }
                    false
                }
                // TransactionCondition::Path(_path, _condition) => true,
                // TransactionCondition::Parameter(param, condition) => {
                //     if let Some(DecodedData::ContractCall(decoded)) =
                //         decoded_data.as_ref().map(|arc| arc.as_ref())
                //     {
                //         let parameter_value = decoded.get_parameter(param);
                //         if let Some(value) = parameter_value {
                //             // return condition.evaluate(value);
                //         }
                //     }
                //     false
                // }
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
            FilterCondition::Transaction(_) => true,
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

    fn decode_data(&self, condition: &FilterCondition) -> Option<Arc<DecodedData>> {
        let FilterCondition::Event(EventCondition::EventData { signature, .. }) = condition else {
            return None;
        };

        // TODO: Verify method signature hash against topic[0] to determine if
        // this log is valid enough to be decoded.

        let event = parse_event_signature(signature).ok()?;
        let event_log = event.decode_log(&self.inner.data).ok()?;

        Some(Arc::new(DecodedData::Event(event_log)))
    }
}

pub(crate) struct EvaluationContext<'a, D>
where
    D: EvaluableData + Send + Sync,
{
    pub(crate) data: Arc<D>,
    pub(crate) state: &'a State,
}

impl<'a, D> EvaluationContext<'a, D>
where
    D: EvaluableData + Send + Sync,
{
    pub(crate) fn new(data: D, state: &'a State) -> Self {
        Self {
            data: Arc::new(data),
            state,
        }
    }

    pub(crate) fn entry(&self, key: &CacheKey) -> Option<Arc<DecodedData>> {
        self.state.decoded_data.get(key).map(|v| v.clone())
    }

    pub(crate) fn insert(
        &self,
        key: &CacheKey,
        data: Arc<DecodedData>,
    ) -> Option<Arc<DecodedData>> {
        self.state.decoded_data.insert(key.clone(), data)
    }

    pub(crate) fn evaluate(&self, condition: &FilterCondition) -> bool {
        if !self.data.pre_evaluate(condition) {
            return false;
        }

        if !condition.needs_decoded_data() {
            return self.data.evaluate(condition, None);
        }

        let key = self.data.cache_key();
        let decoded = self.entry(&key).or_else(|| {
            self.data.decode_data(condition).inspect(|decoded| {
                self.insert(&key, decoded.clone());
            })
        });

        self.data.evaluate(condition, decoded)
    }
}
