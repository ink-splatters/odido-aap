use crate::api::client::OdidoClient;
use owo_colors::OwoColorize;

pub async fn run(_ctx: &crate::commands::Context) {
    let token = std::env::var("ODIDO_TOKEN").expect("ODIDO_TOKEN not set");
    let client = OdidoClient::new(token);
    match client.get_linked_subscriptions().await {
        Ok(linked) => {
            if let Some(subscription) = linked.subscriptions.get(0) {
                match client.get_roaming_bundles(&subscription.url).await {
                    Ok(bundles) => {
                        let total_remaining_kb: u64 = bundles.bundles.iter()
                            .filter(|b| b.zone_color == "NL")
                            .map(|b| b.remaining.value)
                            .sum();
                        let remaining_mb = total_remaining_kb / 1024;
                        println!("{} There is {} MB remaining in your bundle.", "[INFO]".green().bold(), remaining_mb.to_string().bright_yellow());
                    }
                    Err(e) => eprintln!("{} Failed to get roaming bundles: {}", "[ERROR]".red().bold(), e),
                }
            } else {
                eprintln!("{} No subscriptions found.", "[ERROR]".red().bold());
            }
        }
        Err(e) => eprintln!("{} Failed to get subscriptions: {}", "[ERROR]".red().bold(), e),
    }
}
