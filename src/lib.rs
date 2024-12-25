pub mod cache;
/// ! Sieve is a real-time data streaming and filtering engine for ethereum & the superchain
///
pub mod config;
pub(crate) mod engine;
mod filter;
pub(crate) mod ingest;
pub(crate) mod network;
mod utils;

// prelude module for convenient imports
pub mod prelude {
    pub use crate::config;
    pub use crate::engine::FilterEngine;
    pub use crate::filter::conditions::{Filter, FilterNode};
    pub use crate::filter::{ArrayOps, FilterBuilder, LogicalOps, NumericOps, StringOps};
}

use crate::config::ChainConfig;
use engine::FilterEngine;
use filter::conditions::Filter;
use futures::StreamExt;
use ingest::Ingest;
use network::orchestrator::{ChainData, EthereumData};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

const BROADCAST_CHANNEL_SIZE: usize = 10_000;

pub struct Sieve {
    filters: HashMap<Filter, broadcast::Sender<ChainData>>,
    engine: Arc<FilterEngine>,
    ingest: Ingest,
}

impl Sieve {
    pub async fn connect(chains: Vec<ChainConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let ingest = Ingest::new(chains).await;
        let engine = Arc::new(FilterEngine::new());
        let filters = HashMap::new();

        let mut sieve = Self {
            engine,
            ingest,
            filters,
        };

        sieve.start_chain_processors().await?;
        Ok(sieve)
    }

    pub fn subscribe(
        &mut self,
        filter: Filter,
    ) -> Result<BroadcastStream<ChainData>, Box<dyn std::error::Error>> {
        // Get or create a broadcast channel for this filter
        let sender = self
            .filters
            .entry(filter.clone())
            .or_insert_with(|| broadcast::channel(BROADCAST_CHANNEL_SIZE).0);

        // New receiver returned wrapped in BroadcastStream
        let receiver = sender.subscribe();
        Ok(BroadcastStream::new(receiver))
    }

    async fn start_chain_processors(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for chain in self.ingest.active_chains() {
            let mut stream = self.ingest.subscribe_stream(chain.clone()).await?;
            let engine = self.engine.clone();
            let filters = self.filters.clone();

            tokio::spawn(async move {
                while let Some(Ok(chain_data)) = stream.next().await {
                    for (filter, sender) in filters.iter() {
                        if filter.which_chain() != chain {
                            continue;
                        }

                        match &chain_data {
                            ChainData::Ethereum(EthereumData::BlockHeader(block_header)) => {
                                if engine.evaluate_with_context(
                                    filter.filter_node().as_ref(),
                                    block_header.clone(),
                                ) {
                                    let _ = sender.send(chain_data.clone());
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

    fn subscriber_count(&self, filter: &Filter) -> usize {
        self.filters
            .get(filter)
            .map(|sender| sender.receiver_count())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::U256;
    use alloy_rpc_types::Header;
    use config::{Chain, ChainConfigBuilder};
    use filter::{FilterBuilder, LogicalOps, NumericOps, StringOps};
    use futures::StreamExt;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_multiple_subscribers() -> Result<(), Box<dyn std::error::Error>> {
        // 1. Chain Configuration
        let chains = vec![ChainConfigBuilder::builder()
            .rpc("https://ethereum-holesky-rpc.publicnode.com")
            .ws("wss://ws-mainnet.optimism.io")
            .gossipsub("/ip4/0.0.0.0/tcp/9000")
            .bootstrap_peers(vec!["/ip4/127.0.0.1/tcp/8000".to_string()])
            .chain(Chain::Ethereum)
            .build()];

        // 2. Connect to chains via `Sieve`
        let mut sieve = Sieve::connect(chains).await?;

        // 3. Create Filter
        let pool_filter = FilterBuilder::new().pool(|f| {
            f.any_of(|p| {
                p.value().gt(U256::from(100u64));
                p.from().starts_with("0xdead");
                p.to().exact("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
            });
        });

        // Create two subscribers
        let mut sub1 = sieve.subscribe(pool_filter.clone())?;
        let mut sub2 = sieve.subscribe(pool_filter.clone())?;

        // Verify we have two subscribers
        assert_eq!(sieve.subscriber_count(&pool_filter), 2);

        let sub1_task = tokio::spawn(async move {
            let msg = sub1.next().await;
            msg.and_then(|r| r.ok())
        });

        let sub2_task = tokio::spawn(async move {
            let msg = sub2.next().await;
            msg.and_then(|r| r.ok())
        });

        // Ensure subscribers are ready
        sleep(Duration::from_millis(100)).await;

        if let Some(sender) = sieve.filters.get(&pool_filter) {
            let header = Arc::new(Header::default());
            let test_data = ChainData::Ethereum(EthereumData::BlockHeader(header));
            sender.send(test_data.clone())?;

            // Wait for both subscribers to receive the message
            let (msg1, msg2) = tokio::join!(sub1_task, sub2_task);

            // Verify both subscribers received the same data
            assert!(msg1.is_ok());
            assert!(msg2.is_ok());

            let msg1 = msg1.unwrap();
            let msg2 = msg2.unwrap();

            assert!(msg1.is_some());
            assert!(msg2.is_some());
            assert_eq!(msg1, msg2);
        }
        Ok(())
    }
}
