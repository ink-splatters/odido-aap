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
    match usecase.aanvullen().await {
        Ok(()) => {
            println!("{} {}", "[INFO]".green().bold(), "2000MB aangevuld".bold().bright_purple());
        }
        Err(e) => {
            eprintln!("{} {}", "[ERROR]".red().bold(), e);
        }
    }
}
