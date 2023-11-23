use std::env;
use std::error::Error;
use std::sync::Arc;

use ethers::core::abi::{Abi, Token};
use ethers::etherscan::Client;
use ethers::providers::{Middleware, Provider, StreamExt, TransactionStream, Ws};
use ethers::types::{Bytes, Chain, Transaction, H160, H256, U256};
use serde_json;

struct HeuristicResult {
    should_extract_abi: bool,
}

pub async fn loop_mempool(ws_provider: Arc<Provider<Ws>>) {
    // Subscribe to transactions in the mempool
    let tx_hash_stream = ws_provider.subscribe_pending_txs().await.unwrap();
    let mut tx_stream = TransactionStream::new(&ws_provider, tx_hash_stream, 256);

    println!("---------- MONITORING MEMPOOL ----------");
    while let Some(maybe_tx) = tx_stream.next().await {
        if let Ok(tx) = maybe_tx {
            let (tx_data, tx_from, tx_to, tx_value) = decode_tx(&tx);
            let heuristics_results = run_heuristics(&tx.hash, &tx_from, &tx_to, &tx_value);

            let mut decoded_abi = String::new();

            // If the heuristics indicate that we should extract the ABI, do so
            if heuristics_results.should_extract_abi {
                match get_contract_abi(tx_to).await {
                    Ok(abi) => {
                        decoded_abi = abi;
                        println!("Decoded ABI Retrieved: {:?}", &decoded_abi);
                    }

                    // Skip unverified contracts
                    Err(_) => {
                        continue;
                    }
                }
            }

            let decoded_tx_data = decode_tx_data(&tx_data, &decoded_abi);
            // If the decoding was successful, print the decoded data
            if let Ok(_) = decoded_tx_data {
                println!("Decoded tx data: {:?}", decoded_tx_data);
            }
        }
    }
}

// Extracts the transaction data from a transaction receipt.
fn decode_tx(tx: &Transaction) -> (Bytes, H160, Option<H160>, U256) {
    let tx_data = &tx.input;
    let tx_from = &tx.from;
    let tx_to = &tx.to;
    let tx_value = &tx.value;

    (
        tx_data.clone(),
        tx_from.clone(),
        tx_to.clone(),
        tx_value.clone(),
    )
}

// Runs heuristics on the transaction data.
fn run_heuristics(
    tx_hash: &H256,
    tx_from: &H160,
    tx_to: &Option<H160>,
    tx_value: &U256,
) -> HeuristicResult {
    let mut res = HeuristicResult {
        should_extract_abi: false,
    };

    if tx_to.is_none() {
        println!("Contract creation transaction detected: {:?}", tx_hash);
    }

    // Define the value of 1 ETH in Wei
    let one_eth = U256::exp10(18);

    // Check if the transaction value is greater than 1 ETH
    if tx_value > &one_eth {
        println!("Transaction with large ETH value detected: {:?}", tx_hash);
        res.should_extract_abi = true;
    }

    res
}

// Extract contract ABI using Etherscan API.
async fn get_contract_abi<'a>(contract_address: Option<H160>) -> Result<String, Box<dyn Error>> {
    let etherscan_key = env::var("ETHERSCAN_API_KEY").expect("missing ETHERSCAN_API_KEY");

    // Handle the case where contract_address is None
    let contract_address = match contract_address {
        Some(address) => address,
        None => return Err("No contract address provided".into()),
    };

    // Create the client, handling the Result correctly
    let client = Client::new(Chain::Mainnet, &etherscan_key)?;

    // Fetch the contract metadata
    let metadata = client.contract_source_code(contract_address).await?;

    println!(
        "Getting ABI from contract: {:?}",
        metadata.items[0].contract_name
    );

    let ref abi = metadata.items[0].abi;

    // Return the ABI as a String if available
    Ok(abi.to_string())
}

// Decodes the transaction data using the contract ABI.
fn decode_tx_data(tx_data: &Bytes, abi: &String) -> Result<Vec<Token>, Box<dyn Error>> {
    // Parse the ABI
    let abi: Abi = serde_json::from_str(abi)?;

    // Get function selector from first 4 bytes
    let method_id = &tx_data[0..4];

    // The rest of tx_data are the encoded arguments
    let encoded_args = &tx_data[4..];

    // Find the ABI method
    // Find the ABI method matching the method_id
    let method = abi
        .functions
        .values()
        .flatten()
        .find(|function| function.short_signature() == method_id)
        .ok_or_else(|| Box::<dyn Error>::from("Method not found in ABI"))?;

    // Decode the transaction data
    let decoded = method.decode_input(encoded_args)?;

    Ok(decoded.clone())
}
