//! RPC connection to the Ethereum network with alloy
//! Ref: https://alloy.rs/building-with-alloy/connecting-to-a-blockchain/setting-up-a-provider
use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    task::{Context, Poll},
    time::Duration,
};

use alloy_provider::{Provider, ProviderBuilder, RootProvider};
use alloy_rpc_types::{BlockId, BlockNumberOrTag, BlockTransactionsKind, Transaction};
use alloy_transport_http::Http;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use pin_project_lite::pin_project;
use reqwest::Client;
use tokio::{task::JoinHandle, time};
use tracing::debug;

use crate::network::orchestrator::{ChainData, ChainOrchestrator, EthereumData};

// [`BlockStream`] is a self-contained stream that fetches block data from an external source.
// It operates only when polled, which is preferable to manual busy-polling as it leverages
// tokio's runtime scheduler to handle the polling logic efficiently.
pin_project! {
    struct BlockStream {
        #[pin]
        provider: Arc<RootProvider<Http<Client>>>,
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
                    return Poll::Ready(item);
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
                                return Some(ChainData::Ethereum(EthereumData::Block(block_data)));
                            }
                        }
                        Err(e) => {
                            debug!(?e, "Error polling blocks");
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
        provider: Arc<RootProvider<Http<Client>>>,
        tx_stream: Pin<Box<dyn Stream<Item = Vec<Transaction>> + Send>>,
    }
}

impl PendingTxPoolStream {
    async fn new(
        provider: Arc<RootProvider<Http<Client>>>,
        poll_interval: Duration,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Get the poller and configure it with our interval
        let mut poller = provider.watch_full_pending_transactions().await?;

        poller.set_poll_interval(poll_interval);
        let stream = poller.into_stream();

        Ok(Self {
            provider,
            tx_stream: Box::pin(stream),
        })
    }
}

impl Stream for PendingTxPoolStream {
    type Item = ChainData;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();
        match this.tx_stream.poll_next_unpin(cx) {
            Poll::Ready(Some(txs)) => Poll::Ready(Some(ChainData::Ethereum(
                EthereumData::TransactionPool(txs),
            ))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct RpcOrchestrator {
    provider: Arc<RootProvider<Http<Client>>>,
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
        let provider = ProviderBuilder::new().on_http(parsed_rpc_url);

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
        let (tx, rx) = mpsc::channel();

        // Start block polling stream
        let mut block_stream = self.block_stream().await;
        let block_tx = tx.clone();
        let block_is_running = self.is_running.clone();

        let block_task = tokio::spawn(async move {
            while let Some(block) = block_stream.next().await {
                println!("new pending transaction: {block:#?}");
                if !block_is_running.load(Ordering::Relaxed) {
                    break;
                }
                let _ = block_tx.send(block);
            }
        });

        // Start txpool polling stream
        let mut txpool_stream = self.pending_tx_pool_stream().await;
        let pool_tx = tx.clone();
        let pool_is_running = self.is_running.clone();

        let pool_task = tokio::spawn(async move {
            while let Some(transaction_data) = txpool_stream.next().await {
                println!("new pending transaction: {transaction_data:#?}");
                if !pool_is_running.load(Ordering::Relaxed) {
                    break;
                }
                let _ = pool_tx.send(transaction_data);
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
