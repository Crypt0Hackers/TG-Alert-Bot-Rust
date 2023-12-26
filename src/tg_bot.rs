use crate::helpers::{get_tg_config, TelegramConfig};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Update {
    update_id: i64,
    message: Option<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    chat: Chat,
    text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Chat {
    id: i64,
}

pub async fn listen() {
    let TelegramConfig { bot_token, .. } = get_tg_config().await;

    let mut offset = 0;
    loop {
        match get_updates(&bot_token, offset).await {
            Ok(updates) => {
                for update in updates {
                    if let Some(message) = update.message {
                        handle_message(&message);
                    }
                    offset = update.update_id + 1;
                }
            }
            Err(e) => eprintln!("Error getting updates: {}", e),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TelegramResponse {
    ok: bool,
    result: Vec<Update>,
}

async fn get_updates(
    bot_token: &str,
    offset: i64,
) -> Result<Vec<Update>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.telegram.org/bot{}/getUpdates?offset={}",
        bot_token, offset
    );
    let response = client.get(url).send().await?.text().await?;

    let telegram_response: TelegramResponse = serde_json::from_str(&response)?;
    if telegram_response.ok {
        Ok(telegram_response.result)
    } else {
        Err("Error fetching updates from Telegram".into())
    }
}
fn handle_message(message: &Message) {
    if let Some(text) = &message.text {
        // Process the command
        println!(
            "Received message: {} from this chat ID: {}",
            text, message.chat.id
        );
        // Here, implement logic to handle different commands and update configurations
    }
}
