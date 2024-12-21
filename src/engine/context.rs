use super::{
    state::{CacheKey, State},
    DecodedData,
};
use crate::{
    filter::{
        conditions::{BlockHeaderCondition, EventCondition, FilterCondition, TransactionCondition},
        evaluate::EvaluableWithCondition,
    },
    utils::decoder::{parse_event_signature, DecodedLog},
};
use alloy_rpc_types::{Header, Log, Transaction as RpcTransaction};
use std::sync::Arc;

#[allow(dead_code)]
pub(crate) trait EvaluableData {
    type Item: Into<DecodedData>;
    type Condition;

    fn cache_key(&self) -> CacheKey;

    fn matches_condition(condition: &FilterCondition) -> bool;
    fn as_condition(condition: &FilterCondition) -> Option<&Self::Condition>;

    fn evaluate_condition(
        &self,
        condition: &FilterCondition,
        decoded: Option<Arc<DecodedData>>,
    ) -> bool
    where
        Self: EvaluableWithCondition<Self::Condition>;

    fn decode_data(&self, condition: &Self::Condition) -> Self::Item;
}

impl EvaluableData for RpcTransaction {
    type Item = DecodedData;
    type Condition = TransactionCondition;

    fn matches_condition(condition: &FilterCondition) -> bool {
        matches!(condition, FilterCondition::Transaction(_))
    }

    fn as_condition(condition: &FilterCondition) -> Option<&Self::Condition> {
        match condition {
            FilterCondition::Transaction(c) => Some(c),
            _ => None,
        }
    }

    fn evaluate_condition(
        &self,
        condition: &FilterCondition,
        decoded: Option<Arc<DecodedData>>,
    ) -> bool {
        if let Some(tx_condition) = Self::as_condition(condition) {
            self.evaluate(tx_condition, decoded)
        } else {
            false
        }
    }

    fn cache_key(&self) -> CacheKey {
        CacheKey::ContractCall(self.inner.tx_hash().to_string())
    }

    fn decode_data(&self, _condition: &Self::Condition) -> Self::Item {
        unimplemented!()
    }
}

impl EvaluableData for Header {
    type Item = DecodedData;
    type Condition = BlockHeaderCondition;

    fn matches_condition(condition: &FilterCondition) -> bool {
        matches!(condition, FilterCondition::BlockHeader(_))
    }

    fn as_condition(condition: &FilterCondition) -> Option<&Self::Condition> {
        match condition {
            FilterCondition::BlockHeader(c) => Some(c),
            _ => None,
        }
    }

    fn evaluate_condition(
        &self,
        condition: &FilterCondition,
        decoded: Option<Arc<DecodedData>>,
    ) -> bool {
        if let Some(block_header_condition) = Self::as_condition(condition) {
            self.evaluate(block_header_condition, decoded)
        } else {
            false
        }
    }

    fn cache_key(&self) -> CacheKey {
        CacheKey::ContractCall(self.hash.to_string())
    }

    fn decode_data(&self, _condition: &Self::Condition) -> Self::Item {
        unimplemented!()
    }
}

impl EvaluableData for Log {
    type Item = Option<DecodedLog>;
    type Condition = EventCondition;

    fn matches_condition(condition: &FilterCondition) -> bool {
        matches!(condition, FilterCondition::Event(_))
    }

    fn as_condition(condition: &FilterCondition) -> Option<&Self::Condition> {
        match condition {
            FilterCondition::Event(c) => Some(c),
            _ => None,
        }
    }

    fn evaluate_condition(
        &self,
        condition: &FilterCondition,
        decoded: Option<Arc<DecodedData>>,
    ) -> bool {
        if let Some(event_condition) = Self::as_condition(condition) {
            self.evaluate(event_condition, decoded)
        } else {
            false
        }
    }

    fn cache_key(&self) -> CacheKey {
        CacheKey::Event(format!("{:?}", self.log_index))
    }

    fn decode_data(&self, condition: &Self::Condition) -> Self::Item {
        if let EventCondition::EventMatch { signature, .. } = condition {
            let result = parse_event_signature(signature);
            if let Ok(event) = result {
                return event.decode_log(&self.inner.data).ok();
            }
        }
        None
    }
}

impl From<Option<DecodedLog>> for DecodedData {
    fn from(val: Option<DecodedLog>) -> Self {
        DecodedData::Event(val.unwrap())
    }
}

#[allow(dead_code)]
pub(crate) struct EvaluationContext<'a, D>
where
    D: EvaluableData + Send + Sync,
{
    pub(crate) data: Arc<D>,
    pub(crate) state: &'a State,
}

#[allow(dead_code)]
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

    pub(crate) fn insert(&self, key: &CacheKey, data: DecodedData) -> Option<Arc<DecodedData>> {
        self.state.decoded_data.insert(key.clone(), data.into())
    }
}
