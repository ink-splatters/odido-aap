pub mod status;
pub mod aanvullen;
pub mod daemon;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show status (default)
    Status,
    /// request aanvuller
    #[command(visible_aliases=["a","af"])]
    Aanvullen,
    /// run in daemon mode (TODO)
    Daemon,
}

#[derive(Debug)]
pub struct Context {
    // Add more fields if you want to pass more CLI opts
}
