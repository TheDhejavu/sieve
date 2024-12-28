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
    pub use crate::config::{Chain, ChainConfig, ChainConfigBuilder};
    pub use crate::engine::FilterEngine;
    pub use crate::filter::conditions::{Filter, FilterNode};
    pub use crate::filter::{ArrayOps, FilterBuilder, LogicalOps, NumericOps, StringOps};
    pub use crate::Sieve;
}

use crate::config::ChainConfig;
use alloy_rpc_types::{Block, BlockTransactions, Header, Transaction};
use engine::FilterEngine;
use filter::conditions::{EventType, Filter};
use futures::StreamExt;
use ingest::Ingest;
use network::orchestrator::{ChainData, EthereumData};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::{collections::HashMap, hash::DefaultHasher};
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

const BROADCAST_CHANNEL_SIZE: usize = 1_000;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    Transaction(Transaction),
    Pool(Transaction),
    Header(Header),
}

#[derive(Clone)]
pub enum SubscriptionType {
    Default, // Regular filtering  with `subscribe`
    Watch,   // Time-window based filtering with `watch_within`
}

#[derive(Clone)]
pub(crate) struct FilterGroup {
    sender: broadcast::Sender<Event>,
    sub_type: SubscriptionType,
    filters: Vec<Filter>,
}

impl FilterGroup {
    pub fn process_block(&self, block: &Block, engine: &FilterEngine) {
        match self.sub_type {
            SubscriptionType::Default => self.process_block_filters(block, engine),
            SubscriptionType::Watch => self.process_block_within(block, engine),
        }
    }

    pub fn process_transaction(&self, tx: &Transaction, engine: &FilterEngine) {
        match self.sub_type {
            SubscriptionType::Default => self.process_tx_filters(tx, engine),
            SubscriptionType::Watch => self.process_tx_within(tx, engine),
        }
    }

    fn process_block_filters(&self, block: &Block, engine: &FilterEngine) {
        for filter in &self.filters {
            if filter.event_type().is_none() {
                continue;
            }

            if filter.event_type() == Some(EventType::Transaction) {
                if let BlockTransactions::Full(transactions) = &block.transactions {
                    for tx in transactions {
                        if engine.evaluate_with_context(
                            filter.filter_node().as_ref(),
                            Arc::new(tx.clone()),
                        ) {
                            let _ = self.sender.send(Event::Transaction(tx.clone()));
                        }
                    }
                }
            }

            if filter.event_type() == Some(EventType::BlockHeader)
                && engine.evaluate_with_context(
                    filter.filter_node().as_ref(),
                    Arc::new(block.header.clone()),
                )
            {
                let _ = self.sender.send(Event::Header(block.header.clone()));
            }
        }
    }

    fn process_block_within(&self, _block: &Block, _engine: &FilterEngine) {
        todo!("Implement watch_within block processing")
    }

    fn process_tx_filters(&self, transaction: &Transaction, engine: &FilterEngine) {
        for filter in &self.filters {
            if filter.event_type().is_none() {
                continue;
            }

            if filter.event_type() == Some(EventType::Transaction)
                && engine.evaluate_with_context(
                    filter.filter_node().as_ref(),
                    Arc::new(transaction.clone()),
                )
            {
                let _ = self.sender.send(Event::Transaction(transaction.clone()));
            }
        }
    }

    fn process_tx_within(&self, _tx: &Transaction, _engine: &FilterEngine) {
        todo!("Implement watch_within transaction processing")
    }
}


#[derive(Clone)]
pub struct Sieve {
    filters: Arc<RwLock<HashMap<u64, FilterGroup>>>,
    engine: Arc<FilterEngine>,
    ingest: Arc<Ingest>,
}


impl Sieve {
    pub async fn connect(chains: Vec<ChainConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let ingest = Arc::new(Ingest::new(chains).await);
        let engine = Arc::new(FilterEngine::new());
        let filters = Arc::new(RwLock::new(HashMap::new()));

        let sieve = Self {
            engine,
            ingest,
            filters,
        };

        sieve.start_chain_processors().await?;
        Ok(sieve)
    }

    pub async fn subscribe(
        &self,
        filter: Filter,
    ) -> Result<BroadcastStream<Event>, Box<dyn std::error::Error>> {
        let mut filters = self.filters.write().await;
        let group = filters.entry(filter.id()).or_insert_with(|| FilterGroup {
            sender: broadcast::channel(BROADCAST_CHANNEL_SIZE).0,
            sub_type: SubscriptionType::Default,
            filters: vec![filter],
        });

        let receiver = group.sender.subscribe();
        Ok(BroadcastStream::new(receiver))
    }

    pub async fn subscribe_all(
        &self,
        filters: Vec<Filter>,
    ) -> Result<BroadcastStream<Event>, Box<dyn std::error::Error>> {
        let mut filter_entries = self.filters.write().await;
        let mut hasher = DefaultHasher::new();
        for filter in &filters {
            filter.hash(&mut hasher);
        }
        let group_id = hasher.finish();

        let group = filter_entries
            .entry(group_id)
            .or_insert_with(|| FilterGroup {
                sender: broadcast::channel(BROADCAST_CHANNEL_SIZE).0,
                sub_type: SubscriptionType::Default,
                filters,
            });

        let receiver = group.sender.subscribe();
        Ok(BroadcastStream::new(receiver))
    }

    async fn start_chain_processors(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut processor_handles = Vec::new();

        for chain in self.ingest.active_chains() {
            let stream = self.ingest.subscribe_stream(chain.clone()).await?;
            let engine = self.engine.clone();
            let filters = self.filters.clone();

            let handle = tokio::spawn(async move {
                Self::run_chain_processor(stream, engine, filters).await;
            });

            processor_handles.push(handle);
        }

        tokio::spawn(async move {
            for handle in processor_handles {
                if let Err(e) = handle.await {
                    eprintln!("Chain processor failed: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn run_chain_processor(
        mut stream: impl StreamExt<Item = Result<ChainData, BroadcastStreamRecvError>> + Unpin,
        engine: Arc<FilterEngine>,
        filters: Arc<RwLock<HashMap<u64, FilterGroup>>>,
    ) {
        while let Some(Ok(chain_data)) = stream.next().await {
            let filters = filters.read().await;

            match chain_data {
                ChainData::Ethereum(EthereumData::Block(block)) => {
                    for (_, filter_group) in filters.iter() {
                        filter_group.process_block(&block, &engine);
                    }
                }
                ChainData::Ethereum(EthereumData::TransactionPool(tx)) => {
                    for (_, filter_group) in filters.iter() {
                        filter_group.process_transaction(&tx, &engine);
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    async fn subscriber_count(&self, filter: &Filter) -> usize {
        self.filters
            .read()
            .await
            .get(&filter.id())
            .map(|filter_group| filter_group.sender.receiver_count())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::U256;
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
        let sieve = Sieve::connect(chains).await?;

        // 3. Create Filter
        let pool_filter = FilterBuilder::new().pool(|f| {
            f.any_of(|p| {
                p.value().gt(U256::from(100u64));
                p.from().starts_with("0xdead");
                p.to().exact("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
            });
        });

        // Create two subscribers
        let mut sub1 = sieve.subscribe(pool_filter.clone()).await?;
        let mut sub2 = sieve.subscribe(pool_filter.clone()).await?;

        // Verify we have two subscribers
        assert_eq!(sieve.subscriber_count(&pool_filter).await, 2);

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

        if let Some(filter_group) = sieve.filters.read().await.get(&pool_filter.id()) {
            let test_data = Event::Header(Header::default());
            filter_group.sender.send(test_data.clone())?;

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
