use crate::api::usecase::{AutoTopUp, AutoTopUpConfig};
use owo_colors::OwoColorize;
use reqwest::Client;

pub async fn run(ctx: &crate::commands::Context) {
    let token = std::env::var("ODIDO_TOKEN").expect("ODIDO_TOKEN not set");
    let config = AutoTopUpConfig {
        token,
        threshold: ctx.threshold as u64,
        bundle_buying_code: "A0DAY01",
    };
    let client = Client::builder().cookie_store(true).build().unwrap();
    let usecase = AutoTopUp { client: &client, config };
    match usecase.status().await {
        Ok(remaining_mb) => {
            println!("{} There is {} MB remaining in your bundle.", "[INFO]".green().bold(), remaining_mb.to_string().bright_yellow());
        }
        Err(e) => {
            eprintln!("{} {}", "[ERROR]".red().bold(), e);
        }
    }
}
