/// ! Sieve is a real-time data streaming and filtering engine for ethereum & the superchain
pub(crate) mod config;
pub(crate) mod engine;
mod filter;
pub(crate) mod ingest;
pub(crate) mod network;
mod utils;

// Prelude module for convenient imports
pub mod prelude {
    pub use crate::engine::FilterEngine;
    pub use crate::filter::conditions::FilterNode;
    pub use crate::filter::{ArrayOps, FilterBuilder, LogicalOps, NumericOps, StringOps};
}

use crate::config::ChainConfig;
use engine::FilterEngine;
use filter::conditions::Filter;
use futures::StreamExt;
use ingest::Ingest;
use network::orchestrator::{ChainData, EthereumData};
use prelude::FilterNode;
use std::{sync::Arc, time::Duration};

pub struct Sieve {
    filters: Vec<Filter>,
    engine: Arc<FilterEngine>,
    ingest: Ingest,
}

pub struct FilterSubscription {}

pub enum Event {
    Transaction,
}

impl Sieve {
    pub async fn connect(chains: Vec<ChainConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let ingest = Ingest::new(chains).await;
        let engine = Arc::new(FilterEngine::new());
        let filters = Vec::new();

        let mut sieve = Self {
            engine,
            ingest,
            filters,
        };

        sieve.start_chain_processors().await?;
        Ok(sieve)
    }

    async fn start_chain_processors(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for chain in self.ingest.active_chains() {
            // Subscribe to chain data stream
            let mut stream = self.ingest.subscribe_stream(chain.clone()).await?;
            let engine = self.engine.clone();
            let filters = self.filters.clone();

            // Spawn chain-specific processor
            tokio::spawn(async move {
                while let Some(Ok(chain_data)) = stream.next().await {
                    // Get filters relevant to this chain
                    let chain_filters: Vec<_> = filters
                        .iter()
                        .filter(|f| f.which_chain() == chain)
                        .collect();

                    // Process data through relevant filters
                    for filter in chain_filters {
                        match chain_data {
                            ChainData::Ethereum(EthereumData::BlockHeader(ref block_header)) => {
                                if engine.evaluate_with_context(
                                    filter.filter_node(),
                                    block_header.clone(),
                                ) {
                                    // Emit event......
                                }
                            }
                            ChainData::Ethereum(_) => (),
                        }
                    }
                }
            });
        }
        Ok(())
    }

    pub fn subscribe(&self, filter: FilterNode) {
        unimplemented!();
    }

    pub fn submit_after<F, H>(&self, filter: FilterNode, handler: H, delay: Duration) {
        unimplemented!();
    }
}
