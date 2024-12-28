use super::{
    state::{CacheKey, State},
    DecodedData, EvaluableData,
};
use crate::filter::conditions::FilterCondition;
use std::sync::Arc;

/// Context for evaluating filter conditions against data, it  handles caching
/// of decoded data and evaluation state
pub(crate) struct EvaluationContext<'a, D>
where
    D: EvaluableData + Send + Sync,
{
    /// Data to evaluate against conditions
    pub(crate) data: Arc<D>,
    /// Shared state containing cached decoded data
    pub(crate) state: &'a State,
}

impl<'a, D> EvaluationContext<'a, D>
where
    D: EvaluableData + Send + Sync,
{
    // Creates a new [`EvaluationContext`] for evaluation
    pub(crate) fn new(data: Arc<D>, state: &'a State) -> Self {
        Self { data, state }
    }

    /// Gets cached decoded data for a given key
    pub(crate) fn entry(&self, key: &CacheKey) -> Option<Arc<DecodedData>> {
        self.state.decoded_data.get(key).map(|v| v.clone())
    }

    /// Inserts decoded data into the cache
    pub(crate) fn insert(
        &self,
        key: &CacheKey,
        data: Arc<DecodedData>,
    ) -> Option<Arc<DecodedData>> {
        self.state.decoded_data.insert(key.clone(), data)
    }

    /// Evaluates a filter condition against the data
    pub(crate) fn evaluate(&self, condition: &FilterCondition) -> bool {
        // Skip full evaluation if pre-evaluation fails
        if !self.data.pre_evaluate(condition) {
            return false;
        }

        // If no decoded data needed, evaluate directly
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
