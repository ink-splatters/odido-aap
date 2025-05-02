

use clap::builder::styling::*;
use clap::Parser;
mod commands;
use commands::{Commands, Context};

fn styles() -> Styles {
    // Emulate clap v3 default colors
    Styles::styled()
        .header(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))).bold())
        .usage(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
        .error(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))).bold())
}



/// odido.nl aanvullers 
#[derive(Debug,Parser)]
#[command(version,styles=styles())]
struct Cli {
    /// odido.nl bearer token [env: ODIDO_TOKEN]
    #[arg(
        long,
        env = "ODIDO_TOKEN",
        hide_env = true
    )]
    bearer: String,

    #[arg(
        short,
        long,
        help = "Request new aanvullen when less than <threshold> remains",
        env = "ODIDO_THRESHOLD",
        value_name="MB",
        default_value_t = 1500
    )]
    threshold: u32,

    #[arg(
        long = "threshold-max",
        env = "ODIDO_THRESHOLD_MAX",
        value_name="MB",
        default_value_t = 22000,
        hide = true
    )]
    threshold_max: u32,

    #[arg(
        long = "aanvuller-size",
        env = "ODIDO_AANVULLER_SIZE",
        value_name="MB",
        default_value_t = 2000,
        hide = true
    )]
    aanvuller_size: u32,

    #[command(subcommand)]
    command: Option<Commands>
}

fn main() {
    let cli = Cli::parse();
    let ctx = Context {
        threshold: cli.threshold,
        threshold_max: cli.threshold_max,
        aanvuller_size: cli.aanvuller_size,
    };
    match cli.command.unwrap_or(Commands::Status) {
        Commands::Status => commands::status::run(&ctx),
        Commands::Aanvullen => commands::aanvullen::run(&ctx),
        Commands::Daemon => commands::daemon::run(&ctx),
    }
}