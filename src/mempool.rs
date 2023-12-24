use std::sync::Arc;

use ethers::providers::{Middleware, Provider, StreamExt, TransactionStream, Ws};
use ethers::types::{Transaction, H160, U256};
use std::process::Command;

use crate::constants::AAVE_V3_POOL;

struct SimpleAlertConfig {
    monitored_protocols: Vec<H160>,    // List of protocols to monitor
    monitored_addresses: Vec<H160>,    // List of addresses to monitor
    alert_conditions: AlertConditions, // Conditions to trigger an alert
    invariant_conditions: InvariantConditions, // Conditions to trigger an invariant alert
}

struct AlertConditions {
    min_tx_value: U256, // Minimum value of a transaction to trigger an alert
}

struct InvariantConditions {
    min_tx_value: U256, // Minimum value of a transaction to trigger an invariant alert
}

pub async fn loop_mempool(ws_provider: Arc<Provider<Ws>>) {
    // Subscribe to transactions in the mempool
    let tx_hash_stream = ws_provider.subscribe_pending_txs().await.unwrap();
    let mut tx_stream = TransactionStream::new(&ws_provider, tx_hash_stream, 256);

    // Load user alert config
    let alert_config = get_alert_config();

    println!("---------- MONITORING MEMPOOL ----------");
    while let Some(maybe_tx) = tx_stream.next().await {
        if let Ok(tx) = maybe_tx {
            // TX string representation
            let decoded_tx_data = generate_text_from_tx(&tx);
            // Contract string representation
            let decoded_contract = generate_contract_from_address(&tx.to);
        }
    }
}

// TODO
// Function to load user preferences (from file, database, etc.)
fn get_alert_config() -> SimpleAlertConfig {
    SimpleAlertConfig {
        monitored_protocols: vec![H160::from(0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2)],
        monitored_addresses: vec![],
        alert_conditions: AlertConditions {
            min_tx_value: U256::from(100),
        },
        invariant_conditions: InvariantConditions {
            min_tx_value: U256::from(100),
        },
    }
}

// Decodes the transaction data using Heimdall
fn generate_text_from_tx(tx: &Transaction) -> String {
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

// Decodes the contract data using Heimdall
fn generate_contract_from_address(contract: &Option<H160>) -> String {
    let contract_str = match contract {
        Some(addr) => addr.to_string(),
        None => return "".to_string(),
    };

    let output = Command::new("/home/amaechi/.bifrost/bin/heimdall")
        .arg("decompile")
        .arg(&contract_str)
        .arg("--rpc-url")
        .arg("https://eth.llamarpc.com")
        .arg("-vvv")
        .output()
        .expect("Failed to execute command");

    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    // if let Some(start_index) = output_str.find("heimdall::decode") {
    //     // Extract everything from "heimdall::decode" to the end
    //     println!("{}", output_str[start_index..].to_string());
    //     output_str[start_index..].to_string()
    // } else {
    //     "".to_string()
    // }

    if output_str == "".to_string() {
        return "".to_string();
    }

    println!("{}", output_str);
    output_str
}
