//! Pretty one-liners emitted at INFO level.

use std::time::Duration;
use tracing::info;

/// Local HH:MM:SS timestamp.
fn stamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

/// Outbound request.
pub(crate) fn outbound(method: &str, url: &str) {
    info!("{:>8}  >>> {}  {}", stamp(), method, url);
}

/// Inbound response.
pub(crate) fn inbound(status: u16, bytes: usize, dur: Duration) {
    info!(
        "{:>8}  <<< {}  {:>5} B · {:>4} ms",
        stamp(),
        status,
        bytes,
        dur.as_millis()
    );
}