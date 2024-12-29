use alloy_primitives::U256;
use sieve::{prelude::*, Sieve};
use tokio_stream::StreamExt;
use tracing::{error, info};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting l2 transaction monitor...");

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
    let sieve = match Sieve::connect(chains).await {
        Ok(s) => {
            info!("Successfully connected to chains");
            s
        }
        Err(e) => {
            error!("Failed to connect to chains: {:?}", e);
            return Err(e);
        }
    };

    // 3. Create Filter
    let tx_filter = FilterBuilder::new().optimism(|op| {
        op.field("value").gt(U256::from(100u64));
    });

    // 4. Subscribe to events with the filter
    info!("Subscribing to transaction events...");
    let mut events = sieve.subscribe(tx_filter).await?;

    // 5. Handle events
    while let Some(Ok(event)) = events.next().await {
        println!("Received event: {:?}", event);
    }

    Ok(())
}