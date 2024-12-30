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
use alloy_network::{AnyHeader, AnyRpcBlock, AnyRpcTransaction, BlockResponse};
use alloy_rpc_types::{BlockTransactions, Header};
use config::Chain;
use dashmap::DashMap;
use engine::FilterEngine;
use filter::conditions::{EventType, Filter};
use futures::StreamExt;
use ingest::{Ingest, IngestError, IngestGateway};
use network::orchestrator::{AnyRPCNetwork, ChainData};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{collections::HashMap, hash::DefaultHasher};
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::BroadcastStream;

const BROADCAST_CHANNEL_SIZE: usize = 1_000;

#[derive(Error, Debug)]
pub enum SieveError {
    #[error("Failed to connect to chain: {0}")]
    ConnectionError(String),

    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    #[error("Ingest error: {0}")]
    IngestError(#[from] IngestError),

    #[error("Invalid window duration: {0}")]
    InvalidWindowDuration(String),
}

/// A single event that matched a filter.
// In the meantime events are RPC-specific data, in order to be more specific
// we will have to manually create events for each data types by mapping RPC or any other
// network data types to a unified event type.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    /// A transaction included in a block
    Transaction(AnyRpcTransaction),
    /// A transaction from the mempool
    Pool(AnyRpcTransaction),
    /// A block header
    Header(Header<AnyHeader>),
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
    fn evaluate_block(
        &self,
        block: &AnyRpcBlock,
        engine: &FilterEngine,
        chain: &Chain,
    ) -> Vec<(u64, Event)> {
        let mut events = Vec::new();

        for filter in &self.filters {
            if filter.chain() != chain {
                continue;
            }

            // 1. Try to process header
            if filter.event_type() == Some(EventType::BlockHeader)
                && engine.evaluate_with_context(
                    filter.filter_node().as_ref(),
                    Arc::new(block.header.clone()),
                )
            {
                events.push((filter.id(), Event::Header(block.header.clone())));
            }

            let txs = block.transactions();
            // 2. Try to process transactions
            if let BlockTransactions::Full(transactions) = txs {
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
    fn evaluate_transaction(
        &self,
        tx: &AnyRpcTransaction,
        engine: &FilterEngine,
        chain: &Chain,
    ) -> Vec<(u64, Event)> {
        let mut events = Vec::new();

        for filter in &self.filters {
            if filter.chain() != chain {
                continue;
            }

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
#[derive(Debug)]
pub(crate) struct Window {
    /// Time when this window expires
    expires_at: Instant,
    /// Events matched against filters, None means filter not yet matched and
    /// Index in the vector corresponds to the original filter position.
    matched_events: Vec<Option<Event>>,
    /// Count of remaining unmatched filters
    remaining_matches: AtomicU64,
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
            remaining_matches: AtomicU64::new(size as u64),
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
                let remaining_matches = self.remaining_matches.fetch_sub(1, Ordering::Relaxed);

                if remaining_matches == 1 {
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
    windows: Arc<DashMap<u64, Window>>,
    /// How often to check for expired windows
    purge_interval: Duration,
    /// Callback for handling expired windows
    on_expired: Arc<dyn Fn(u64) + Send + Sync>,
}

impl WindowManager {
    /// Creates a new [`WindowManager`] instance
    pub(crate) fn new(
        purge_interval: Duration,
        on_expired: impl Fn(u64) + Send + Sync + 'static,
    ) -> Self {
        let manager = Self {
            windows: Arc::new(DashMap::new()),
            purge_interval,
            on_expired: Arc::new(on_expired),
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
                drop(window); // drop mutable lock to prevent dead-lock.
                self.windows.remove(&group_id);
                return;
            }

            for (filter_id, event) in events {
                if let Some(matched_events) = window.try_match(filter_id, event) {
                    group.send_window_event(EventWindow::Match(matched_events));
                    drop(window);
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
        let on_expired = self.on_expired.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                windows.retain(|group_id, window| {
                    if window.is_expired() {
                        (on_expired)(*group_id);
                        false
                    } else {
                        true
                    }
                });
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
    ingest: Arc<dyn IngestGateway>,
    /// Window management system
    window_manager: Arc<WindowManager>,
}

impl Sieve {
    /// Connects to specified chains and initializes the filtering engine
    ///
    /// # Arguments
    /// * `chains` - List of chain configurations to connect to
    pub async fn connect(chains: Vec<ChainConfig>) -> Result<Self, SieveError> {
        let ingest = Arc::new(Ingest::new(chains).await);
        let engine = Arc::new(FilterEngine::new());
        let filters = Arc::new(RwLock::new(HashMap::new()));

        let window_manager = Arc::new(WindowManager::new(
            Duration::from_secs(1),
            Self::handle_window_expiration(filters.clone()),
        ));

        let sieve = Self {
            engine,
            ingest,
            filters,
            window_manager,
        };

        sieve
            .start_chain_processors()
            .await
            .map_err(|e| SieveError::ConnectionError(e.to_string()))?;

        Ok(sieve)
    }

    /// Subscribes to events matching a single filter
    ///
    /// # Arguments
    /// * `filter` - Filter to match events against
    ///
    /// # Returns
    /// Stream of matching events
    pub async fn subscribe(&self, filter: Filter) -> Result<BroadcastStream<Event>, SieveError> {
        let mut filters = self.filters.write().await;
        let group = FilterGroup::new(filter.id(), vec![filter], SubscriptionType::Default);

        let group = filters.entry(group.group_id).or_insert_with(|| group);

        let receiver = match &group.sender {
            GroupSender::Default(sender) => sender.subscribe(),
            _ => {
                return Err(SieveError::SubscriptionError(
                    "Invalid subscription type".to_string(),
                ))
            }
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
    ) -> Result<BroadcastStream<EventWindow>, SieveError> {
        if duration.is_zero() {
            return Err(SieveError::InvalidWindowDuration(
                "Window duration cannot be zero".to_string(),
            ));
        }

        let mut filter_entries = self.filters.write().await;

        let mut hasher = DefaultHasher::new();
        for filter in &filters {
            filter.hash(&mut hasher);
        }
        let group_id = hasher.finish();
        let group = FilterGroup::new(group_id, filters.clone(), SubscriptionType::WatchWindow);

        let receiver = match &group.sender {
            GroupSender::Watch(sender) => sender.subscribe(),
            _ => {
                return Err(SieveError::SubscriptionError(
                    "Invalid subscription type".to_string(),
                ))
            }
        };

        let filter_ids: Vec<u64> = filters.iter().map(|f| f.id()).collect();
        self.window_manager
            .create_window(group_id, filter_ids, duration);

        filter_entries.insert(group_id, group);
        Ok(BroadcastStream::new(receiver))
    }

    /// Processes a block through all filter groups
    async fn process_any_rpc_block(&self, block: &AnyRpcBlock, chain: &Chain) {
        let filters = self.filters.read().await;

        for group in filters.values() {
            let matches = group.evaluate_block(block, &self.engine, chain);
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
    async fn process_any_rpc_transaction(&self, tx: &AnyRpcTransaction, chain: &Chain) {
        let filters = self.filters.read().await;

        for group in filters.values() {
            let matches = group.evaluate_transaction(tx, &self.engine, chain);

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
    async fn start_chain_processors(&self) -> Result<(), SieveError> {
        let mut processor_handles = Vec::new();
        for chain in self.ingest.active_chains() {
            let mut stream = self.ingest.subscribe_stream(chain.clone()).await?;
            let sieve = self.clone();

            let handle = tokio::spawn(async move {
                while let Some(Ok(chain_data)) = stream.next().await {
                    match chain_data {
                        ChainData::AnyRPCNetwork(AnyRPCNetwork::Block(block)) => {
                            sieve.process_any_rpc_block(&block, &chain).await;
                        }
                        ChainData::AnyRPCNetwork(AnyRPCNetwork::TransactionPool(tx)) => {
                            sieve.process_any_rpc_transaction(&tx, &chain).await;
                        }
                    }
                }
            });

            processor_handles.push(handle);
        }

        Ok(())
    }
    /// Handle window expiration by sending timeout event
    fn handle_window_expiration(
        filters: Arc<RwLock<HashMap<u64, FilterGroup>>>,
    ) -> impl Fn(u64) + Send + Sync + 'static {
        move |group_id| {
            if let Ok(filters_rw) = filters.try_read() {
                if let Some(group) = filters_rw.get(&group_id) {
                    group.send_window_event(EventWindow::Timeout);
                }
            }
        }
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
    use std::sync::Mutex;

    use super::*;
    use alloy_consensus::Transaction;
    use alloy_primitives::U256;
    use alloy_rpc_types::Block;
    use config::Chain;
    use filter::{FilterBuilder, NumericOps};
    use futures::StreamExt;
    use ingest::IngestError;
    use tokio::time::Duration;
    use utils::test_utils::generate_random_transaction;

    pub struct MockIngest {
        chain_states: Arc<Mutex<HashMap<Chain, broadcast::Sender<ChainData>>>>,
    }

    impl MockIngest {
        pub fn new() -> Self {
            let mut chain_states = HashMap::new();
            chain_states.insert(Chain::Ethereum, broadcast::channel(32).0);
            Self {
                chain_states: Arc::new(Mutex::new(chain_states)),
            }
        }

        pub fn mock_chain_data(
            &self,
            chain: Chain,
            data: ChainData,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let states = self.chain_states.lock().unwrap();
            if let Some(sender) = states.get(&chain) {
                sender.send(data)?;
            }
            Ok(())
        }

        fn subscribe(&self, chain: Chain) -> Result<broadcast::Receiver<ChainData>, IngestError> {
            let mut states = self.chain_states.lock().unwrap();
            let sender = states
                .entry(chain.clone())
                .or_insert_with(|| broadcast::channel(32).0);
            Ok(sender.subscribe())
        }
    }

    #[async_trait::async_trait]
    impl IngestGateway for MockIngest {
        async fn subscribe_stream(
            &self,
            chain: Chain,
        ) -> Result<BroadcastStream<ChainData>, IngestError> {
            let receiver = self.subscribe(chain)?;
            Ok(BroadcastStream::new(receiver))
        }

        fn is_active(&self, chain: &Arc<Chain>) -> bool {
            self.chain_states.lock().unwrap().contains_key(chain)
        }

        async fn stop_chain(&mut self, chain: Arc<Chain>) -> Result<(), IngestError> {
            self.chain_states.lock().unwrap().remove(&chain);
            Ok(())
        }

        async fn stop_all(&mut self) -> Result<(), IngestError> {
            self.chain_states.lock().unwrap().clear();
            Ok(())
        }

        fn active_chains(&self) -> Vec<Chain> {
            self.chain_states.lock().unwrap().keys().cloned().collect()
        }
    }

    async fn setup_test_sieve() -> Result<(Sieve, Arc<MockIngest>), Box<dyn std::error::Error>> {
        let mock_ingest = Arc::new(MockIngest::new());
        let engine = Arc::new(FilterEngine::new());
        let filters: Arc<RwLock<HashMap<u64, FilterGroup>>> = Arc::new(RwLock::new(HashMap::new()));

        let clone_filters = filters.clone();
        let window_manager = Arc::new(WindowManager::new(
            Duration::from_secs(1),
            move |group_id| {
                if let Ok(filters_rw) = clone_filters.try_read() {
                    if let Some(group) = filters_rw.get(&group_id) {
                        group.send_window_event(EventWindow::Timeout);
                    }
                }
            },
        ));

        let sieve = Sieve {
            engine,
            ingest: mock_ingest.clone(),
            filters,
            window_manager,
        };

        sieve.start_chain_processors().await?;

        Ok((sieve, mock_ingest))
    }

    #[tokio::test]
    async fn test_single_subscribe() -> Result<(), Box<dyn std::error::Error>> {
        let (sieve, mock_ingest) = setup_test_sieve().await?;

        let filter = FilterBuilder::new().transaction(|f| {
            f.value().gte(U256::from(0));
        });

        // Subscribe to the filter.
        let mut stream = sieve.subscribe(filter).await?;

        let tx = generate_random_transaction(100);
        let block = Block {
            transactions: BlockTransactions::Full(vec![tx.clone()]),
            ..Default::default()
        };

        // Send test data through mock ingest
        mock_ingest
            .mock_chain_data(
                Chain::Ethereum,
                ChainData::AnyRPCNetwork(AnyRPCNetwork::Block(AnyRpcBlock::new(block))),
            )
            .expect("must mock chain data.");

        // Wait for and verify the received event
        if let Some(Ok(event)) = stream.next().await {
            match event {
                Event::Transaction(received_tx) => {
                    assert_eq!(received_tx.value(), tx.value());
                }
                _ => unreachable!(),
            }
        } else {
            panic!("No event received");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_watch_within() -> Result<(), Box<dyn std::error::Error>> {
        let (sieve, mock_ingest) = setup_test_sieve().await?;
        let filter1 = FilterBuilder::new().transaction(|f| {
            f.value().gt(U256::from(1000));
        });

        let filter2 = FilterBuilder::new().transaction(|f| {
            f.value().gt(U256::from(2000));
        });

        let mut stream = sieve
            .watch_within(vec![filter1, filter2], Duration::from_secs(5))
            .await?;

        let tx1 = generate_random_transaction(1500);
        let tx2 = generate_random_transaction(2500);

        let block = Block {
            transactions: BlockTransactions::Full(vec![tx1.clone(), tx2.clone()]),
            ..Default::default()
        };

        mock_ingest.mock_chain_data(
            Chain::Ethereum,
            ChainData::AnyRPCNetwork(AnyRPCNetwork::Block(AnyRpcBlock::new(block))),
        )?;

        // Verify we receive a match with both events
        if let Some(Ok(window_event)) = stream.next().await {
            print!("window event");
            match window_event {
                EventWindow::Match(events) => {
                    assert_eq!(events.len(), 2, "Should receive exactly 2 events");

                    let transaction_values: Vec<U256> = events
                        .iter()
                        .filter_map(|event| match event {
                            Event::Transaction(tx) => Some(tx.value()),
                            _ => None,
                        })
                        .collect();

                    assert!(
                        transaction_values.contains(&U256::from(1500))
                            && transaction_values.contains(&U256::from(2500)),
                        "Did not receive expected transactions with values 1500 and 2500"
                    );
                }
                EventWindow::Timeout => panic!("Received timeout instead of match"),
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_watch_within_timeout() -> Result<(), Box<dyn std::error::Error>> {
        let (sieve, _mock_ingest) = setup_test_sieve().await?;

        let filter1 = FilterBuilder::new().transaction(|f| {
            f.value().gt(U256::from(1000));
        });

        let filter2 = FilterBuilder::new().transaction(|f| {
            f.value().gt(U256::from(2000));
        });

        let mut stream = sieve
            .watch_within(vec![filter1, filter2], Duration::from_millis(0))
            .await?;

        if let Some(Ok(window_event)) = stream.next().await {
            match window_event {
                EventWindow::Timeout => {}
                EventWindow::Match(_) => panic!("Expected timeout, got match"),
            }
        }

        Ok(())
    }
}
