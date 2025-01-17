use alloy_primitives::U256;
use eyre::{Result, WrapErr};
use sieve::{prelude::*, EventWindow, Sieve};
use std::time::Duration;
use tokio_stream::StreamExt;
use tracing::info;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting l1 & l2 transaction listener...");

    // 1. Chain Configuration
    let chains = vec![
        ChainConfigBuilder::builder()
            .rpc("https://optimism-sepolia-rpc.publicnode.com")
            .ws("wss://optimism-sepolia-rpc.publicnode.com")
            .chain(Chain::Optimism)
            .build(),
        ChainConfigBuilder::builder()
            .rpc("https://ethereum-holesky-rpc.publicnode.com")
            .ws("wss://ethereum-holesky-rpc.publicnode.com")
            .chain(Chain::Ethereum)
            .build(),
    ];

    // 2. Connect to chains via `Sieve`
    let sieve = Sieve::connect(chains)
        .await
        .wrap_err("Failed to connect to chains")?;

    // 3. Create Filter
    let eth_tx_filter = FilterBuilder::new()
        .chain(Chain::Ethereum)
        .transaction(|op| {
            op.field("value").gt(U256::from(100u64));
        });

    let op_tx_filter = FilterBuilder::new()
        .chain(Chain::Optimism)
        .transaction(|op| {
            op.field("value").gt(U256::from(100u64));
        });

    // 4. Subscribe to events with the filter
    info!("Subscribing to transaction events on multiple chains ...");
    let mut events = sieve
        .watch_within(
            vec![eth_tx_filter, op_tx_filter],
            Duration::from_secs(5 * 60 * 60),
        )
        .await
        .wrap_err("Failed to subscribe to chain events")?;

    // 5. Handle events
    while let Some(Ok(event)) = events.next().await {
        match event {
            // Handle matched events within the time window
            EventWindow::Match(events) => {
                // Events are ordered based on filter ordering ["eth_event", "op_event"]
                println!("Found matching events within time window: {events:#?}");
            }
            // Handle events that timed out without a match
            EventWindow::Timeout => {
                println!("Time window expired without finding all matches");
            }
        }
    }

    Ok(())
}
