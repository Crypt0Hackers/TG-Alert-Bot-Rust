use std::str::FromStr;
use std::sync::Arc;

use reqwest;
use std::error::Error;

use ethers::providers::{Middleware, Provider, StreamExt, TransactionStream, Ws};
use ethers::types::{Transaction, H160, H256, U256};
use std::process::Command;

// CONSTANTS FOR TESTING
const AAVE_V3_POOL: &str = "0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2";
const UNI_V3_ROUTER: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
const LIDO_STETH: &str = "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84";

struct SimpleAlertConfig {
    monitored_protocols: Vec<H160>,    // List of protocols to monitor
    alert_conditions: AlertConditions, // Conditions to trigger an alert
    invariant_conditions: InvariantConditions, // Conditions to trigger an invariant alert
    is_custom: bool,                   // Decide whether we use calldata or predefined conditions
    alert_recipients: Vec<String>,     // List of recipients to send alerts to
}

struct AlertConditions {
    min_tx_value: U256, // Minimum value of a transaction to trigger an alert
    monitored_addresses: Vec<H160>, // List of addresses to monitor
}

struct InvariantConditions {
    calldata: String, // Calldata to trigger an invariant check
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
            // Check if the transaction is from a monitored protocol and monitored protocols contains at least one address
            if let Some(to_addr) = tx.to {
                if !alert_config.monitored_protocols.contains(&to_addr)
                    && !alert_config.monitored_protocols.is_empty()
                {
                    continue;
                }
            } else {
                continue;
            }

            // Check if the transaction value is greater than the minimum value
            if tx.value <= alert_config.alert_conditions.min_tx_value {
                continue;
            }

            // Check if the transaction is from a monitored address
            if !alert_config
                .alert_conditions
                .monitored_addresses
                .contains(&tx.from)
                && !alert_config.alert_conditions.monitored_addresses.is_empty()
            {
                continue;
            }

            // At this point, we have a transaction that meets all the conditions to trigger an alert
            send_alert(alert_config.alert_recipients.clone(), &tx).await;

            // Contract string representation
            // let decoded_contract = generate_contract_from_address(&tx.to);
        }
    }
}

// TODO
// Function to load user preferences (from file, database, etc.)
fn get_alert_config() -> SimpleAlertConfig {
    SimpleAlertConfig {
        monitored_protocols: vec![
            H160::from_str(AAVE_V3_POOL).expect("Invalid Address"),
            H160::from_str(UNI_V3_ROUTER).expect("Invalid Address"),
            H160::from_str(LIDO_STETH).expect("Invalid Address"),
        ],
        alert_conditions: AlertConditions {
            min_tx_value: U256::from(1 / 100),
            monitored_addresses: vec![],
        },
        invariant_conditions: InvariantConditions {
            calldata: "".to_string(),
        },
        is_custom: false,
        alert_recipients: vec!["okolobiam@gmail.com"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

// TODO
// Function to send alerts to users
async fn send_alert(alert_recipients: Vec<String>, tx: &Transaction) {
    for recipient in alert_recipients {
        println!("Sending alert to {}", recipient);
    }

    // TX string representation
    let decoded_tx_data = generate_text_from_tx(&tx);

    println!("TX Data for this alert: {}", decoded_tx_data);

    if let Err(e) = send_telegram_alert(&tx.hash, &decoded_tx_data).await {
        println!("Error sending alert: {}", e);
    }
}

async fn send_telegram_alert(tx: &H256, decoded_tx_data: &str) -> Result<(), Box<dyn Error>> {
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").expect("missing TELEGRAM_BOT_TOKEN");
    let chat_id = std::env::var("TELEGRAM_CHAT_ID").expect("missing TELEGRAM_CHAT_ID");
    let message = format!("An alert was raised for the following transaction: \n\n {:?} \n\nHere's the decoded transaction data: \n {} ", tx, decoded_tx_data);

    let client = reqwest::Client::new();
    let telegram_url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let params = [("chat_id", chat_id), ("text", message)];

    client
        .post(&telegram_url)
        .form(&params)
        .send()
        .await?
        .error_for_status()?; // Check if the request was successful

    Ok(())
}

// Decodes the transaction data using Heimdall
fn generate_text_from_tx(tx: &Transaction) -> String {
    let tx_hash = tx.hash();

    // Attempt to execute the command
    let output = match Command::new("/home/amaechi/.bifrost/bin/heimdall")
        .arg("decode")
        .arg(format!("{:?}", tx_hash))
        .arg("--truncate-calldata")
        .arg("--rpc-url")
        .arg("https://eth.llamarpc.com")
        .output()
    {
        Ok(output) => output,
        Err(_) => return "".to_string(), // Return an empty string in case of an error
    };

    let output_str = String::from_utf8_lossy(&output.stdout).to_string();

    // Extract the relevant part of the output
    if let Some(start_index) = output_str.find("heimdall::decode") {
        output_str[start_index + 16..].to_string()
    } else {
        "".to_string()
    }
}

// Decodes the contract data using Heimdall
// fn generate_contract_from_address(contract: &Option<H160>) -> String {
//     let contract_str = match contract {
//         Some(addr) => addr.to_string(),
//         None => return "".to_string(),
//     };

//     let output = Command::new("/home/amaechi/.bifrost/bin/heimdall")
//         .arg("decompile")
//         .arg(&contract_str)
//         .arg("--rpc-url")
//         .arg("https://eth.llamarpc.com")
//         .arg("-vvv")
//         .output()
//         .expect("Failed to execute command");

//     let output_str = String::from_utf8_lossy(&output.stdout).to_string();

//     // if let Some(start_index) = output_str.find("heimdall::decode") {
//     //     // Extract everything from "heimdall::decode" to the end
//     //     println!("{}", output_str[start_index..].to_string());
//     //     output_str[start_index..].to_string()
//     // } else {
//     //     "".to_string()
//     // }

//     if output_str == "".to_string() {
//         return "".to_string();
//     }

//     println!("{}", output_str);
//     output_str
// }
