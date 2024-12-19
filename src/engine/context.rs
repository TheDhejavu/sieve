use super::{
    state::{CacheKey, State},
    DecodedContractCall, DecodedData,
};
use crate::filter::{
    conditions::FilterCondition,
    evaluate::{Evaluable, EvaluableWithDecodedData},
};
use alloy_rpc_types::{Block, Transaction as RpcTransaction};
use std::sync::Arc;

// Evaluable context
#[allow(dead_code)]
pub(crate) trait EvaluableContext {
    fn evaluate_condition(&self, condition: &FilterCondition) -> bool;
}

impl EvaluableContext for RpcTransaction {
    fn evaluate_condition(&self, condition: &FilterCondition) -> bool {
        match condition {
            FilterCondition::Transaction(tx_condition) => {
                tx_condition.evaluate(self, None::<&DecodedData>)
            }
            _ => false, // Non-transaction conditions always return false
        }
    }
}

impl EvaluableContext for Block {
    fn evaluate_condition(&self, condition: &FilterCondition) -> bool {
        match condition {
            FilterCondition::Block(block_condition) => block_condition.evaluate(self),
            _ => false,
        }
    }
}

#[allow(dead_code)]
pub(crate) struct EvaluationContext<'a, T> {
    pub(crate) data: Arc<T>,
    pub(crate) state: &'a State,
}

#[allow(dead_code)]
impl<'a, T> EvaluationContext<'a, T> {
    pub(crate) fn new(data: T, state: &'a State) -> Self {
        Self {
            data: Arc::new(data),
            state,
        }
    }

    pub(crate) fn entry(&self, key: CacheKey) -> Option<Arc<DecodedData>> {
        self.state.decoded_data.get(&key).map(|v| v.clone())
    }
}
