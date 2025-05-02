use crate::api::client::OdidoClient;
use owo_colors::OwoColorize;

pub async fn run(_ctx: &crate::commands::Context) {
    let token = std::env::var("ODIDO_TOKEN").expect("ODIDO_TOKEN not set");
    let client = OdidoClient::new(token);
    match client.get_linked_subscriptions().await {
        Ok(linked) => {
            if let Some(subscription) = linked.subscriptions.get(0) {
                match client.top_up(&subscription.url, "A0DAY01").await {
                    Ok(()) => println!("{} {}", "[INFO]".green().bold(), "2000MB aangevuld".bold().bright_purple()),
                    Err(e) => eprintln!("{} Failed to top up: {}", "[ERROR]".red().bold(), e),
                }
            } else {
                eprintln!("{} No subscriptions found.", "[ERROR]".red().bold());
            }
        }
        Err(e) => eprintln!("{} Failed to get subscriptions: {}", "[ERROR]".red().bold(), e),
    }
}
