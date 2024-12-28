/// ! Sieve is a real-time data streaming and filtering engine for ethereum & the superchain
pub(crate) mod cache;
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
use dashmap::DashMap;
use engine::FilterEngine;
use filter::conditions::{EventType, Filter};
use futures::StreamExt;
use ingest::Ingest;
use network::orchestrator::{ChainData, EthereumData};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{collections::HashMap, hash::DefaultHasher};
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::BroadcastStream;

const BROADCAST_CHANNEL_SIZE: usize = 1_000;

/// A single event that matched a filter
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    /// A transaction included in a block
    Transaction(Transaction),
    /// A transaction from the mempool
    Pool(Transaction),
    /// A block header
    Header(Header),
}

/// A window-based event that contains either matched events or a timeout
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EventWindow {
    /// All filter conditions were met within the time window
    Match(Vec<Event>),
    /// Time window expired before all conditions were met
    Timeout,
}

/// [`SubscriptionType`] is a type of subscription for filter groups
#[derive(Clone)]
pub enum SubscriptionType {
    /// Regular filtering with `subscribe` - emits events as they occur
    Default,
    /// Window-based filtering with `watch_within` - collects events until all conditions met
    WatchWindow,
}

/// [`GroupSender`] type based on subscription type
#[derive(Clone)]
pub enum GroupSender {
    /// Sender for individual events
    Default(broadcast::Sender<Event>),
    /// Sender for window-based events
    Watch(broadcast::Sender<EventWindow>),
}

/// [`FilterGroup`] handles the evaluation of filters and sending of events.
/// It maintains a group of related filters and handles events based on its subscription type.
#[derive(Clone)]
pub(crate) struct FilterGroup {
    /// Unique identifier for this filter group
    group_id: u64,
    /// Set of filters to evaluate
    filters: Vec<Filter>,
    /// Event sender based on subscription type
    sender: GroupSender,
    /// Type of subscription
    sub_type: SubscriptionType,
}

impl FilterGroup {
    /// Creates a new [`FilterGroup`] with the specified parameters
    fn new(group_id: u64, filters: Vec<Filter>, sub_type: SubscriptionType) -> Self {
        let sender = match sub_type {
            SubscriptionType::Default => {
                GroupSender::Default(broadcast::channel(BROADCAST_CHANNEL_SIZE).0)
            }
            SubscriptionType::WatchWindow => {
                GroupSender::Watch(broadcast::channel(BROADCAST_CHANNEL_SIZE).0)
            }
        };

        Self {
            group_id,
            filters,
            sender,
            sub_type,
        }
    }

    /// Evaluates a block against all filters in the group
    fn evaluate_block(&self, block: &Block, engine: &FilterEngine) -> Vec<(u64, Event)> {
        let mut events = Vec::new();

        for filter in &self.filters {
            // 1. Try to process header
            if filter.event_type() == Some(EventType::BlockHeader)
                && engine.evaluate_with_context(
                    filter.filter_node().as_ref(),
                    Arc::new(block.header.clone()),
                )
            {
                events.push((filter.id(), Event::Header(block.header.clone())));
            }

            // 2. Try to process transactions
            if let BlockTransactions::Full(transactions) = &block.transactions {
                for tx in transactions {
                    if filter.event_type() == Some(EventType::Transaction)
                        && engine.evaluate_with_context(
                            filter.filter_node().as_ref(),
                            Arc::new(tx.clone()),
                        )
                    {
                        events.push((filter.id(), Event::Transaction(tx.clone())));
                    }
                }
            }
        }

        events
    }
    /// Evaluates a mempool transaction against this group's filters
    fn evaluate_transaction(&self, tx: &Transaction, engine: &FilterEngine) -> Vec<(u64, Event)> {
        let mut events = Vec::new();

        for filter in &self.filters {
            if filter.event_type() == Some(EventType::Transaction)
                && engine.evaluate_with_context(filter.filter_node().as_ref(), Arc::new(tx.clone()))
            {
                events.push((filter.id(), Event::Pool(tx.clone())));
            }
        }

        events
    }
    /// Sends a single event to subscribers if this is a default subscription
    fn send_event(&self, event: Event) {
        if let GroupSender::Default(sender) = &self.sender {
            let _ = sender.send(event);
        }
    }

    /// Sends a window event to subscribers if this is a watch window subscription
    fn send_window_event(&self, event: EventWindow) {
        if let GroupSender::Watch(sender) = &self.sender {
            let _ = sender.send(event);
        }
    }
}

/// Window manages the state of time-based event matching.
/// It tracks which filters have matched and collects events until all conditions are met.
#[derive(Clone)]
pub(crate) struct Window {
    /// Time when this window expires
    expires_at: Instant,
    /// Events matched against filters, None means filter not yet matched and
    /// Index in the vector corresponds to the original filter position.
    matched_events: Vec<Option<Event>>,
    /// Count of remaining unmatched filters
    remaining_matches: usize,
    /// Original filter IDs in order
    filter_ids: Vec<u64>,
}

impl Window {
    /// Creates a new [`Window`] instance with a set duration
    fn new(filter_ids: Vec<u64>, duration: Duration) -> Self {
        let size = filter_ids.len();
        Self {
            expires_at: Instant::now() + duration,
            matched_events: vec![None; size],
            remaining_matches: size,
            filter_ids,
        }
    }
    /// Checks if this window has expired
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Attempts to match an event against an unmatched filter
    fn try_match(&mut self, filter_id: u64, event: Event) -> Option<Vec<Event>> {
        if self.is_expired() {
            return None;
        }

        if let Some(pos) = self.filter_ids.iter().position(|id| *id == filter_id) {
            if self.matched_events[pos].is_none() {
                self.matched_events[pos] = Some(event);
                self.remaining_matches -= 1;

                if self.remaining_matches == 0 {
                    return Some(
                        self.matched_events
                            .iter()
                            .filter_map(|e| e.clone())
                            .collect(),
                    );
                }
            }
        }
        None
    }
}

/// [`WindowManager`] handles the lifecycle of all active time windows.
/// It creates windows, processes events, and handles expiration.
pub(crate) struct WindowManager {
    /// Active windows mapped by group ID
    windows: DashMap<u64, Window>,
    /// How often to check for expired windows
    purge_interval: Duration,
}

impl WindowManager {
    /// Creates a new [`WindowManager`] instance
    pub(crate) fn new(purge_interval: Duration) -> Self {
        let manager = Self {
            windows: DashMap::new(),
            purge_interval,
        };
        manager.start_periodic_purge();
        manager
    }

    /// Creates a new window for a filter group
    pub(crate) fn create_window(&self, group_id: u64, filter_ids: Vec<u64>, duration: Duration) {
        let window = Window::new(filter_ids, duration);
        self.windows.insert(group_id, window);
    }

    /// Processes a batch of events for a window
    pub(crate) fn process_events(
        &self,
        group_id: u64,
        events: Vec<(u64, Event)>,
        group: &FilterGroup,
    ) {
        if let Some(mut window) = self.windows.get_mut(&group_id) {
            if window.is_expired() {
                group.send_window_event(EventWindow::Timeout);
                self.windows.remove(&group_id);
                return;
            }

            for (filter_id, event) in events {
                if let Some(matched_events) = window.try_match(filter_id, event) {
                    group.send_window_event(EventWindow::Match(matched_events));
                    self.windows.remove(&group_id);
                    return;
                }
            }
        }
    }

    /// Starts background task to periodically remove expired windows
    fn start_periodic_purge(&self) {
        let windows = self.windows.clone();
        let interval = self.purge_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                windows.retain(|_, window| !window.is_expired());
            }
        });
    }
}

/// Sieve is the main entry point for the filtering engine.
/// It coordinates filter evaluation, event processing, and window management.
#[derive(Clone)]
pub struct Sieve {
    /// Active filter groups
    filters: Arc<RwLock<HashMap<u64, FilterGroup>>>,
    /// Filter evaluation engine
    engine: Arc<FilterEngine>,
    /// Data ingestion handler
    ingest: Arc<Ingest>,
    /// Window management system
    window_manager: Arc<WindowManager>,
}

impl Sieve {
    /// Connects to specified chains and initializes the filtering engine
    ///
    /// # Arguments
    /// * `chains` - List of chain configurations to connect to
    pub async fn connect(chains: Vec<ChainConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let ingest = Arc::new(Ingest::new(chains).await);
        let engine = Arc::new(FilterEngine::new());
        let filters = Arc::new(RwLock::new(HashMap::new()));
        let window_manager = Arc::new(WindowManager::new(Duration::from_secs(1)));

        let sieve = Self {
            engine,
            ingest,
            filters,
            window_manager,
        };

        sieve.start_chain_processors().await?;
        Ok(sieve)
    }

    /// Subscribes to events matching a single filter
    ///
    /// # Arguments
    /// * `filter` - Filter to match events against
    ///
    /// # Returns
    /// Stream of matching events
    pub async fn subscribe(
        &self,
        filter: Filter,
    ) -> Result<BroadcastStream<Event>, Box<dyn std::error::Error>> {
        let mut filters = self.filters.write().await;
        let group = FilterGroup::new(filter.id(), vec![filter], SubscriptionType::Default);

        let group = filters.entry(group.group_id).or_insert_with(|| group);

        let receiver = match &group.sender {
            GroupSender::Default(sender) => sender.subscribe(),
            _ => unreachable!(),
        };

        Ok(BroadcastStream::new(receiver))
    }

    /// Creates a time-window based subscription
    ///
    /// # Arguments
    /// * `filters` - List of filters to match
    /// * `duration` - How long to wait for all conditions
    ///
    /// # Returns
    /// Stream of window events (matches or timeout)
    pub async fn watch_within(
        &self,
        filters: Vec<Filter>,
        duration: Duration,
    ) -> Result<BroadcastStream<EventWindow>, Box<dyn std::error::Error>> {
        let mut filter_entries = self.filters.write().await;

        let mut hasher = DefaultHasher::new();
        for filter in &filters {
            filter.hash(&mut hasher);
        }
        let group_id = hasher.finish();
        let group = FilterGroup::new(group_id, filters.clone(), SubscriptionType::WatchWindow);

        let receiver = match &group.sender {
            GroupSender::Watch(sender) => sender.subscribe(),
            _ => unreachable!(),
        };

        let filter_ids: Vec<u64> = filters.iter().map(|f| f.id()).collect();
        self.window_manager
            .create_window(group_id, filter_ids, duration);

        filter_entries.insert(group_id, group);
        Ok(BroadcastStream::new(receiver))
    }

    /// Processes a block through all filter groups
    async fn process_block(&self, block: &Block) {
        let filters = self.filters.read().await;

        for group in filters.values() {
            let matches = group.evaluate_block(block, &self.engine);

            match group.sub_type {
                SubscriptionType::Default => {
                    for (_, event) in matches {
                        group.send_event(event);
                    }
                }
                SubscriptionType::WatchWindow => {
                    self.window_manager
                        .process_events(group.group_id, matches, group);
                }
            }
        }
    }

    /// Processes a transaction through all filter groups
    async fn process_transaction(&self, tx: &Transaction) {
        let filters = self.filters.read().await;

        for group in filters.values() {
            let matches = group.evaluate_transaction(tx, &self.engine);

            match group.sub_type {
                SubscriptionType::Default => {
                    for (_, event) in matches {
                        group.send_event(event);
                    }
                }
                SubscriptionType::WatchWindow => {
                    self.window_manager
                        .process_events(group.group_id, matches, group);
                }
            }
        }
    }
    /// Starts background tasks for processing chain data
    async fn start_chain_processors(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut processor_handles = Vec::new();
        for chain in self.ingest.active_chains() {
            let mut stream = self.ingest.subscribe_stream(chain.clone()).await?;
            let sieve = self.clone();

            let handle = tokio::spawn(async move {
                while let Some(Ok(chain_data)) = stream.next().await {
                    match chain_data {
                        ChainData::Ethereum(EthereumData::Block(block)) => {
                            sieve.process_block(&block).await;
                        }
                        ChainData::Ethereum(EthereumData::TransactionPool(tx)) => {
                            sieve.process_transaction(&tx).await;
                        }
                    }
                }
            });

            processor_handles.push(handle);
        }

        Ok(())
    }
    /// Gets the current number of subscribers for a filter
    #[allow(dead_code)]
    async fn subscriber_count(&self, filter: &Filter) -> usize {
        self.filters
            .read()
            .await
            .get(&filter.id())
            .map(|filter_group| match &filter_group.sender {
                GroupSender::Default(event_sender) => event_sender.receiver_count(),
                GroupSender::Watch(sender) => sender.receiver_count(),
            })
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
            let sender = match &filter_group.sender {
                GroupSender::Default(sender) => sender,
                _ => unreachable!(),
            };

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
