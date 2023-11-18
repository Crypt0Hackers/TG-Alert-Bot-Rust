use mev_overwatch::run;

#[tokio::main]
async fn main() {
    println!("MEV Detection System Started");

    dotenv::dotenv().ok();

    run().await;
}
