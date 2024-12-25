use crate::{config::ChainConfig, filter::conditions::FilterNode};

struct Ingest {
    filters: Vec<FilterNode>,
}

impl Ingest {
    pub(crate) fn new(config: ChainConfig) -> Self {
        // 1. Create the RPC instance for all chains here

        // 2. Create Orchestrator for all chains and consolidate everything into
        // distinct stream for each chain.

        // 3. Listen to each streams (blocks, pending transactions e.t.c)

        // 4. Evaluate stream against filters and emit events accordinly
        Self {
            filters: Vec::new(),
        }
    }
}
