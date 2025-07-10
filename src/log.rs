//! Pretty one-liners printed directly with `println!`.

use chrono::Local;
use owo_colors::OwoColorize;
use std::time::Duration;

/// Local [HH:MM:SS] timestamp.
fn ts() -> String {
    Local::now().format("[%H:%M:%S]").to_string()
}

/// Outbound request.
pub(crate) fn outbound(method: &str, url: &str) {
    println!("{} >>> {:<4} {}", ts().dimmed(), method.bold(), url);
}

/// Inbound response.
pub(crate) fn inbound(status: u16, url: &str, bytes: usize, dur: Duration) {
    let coloured_status: String = match status {
        200..=299 => format!("{}", status.green()),
        400..=499 => format!("{}", status.yellow()),
        500..=599 => format!("{}", status.red()),
        _ => format!("{status}"),
    };

    println!(
        "{} <<< {} {} · {} B · {} ms",
        ts().dimmed(),
        coloured_status,
        url,
        bytes,
        dur.as_millis()
    );
}
