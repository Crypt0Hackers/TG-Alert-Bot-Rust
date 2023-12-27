use std::error::Error;
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;

use reqwest;

use ethers::providers::{Middleware, Provider, StreamExt, TransactionStream, Ws};
use ethers::types::{Address, Transaction, H160, H256, U256};

use hex::{decode, encode};

// Telegram bot token and chat id
use crate::helpers::{get_tg_config, TelegramConfig};

// CONSTANTS FOR TESTING
const AAVE_V3_POOL: &str = "0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2";
const UNI_V3_ROUTER: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
const LIDO_STETH: &str = "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84";

struct SimpleAlertConfig {
    monitored_protocols: Vec<H160>, // List of protocols to monitor
    wallet_tracker: Vec<WalletTrackerConfig>, // Conditions to trigger an alert
    invariant_conditions: InvariantConditions, // Conditions to trigger an invariant alert
    is_custom: bool,                // Decide whether we use calldata or predefined conditions
    receive_decoded_tx: bool,       // Decide whether to receive decoded tx data or not
    chat_id: String,                // Telegram chat id
}

struct WalletTrackerConfig {
    token_address: H160, // Address of the token to monitor
    min_tx_value: U256,  // Minimum value of a transaction to trigger an alert
}

struct WalletAlert {
    tx_hash: H256,
    token_address: H160,
    tx_value: U256,
    is_alert: bool,
}

// TODO
struct InvariantAlert {
    tx_hash: H256,
}
// TODO
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
                if (!alert_config.monitored_protocols.contains(&to_addr))
                    && !alert_config.monitored_protocols.is_empty()
                {
                    continue;
                }
            } else {
                continue;
            }

            // Create empty wallet tracker config for alerts
            let mut wallet_alert = WalletAlert {
                tx_hash: H256::zero(),
                token_address: H160::zero(),
                tx_value: U256::zero(),
                is_alert: false,
            };

            // Loop through wallet tracker configs and check if the transaction meets the conditions
            for wallet_tracker_config in &alert_config.wallet_tracker {
                // Check if the wallet tracker contains a native token alert
                if wallet_tracker_config.token_address == H160::zero() {
                    if tx.value >= wallet_tracker_config.min_tx_value {
                        wallet_alert.is_alert = true;
                        wallet_alert.tx_hash = tx.hash;
                        wallet_alert.tx_value = tx.value;
                        wallet_alert.token_address = H160::zero();
                        break;
                    }
                }

                // TODO
                // Check if the wallet tracker contains a token alert
                // Function to extract token transfers and amounts from calldata
            }

            // If no alert was raised, continue to the next transaction
            if !wallet_alert.is_alert {
                continue;
            }

            let TelegramConfig { bot_token } = get_tg_config().await;

            println!("Alert raised for transaction: {:?}", &tx);

            // Send an alert to the user
            if let Err(e) = send_telegram_alert(
                &wallet_alert,
                &bot_token,
                &alert_config.chat_id,
                &alert_config.receive_decoded_tx,
            )
            .await
            {
                println!("Error sending alert: {}", e);
            }
        }
    }
}

async fn send_telegram_alert(
    alert: &WalletAlert,
    bot_token: &str,
    chat_id: &str,
    receive_decoded_tx: &bool,
) -> Result<(), Box<dyn Error>> {
    // Generate the message to send to the user
    let mut message = "".to_string();

    // Decode the transaction data if the user wants to receive decoded tx data
    if *receive_decoded_tx {
        let decoded_tx_data = decode_tx_data(&alert.tx_hash);
        println!("{}", &decoded_tx_data);
        message = format!("An alert was raised for the following transaction: \n\n {:?} \n\nHere's the decoded transaction data: \n {} ", &alert.tx_hash, &decoded_tx_data);
    } else {
        message = format!(
            "An alert was raised for the following transaction: \n\n {:?}",
            &alert.tx_hash
        );
    }

    let client = reqwest::Client::new();
    let telegram_url = format!("https://api.telegram.org/bot{}/sendMessage", &bot_token);
    let params = [("chat_id", chat_id), ("text", &message)];

    client
        .post(&telegram_url)
        .form(&params)
        .send()
        .await?
        .error_for_status()?; // Check if the request was successful

    Ok(())
}

// Decodes the transaction data using Heimdall
fn decode_tx_data(tx: &H256) -> String {
    // Attempt to execute the command
    let output = match Command::new("/home/amaechi/.bifrost/bin/heimdall")
        .arg("decode")
        .arg(format!("{:?}", tx))
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
        output_str[&start_index + 16..].to_string()
    } else {
        "".to_string()
    }
}

// TODO
// Function to load user preferences (from database using chat_id)
fn get_alert_config() -> SimpleAlertConfig {
    SimpleAlertConfig {
        monitored_protocols: vec![
            H160::from_str(AAVE_V3_POOL).unwrap(),
            H160::from_str(UNI_V3_ROUTER).unwrap(),
            H160::from_str(LIDO_STETH).unwrap(),
        ],
        wallet_tracker: vec![WalletTrackerConfig {
            token_address: H160::zero(),
            min_tx_value: U256::from(1 / 100),
        }],
        invariant_conditions: InvariantConditions {
            calldata: "".to_string(),
        },
        is_custom: false,
        receive_decoded_tx: true,
        chat_id: "1782643511".to_string(),
    }
}

pub fn extract_token_transfer(tx_data: &str) -> Result<(Address, U256), &'static str> {
    // Check if the transaction data is a token transfer
    if tx_data.len() < 138 {
        return Err("Transaction data is not a token transfer");
    }

    // Extract the token address and value from the transaction data
    let token_address = &tx_data[34..74];
    let token_value = &tx_data[74..138];

    // Decode the token address and value
    let token_address = decode(token_address).unwrap();
    let token_value = decode(token_value).unwrap();

    // Convert the token address and value to the correct types
    let token_address = Address::from_slice(&token_address);
    let token_value = U256::from_big_endian(&token_value);

    Ok((token_address, token_value))
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
