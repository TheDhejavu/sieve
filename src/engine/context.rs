use super::{
    state::{CacheKey, State},
    DecodedData, EvaluableData,
};
use crate::filter::conditions::FilterCondition;
use std::sync::Arc;

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
