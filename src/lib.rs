pub mod block_scanner;
pub mod helpers;
pub mod mempool;
pub mod tg_bot;

use std::sync::Arc;

use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::*;

use crate::helpers::setup_signer;

pub struct Config {
    #[allow(dead_code)]
    pub http: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    #[allow(dead_code)]
    pub wss: Arc<Provider<Ws>>,
}

impl Config {
    pub async fn new() -> Self {
        let network = std::env::var("NETWORK_RPC").expect("missing NETWORK_RPC");
        let provider: Provider<Http> = Provider::<Http>::try_from(network).unwrap();
        let middleware = Arc::new(setup_signer(provider.clone()).await);

        let ws_network = std::env::var("NETWORK_WSS").expect("missing NETWORK_WSS");
        let ws_provider: Provider<Ws> = Provider::<Ws>::connect(ws_network).await.unwrap();
        Self {
            http: middleware,
            wss: Arc::new(ws_provider),
        }
    }
}

pub async fn run() {
    let config = Config::new().await;

    // Thread for checking what block we're on.
    tokio::spawn(async move {
        block_scanner::loop_blocks(Arc::clone(&config.http)).await;
    });

    // Thread for listening to Telegram messages.
    tokio::spawn(async move {
        tg_bot::listen().await;
    });

    // Main loop to monitor the mempool.
    mempool::loop_mempool(Arc::clone(&config.wss)).await;
}
