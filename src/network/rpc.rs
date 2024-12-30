//! RPC connection to the Ethereum network with alloy
//! Ref: https://alloy.rs/building-with-alloy/connecting-to-a-blockchain/setting-up-a-provider
use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
    time::Duration,
};

use alloy_network::AnyNetwork;
use alloy_primitives::U256;
use alloy_provider::{Provider, ProviderBuilder, RootProvider};
use alloy_rpc_types::{BlockId, BlockNumberOrTag, BlockTransactionsKind};
use alloy_transport_http::Http;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use pin_project_lite::pin_project;
use reqwest::Client;
use tokio::{
    sync::mpsc::{self, Receiver},
    task::JoinHandle,
    time,
};
use tracing::{debug, error};

use crate::network::orchestrator::{AnyRPCNetwork, ChainData, ChainOrchestrator};

// [`BlockStream`] is a self-contained stream that fetches block data from an external source.
// It operates only when polled, which is preferable to manual busy-polling as it leverages
// tokio's runtime scheduler to handle the polling logic efficiently.
pin_project! {
    struct BlockStream {
        #[pin]
        provider: Arc<RootProvider<Http<Client>, AnyNetwork> >,
        interval: time::Interval,
        future: Option<Pin<Box<dyn Future<Output = Option<ChainData>> + Send>>>,
    }
}

impl Stream for BlockStream {
    type Item = ChainData;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();

        if let Some(fut) = this.future {
            match fut.as_mut().poll(cx) {
                Poll::Ready(item) => {
                    *this.future = None;
                    if item.is_some() {
                        return Poll::Ready(item);
                    }
                    // continue polling
                    return self.poll_next(cx);
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        match this.interval.poll_tick(cx) {
            Poll::Ready(_) => {
                let provider = this.provider.clone();

                *this.future = Some(Box::pin(async move {
                    match provider
                        .get_block(
                            BlockId::Number(BlockNumberOrTag::Latest),
                            BlockTransactionsKind::Full,
                        )
                        .await
                    {
                        Ok(block) => {
                            if let Some(block_data) = block {
                                return Some(ChainData::AnyRPCNetwork(AnyRPCNetwork::Block(
                                    block_data,
                                )));
                            }
                        }
                        Err(e) => {
                            error!(?e, "Error polling blocks");
                        }
                    }
                    None
                }));
                self.poll_next(cx)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// [`PendingTxPoolStream`] is a self-contained stream that has all it needs to get pending transaction data
pin_project! {
    struct PendingTxPoolStream {
        provider: Arc<RootProvider<Http<Client>, AnyNetwork> >,
        filter_id: U256,
        interval: time::Interval,
        future: Option<Pin<Box<dyn Future<Output = Option<ChainData>> + Send>>>,
    }
}

impl PendingTxPoolStream {
    async fn new(
        provider: Arc<RootProvider<Http<Client>, AnyNetwork>>,
        poll_interval: Duration,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let filter_id = provider.new_pending_transactions_filter(false).await?;

        Ok(Self {
            interval: time::interval(poll_interval),
            future: None,
            provider,
            filter_id,
        })
    }
}

impl Stream for PendingTxPoolStream {
    type Item = ChainData;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();
        if let Some(fut) = this.future {
            match fut.as_mut().poll(cx) {
                Poll::Ready(item) => {
                    *this.future = None;
                    if item.is_some() {
                        return Poll::Ready(item);
                    }
                    // If no transaction was found, continue polling
                    return self.poll_next(cx);
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        match this.interval.poll_tick(cx) {
            Poll::Ready(_) => {
                let provider = this.provider.clone();
                let filter_id = *this.filter_id;

                *this.future = Some(Box::pin(async move {
                    if let Ok(changes) = provider.get_filter_changes(filter_id).await {
                        for tx_hash in changes {
                            if let Ok(Some(tx)) = provider.get_transaction_by_hash(tx_hash).await {
                                return Some(ChainData::AnyRPCNetwork(
                                    AnyRPCNetwork::TransactionPool(tx),
                                ));
                            }
                        }
                    }
                    None
                }));
                self.poll_next(cx)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct RpcOrchestrator {
    provider: Arc<RootProvider<Http<Client>, AnyNetwork>>,
    poll_interval: Duration,
    is_running: Arc<AtomicBool>,
    name: String,
    block_task: Option<JoinHandle<()>>,
    tx_pool_task: Option<JoinHandle<()>>,
}

impl RpcOrchestrator {
    pub fn new(
        name: String,
        rpc_url: String,
        poll_interval: Duration,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed_rpc_url = rpc_url.parse()?;
        let provider: RootProvider<Http<Client>, AnyNetwork> = ProviderBuilder::new()
            .network::<AnyNetwork>()
            .on_http(parsed_rpc_url);

        Ok(Self {
            provider: Arc::new(provider),
            poll_interval,
            is_running: Arc::new(AtomicBool::new(false)),
            name,
            block_task: None,
            tx_pool_task: None,
        })
    }

    async fn block_stream(&self) -> BlockStream {
        BlockStream {
            provider: Arc::clone(&self.provider),
            interval: time::interval(self.poll_interval),
            future: None,
        }
    }

    async fn pending_tx_pool_stream(&self) -> PendingTxPoolStream {
        let pending_tx_pool_stream =
            PendingTxPoolStream::new(self.provider.clone(), self.poll_interval).await;
        pending_tx_pool_stream.expect("unable to stream pending transaction")
    }
}

#[async_trait]
impl ChainOrchestrator for RpcOrchestrator {
    async fn start(&mut self) -> Result<Receiver<ChainData>, Box<dyn std::error::Error>> {
        if self.is_running.load(Ordering::Relaxed) {
            return Err("Orchestrator is already running".into());
        }

        self.is_running.store(true, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel(10_000);

        // Start block polling stream
        let mut block_stream = self.block_stream().await;
        let block_tx = tx.clone();
        let block_is_running = self.is_running.clone();

        let block_task = tokio::spawn(async move {
            while let Some(block) = block_stream.next().await {
                debug!("new pending block: {block:#?}");
                if !block_is_running.load(Ordering::Relaxed) {
                    break;
                }
                let _ = block_tx.send(block).await;
            }
        });

        // Start txpool polling stream
        let mut pending_tx_stream = self.pending_tx_pool_stream().await;
        let pool_tx = tx.clone();
        let pool_is_running = self.is_running.clone();

        let pool_task = tokio::spawn(async move {
            while let Some(tx) = pending_tx_stream.next().await {
                println!("new pending transaction: {tx:#?}");
                if !pool_is_running.load(Ordering::Relaxed) {
                    break;
                }
                let _ = pool_tx.send(tx).await;
            }
        });

        self.block_task = Some(block_task);
        self.tx_pool_task = Some(pool_task);

        Ok(rx)
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_running.store(false, Ordering::Relaxed);

        if let Some(task) = &self.block_task {
            task.abort();
        }

        if let Some(task) = &self.tx_pool_task {
            task.abort();
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}


// The filter engine is designed as a high-performance, memory-efficient system for processing multiple concurrent filters across super-chain and Ethereum data streams. It implements a priority-based filtering approach with several optimizations to efficiently handle high volumes of concurrent filters. At its core, it combines simple predicate matching with optimized execution strategies to handle thousands of concurrent filters efficiently.
// The engine evaluates each filter node using parallelization to efficiently traverse the condition trees. For each incoming data (blocks, transactions e.t.c), it performs pre-evaluation steps to quickly determine if a block might match a filter's criteria, preventing unnecessary decoding of  data such as calldata, input e.t.c. The engine also implements intelligent caching of decoded data, ensuring that frequently accessed data is decoded only once and reused across multiple filter evaluations.