use std::sync::Arc;

use ethers::providers::{Middleware, Provider, StreamExt, TransactionStream, Ws};

pub async fn loop_mempool(ws_provider: Arc<Provider<Ws>>) {
    // Subscribe on newPendingTransactions.
    let tx_hash_stream = ws_provider.subscribe_pending_txs().await.unwrap();
    let mut tx_stream = TransactionStream::new(&ws_provider, tx_hash_stream, 256);

    println!("---------- MONITORING MEMPOOL ----------");
    while let Some(maybe_tx) = tx_stream.next().await {
        if let Ok(tx) = maybe_tx {
            println!("---------- NEW TX ----------");
            println!("Tx Hash: {:?}", tx.hash());
        }
    }
}
