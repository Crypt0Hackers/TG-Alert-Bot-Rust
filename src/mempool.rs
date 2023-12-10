use std::sync::Arc;

use ethers::providers::{Middleware, Provider, StreamExt, TransactionStream, Ws};
use ethers::types::{Bytes, Transaction, H160, U256};
use std::process::Command;

pub async fn loop_mempool(ws_provider: Arc<Provider<Ws>>) {
    // Subscribe to transactions in the mempool
    let tx_hash_stream = ws_provider.subscribe_pending_txs().await.unwrap();
    let mut tx_stream = TransactionStream::new(&ws_provider, tx_hash_stream, 256);

    println!("---------- MONITORING MEMPOOL ----------");
    while let Some(maybe_tx) = tx_stream.next().await {
        if let Ok(tx) = maybe_tx {
            // Decode Transaction Hash
            let decoded_tx_data = decode_tx(&tx);

            // Run Heuristics on the transaction

            // Raise alert if heuristics indicate malicious activity
        }
    }
}

// Decodes the transaction data using the contract ABI.
fn decode_tx(tx: &Transaction) -> String {
    let tx_hash = tx.hash();

    let output = Command::new("/home/amaechi/.bifrost/bin/heimdall")
        .arg("decode")
        .arg(format!("{:?}", tx_hash))
        .arg("--truncate-calldata")
        .arg("--rpc-url")
        .arg("https://eth.llamarpc.com")
        .output()
        .expect("Failed to execute command");

    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    if let Some(start_index) = output_str.find("heimdall::decode") {
        // Extract everything from "heimdall::decode" to the end
        output_str[start_index..].to_string()
    } else {
        "".to_string()
    }
}
