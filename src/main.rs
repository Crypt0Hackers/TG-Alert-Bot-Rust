use mev_overwatch::run;

#[tokio::main]
async fn main() {
    println!("CryptoSentinel is watching...");

    dotenv::dotenv().ok();

    run().await;
}
