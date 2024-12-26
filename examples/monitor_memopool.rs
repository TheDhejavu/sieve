use alloy_primitives::U256;
use sieve::{prelude::*, Sieve};
use tokio_stream::StreamExt;
use tracing::{error, info};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting Sieve mempool monitor...");

    // 1. Chain Configuration
    let chains = vec![ChainConfigBuilder::builder()
        .rpc("https://ethereum-holesky-rpc.publicnode.com")
        .ws("wss://ws-mainnet.optimism.io")
        .gossipsub("/ip4/0.0.0.0/tcp/9000")
        .bootstrap_peers(vec!["/ip4/127.0.0.1/tcp/8000".to_string()])
        .chain(Chain::Ethereum)
        .build()];

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
    let pool_filter = FilterBuilder::new().pool(|f| {
        f.any_of(|p| {
            p.value().gt(U256::from(100u64));
            p.from().starts_with("0xdead");
            p.to().exact("0x742d35Cc6634C0532925a3b844Bc454e4438f44e");
        });
    });

    // 4. Subscribe to events with the filter
    info!("Subscribing to mempool events...");
    let mut events = sieve.subscribe(pool_filter).await?;

    // 5. Handle events
    while let Some(Ok(event)) = events.next().await {
        println!("Received event: {:?}", event);
    }

    Ok(())
}
