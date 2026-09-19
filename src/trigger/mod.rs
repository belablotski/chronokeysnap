//! Capture triggers: hotkey, timer, and manual, all feeding the same event channel.

pub mod hotkey;
pub mod manual;
pub mod timer;

use tokio::sync::mpsc;

use crate::config::TriggerConfig;

/// Which trigger caused a capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureEvent {
    Hotkey,
    Timer,
    Manual,
}

/// Start all enabled triggers, returning a channel that receives a `CaptureEvent`
/// each time a hotkey is pressed, the timer fires, or a manual capture is requested.
pub fn start(config: &TriggerConfig) -> anyhow::Result<mpsc::Receiver<CaptureEvent>> {
    let (tx, rx) = mpsc::channel(16);

    if config.hotkey.enabled {
        hotkey::spawn(&config.hotkey.combination, tx.clone())?;
    }
    if config.timer.enabled {
        timer::spawn(config.timer.interval_seconds, tx.clone());
    }
    if config.manual.enabled {
        manual::spawn(config.manual.port, tx.clone())?;
    }

    Ok(rx)
}
