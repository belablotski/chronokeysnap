//! Global hotkey trigger: listens for a system-wide keyboard shortcut.

use anyhow::{Context, Result};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::HotKey};
use tokio::sync::mpsc::Sender;
use tracing::info;

use super::CaptureEvent;

/// Register `combination` (e.g. "Ctrl+Shift+S") and forward a `CaptureEvent`
/// each time it's pressed.
///
/// Spawns a dedicated OS thread because hotkey events are delivered through a
/// blocking channel, and keeps the `GlobalHotKeyManager` alive on that thread
/// for as long as it runs (dropping it unregisters the hotkey).
pub fn spawn(combination: &str, tx: Sender<CaptureEvent>) -> Result<()> {
    let hotkey: HotKey = combination
        .parse()
        .with_context(|| format!("invalid hotkey combination: {combination}"))?;

    let manager =
        GlobalHotKeyManager::new().context("failed to create global hotkey manager")?;
    manager
        .register(hotkey)
        .with_context(|| format!("failed to register hotkey: {combination}"))?;
    info!(hotkey = combination, "hotkey registered");

    let combination = combination.to_string();
    std::thread::spawn(move || {
        let _manager = manager;
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            if event.state == HotKeyState::Pressed {
                info!(hotkey = %combination, "hotkey pressed");
                if tx.blocking_send(CaptureEvent::Hotkey).is_err() {
                    break;
                }
            }
        }
    });

    Ok(())
}
