use alloy_consensus::TypedTransaction;
use dashmap::DashMap;
use std::sync::Arc;

use crate::filter::conditions::FilterNode;

struct Block {
    block_numer: u64,
    transactions: Vec<TypedTransaction>,
}

// Core cache for decoded data
#[derive(Hash, Eq, PartialEq, Clone)]
enum CacheKey {
    TransferData(String), // [key:txhash] -> decoded transfer
    ContractCall(String), // [key:txhash] -> decoded call
}

enum DecodedData {
    Transfer {
        method: String,
        to: String,
        amount: String,
    },
    Call {
        method: String,
        params: Vec<String>,
    },
}

// Shared state primarily for caching decoded data
struct State {
    decoded_data: DashMap<CacheKey, Arc<DecodedData>>,
}

// Evaluation context that gets passed to each evaluation
struct EvaluationContext<'a, B> {
    data: Arc<B>,
    state: &'a mut State,
}

impl<B> EvaluationContext<'_, B> {
    fn entry(&self, key: CacheKey) -> Option<Arc<DecodedData>> {
        if let Some(cached) = self.state.decoded_data.get(&key) {
            return Some(cached.clone());
        }
        None
    }
}

struct FilterEngine {
    state: State,
}

impl FilterEngine {
    fn new() -> Self {
        Self {
            state: State {
                decoded_data: DashMap::new(),
            },
        }
    }

    fn evaluate_block(&mut self, filter: &FilterNode, block: Block) -> bool {
        let ctx = EvaluationContext {
            data: Arc::new(block),
            state: &mut self.state,
        };

        true
    }

    fn evaluate_pool(&mut self, filter: &FilterNode, block: Block) -> bool {
        let ctx = EvaluationContext {
            data: Arc::new(block),
            state: &mut self.state,
        };

        true
    }
}
