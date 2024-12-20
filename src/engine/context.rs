use super::{
    state::{CacheKey, State},
    DecodedData,
};
use crate::filter::{
    conditions::FilterCondition,
    evaluate::{Evaluable, EvaluableWithDecodedData},
};
use alloy_rpc_types::{Block, Transaction as RpcTransaction};
use std::sync::Arc;

#[allow(dead_code)]
pub(crate) trait EvaluableData {
    fn cache_key(&self) -> CacheKey;

    fn evaluate_condition(
        &self,
        condition: &FilterCondition,
        decoded: Option<Arc<DecodedData>>,
    ) -> bool;
}

impl EvaluableData for RpcTransaction {
    fn evaluate_condition(
        &self,
        condition: &FilterCondition,
        decoded: Option<Arc<DecodedData>>,
    ) -> bool {
        match condition {
            FilterCondition::Transaction(tx_condition) => tx_condition.evaluate(self, decoded),
            _ => false,
        }
    }

    fn cache_key(&self) -> CacheKey {
        CacheKey::ContractCall(self.inner.tx_hash().to_string())
    }
}

impl EvaluableData for Block {
    fn evaluate_condition(
        &self,
        condition: &FilterCondition,
        _decoded: Option<Arc<DecodedData>>,
    ) -> bool {
        match condition {
            FilterCondition::Block(tx_condition) => tx_condition.evaluate(self),
            _ => false,
        }
    }

    fn cache_key(&self) -> CacheKey {
        CacheKey::ContractCall(self.header.hash.to_string())
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

    pub(crate) fn entry(&self, key: CacheKey) -> Option<Arc<DecodedData>> {
        self.state.decoded_data.get(&key).map(|v| v.clone())
    }
}
