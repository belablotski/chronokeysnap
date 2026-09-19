//! Cross-platform screen capture, producing in-memory PNG-encoded screenshots.

use std::io::Cursor;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use image::{DynamicImage, ImageFormat as EncodedFormat};
use xcap::Monitor;

use crate::storage::{CaptureItem, ImageFormat};

/// Capture every connected monitor, returning one `CaptureItem` per monitor.
pub fn capture_all_monitors() -> Result<Vec<CaptureItem>> {
    let monitors = Monitor::all().context("failed to enumerate monitors")?;
    let timestamp = Utc::now();

    monitors
        .iter()
        .map(|monitor| capture_monitor(monitor, timestamp))
        .collect()
}

fn capture_monitor(monitor: &Monitor, timestamp: DateTime<Utc>) -> Result<CaptureItem> {
    let id = monitor
        .id()
        .map(|id| id.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let image = monitor
        .capture_image()
        .with_context(|| format!("failed to capture monitor {id}"))?;

    let mut bytes = Vec::new();
    DynamicImage::ImageRgba8(image)
        .write_to(&mut Cursor::new(&mut bytes), EncodedFormat::Png)
        .context("failed to encode screenshot as PNG")?;

    Ok(CaptureItem {
        image: bytes,
        format: ImageFormat::Png,
        timestamp,
        monitor_id: Some(id),
    })
}
