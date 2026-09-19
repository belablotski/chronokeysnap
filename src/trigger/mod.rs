//! Capture triggers: hotkey and timer, both feeding the same event channel.

pub mod hotkey;
pub mod timer;

use tokio::sync::mpsc;

use crate::config::TriggerConfig;

/// Which trigger caused a capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureEvent {
    Hotkey,
    Timer,
}

/// Start all enabled triggers, returning a channel that receives a `CaptureEvent`
/// each time a hotkey is pressed or the timer fires.
pub fn start(config: &TriggerConfig) -> anyhow::Result<mpsc::Receiver<CaptureEvent>> {
    let (tx, rx) = mpsc::channel(16);

    if config.hotkey.enabled {
        hotkey::spawn(&config.hotkey.combination, tx.clone())?;
    }
    if config.timer.enabled {
        timer::spawn(config.timer.interval_seconds, tx.clone());
    }

    Ok(rx)
}
