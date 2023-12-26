use ethers::prelude::{k256::ecdsa::SigningKey, *};

/// Sets up middleware w/ our private key env var.
pub async fn setup_signer(
    provider: Provider<Http>,
) -> SignerMiddleware<Provider<Http>, Wallet<SigningKey>> {
    let chain_id = provider
        .get_chainid()
        .await
        .expect("Failed to get chain id.");

    let priv_key = std::env::var("PRIVATE_KEY").expect("missing PRIVATE_KEY");

    let wallet = priv_key
        .parse::<LocalWallet>()
        .expect("Failed to parse wallet")
        .with_chain_id(chain_id.as_u64());

    SignerMiddleware::new(provider, wallet)
}

pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

// Retrieves telegram bot token and chat id from env vars.
pub async fn get_tg_config() -> TelegramConfig {
    // Load telegram config
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").expect("missing TELEGRAM_BOT_TOKEN");
    let chat_id = std::env::var("TELEGRAM_CHAT_ID").expect("missing TELEGRAM_CHAT_ID");

    TelegramConfig { bot_token, chat_id }
}
