mod capture;
mod config;
mod storage;
mod trigger;

use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{error, info};

use config::Config;
use storage::{Storage, local::LocalStorage};
use tracing_subscriber::EnvFilter;

/// Capture screenshots on a hotkey or timer and save them to a storage backend.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the config file (defaults to the platform config directory).
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Default to "info" level so logs are visible without setting RUST_LOG.
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    info!("chronokeysnap starting");

    let args = Args::parse();
    let config_path = match args.config {
        Some(path) => path,
        None => Config::default_path()?,
    };
    let config = Config::load(&config_path)?;
    info!(path = %config_path.display(), "loaded config");

    let storage: Arc<dyn Storage> = match config.storage.backend {
        config::StorageBackendKind::Local => {
            Arc::new(LocalStorage::new(config.resolved_local_folder())?)
        }
    };
    info!(
        backend = ?config.storage.backend,
        folder = %config.resolved_local_folder().display(),
        "storage backend configured"
    );

    let mut events = trigger::start(&config.trigger).context("failed to start triggers")?;
    info!(
        hotkey_enabled = config.trigger.hotkey.enabled,
        hotkey = %config.trigger.hotkey.combination,
        timer_enabled = config.trigger.timer.enabled,
        "triggers started, waiting for capture events (Ctrl+C to quit)"
    );

    loop {
        tokio::select! {
            event = events.recv() => {
                match event {
                    // Spawn instead of awaiting inline, so a slow/in-flight
                    // capture can't delay noticing the next Ctrl+C or event.
                    Some(event) => {
                        let storage = Arc::clone(&storage);
                        tokio::spawn(async move { on_capture_event(&storage, event).await });
                    }
                    None => {
                        info!("all triggers stopped, exiting");
                        break;
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("received Ctrl+C, shutting down");
                break;
            }
        }
    }

    Ok(())
}

async fn on_capture_event(storage: &Arc<dyn Storage>, event: trigger::CaptureEvent) {
    match event {
        trigger::CaptureEvent::Hotkey => info!("hotkey pressed, capturing screenshot"),
        trigger::CaptureEvent::Timer => info!("timer tick, capturing screenshot"),
    }

    // capture_all_monitors is synchronous and can be slow (or panic, e.g. some
    // xcap backends on Wayland without zxdg_output_manager_v1); run it on a
    // blocking thread so it can't stall the event loop (and Ctrl+C handling).
    let captured = tokio::task::spawn_blocking(capture::capture_all_monitors).await;
    let items = match captured {
        Ok(Ok(items)) => items,
        Ok(Err(err)) => {
            error!(error = %err, "failed to capture screenshot");
            return;
        }
        Err(join_err) => {
            error!(error = %join_err, "screenshot capture task failed (likely an unsupported display backend)");
            return;
        }
    };

    for item in items {
        match storage.save(&item).await {
            Ok(location) => info!(location = %location.0, "saved screenshot"),
            Err(err) => error!(error = %err, "failed to save screenshot"),
        }
    }
}
