use std::error::Error;
use std::sync::Arc;

use ethers::etherscan::Client;
use ethers::providers::{Middleware, Provider, StreamExt, TransactionStream, Ws};
use ethers::types::{Bytes, Chain, Transaction, H160, H256, U256};

pub async fn loop_mempool(ws_provider: Arc<Provider<Ws>>) {
    // Subscribe to transactions in the mempool
    let tx_hash_stream = ws_provider.subscribe_pending_txs().await.unwrap();
    let mut tx_stream = TransactionStream::new(&ws_provider, tx_hash_stream, 256);

    println!("---------- MONITORING MEMPOOL ----------");
    while let Some(maybe_tx) = tx_stream.next().await {
        match maybe_tx {
            Ok(tx) => {
                println!("Processing transaction: {:?}", tx.hash());
                let (tx_hash, tx_data, tx_from, tx_to, tx_value) = decode_tx(tx);
                let (should_analyse) = run_heuristics(tx_data, tx_from, tx_to, tx_value);
                if should_analyse {
                    println!("Contract creation transaction detected: {:?}", tx_hash)
                    // match get_contract_abi(tx_to).await {
                    //     Ok(abi) => {
                    //         // Process the ABI, maybe log it or perform further analysis
                    //         println!("ABI: {:?}", abi);
                    //     }
                    //     Err(e) => eprintln!("Error fetching contract ABI: {:?}", e),
                    // }
                }
            }
            Err(e) => eprintln!("Error processing transaction: {:?}", e),
        }
    }
}

// Extracts the transaction data from a transaction receipt.
fn decode_tx(tx: Transaction) -> (H256, Bytes, H160, Option<H160>, U256) {
    let tx_hash = tx.hash();
    let tx_data = tx.input;
    let tx_from = tx.from;
    let tx_to = tx.to;
    let tx_value = tx.value;

    (tx_hash, tx_data, tx_from, tx_to, tx_value)
}

// Runs heuristics on the transaction data.
fn run_heuristics(tx_data: Bytes, tx_from: H160, tx_to: Option<H160>, tx_value: U256) -> bool {
    if tx_to.is_none()
    // Add more heuristics here.
    {
        return true;
    } else {
        return false;
    }
}

// Extract contract ABI using Etherscan API.
async fn get_contract_abi(contract_address: Option<H160>) -> Result<String, Box<dyn Error>> {
    let etherscan_key = std::env::var("ETHERSCAN_API_KEY").expect("missing ETHERSCAN_API_KEY");

    // Create the client, handling the Result correctly
    let client = Client::new(Chain::Mainnet, &etherscan_key)?;

    // Fetch the contract metadata
    let metadata = client
        .contract_source_code(contract_address.unwrap())
        .await?;

    println!(
        "Interacting With Contract: {:?}",
        metadata.items[0].contract_name
    );

    // Return the ABI as a String if available
    Ok(metadata.items[0].abi.clone())
}
