//! Timer trigger: fires a `CaptureEvent` on a fixed interval.

use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, interval};

use super::CaptureEvent;

/// Spawn a background task that sends a `CaptureEvent` every `interval_seconds`.
pub fn spawn(interval_seconds: u64, tx: Sender<CaptureEvent>) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_seconds.max(1)));
        // The first tick fires immediately; skip it so we wait a full interval
        // before the first capture.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            if tx.send(CaptureEvent::Timer).await.is_err() {
                break;
            }
        }
    });
}
