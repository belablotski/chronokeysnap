//! Storage abstraction: the `Storage` trait plus concrete backends.
//!
//! `local` is the only backend for now; future backends (Azure Blob, S3, GCS)
//! will live alongside it behind Cargo feature flags.

pub mod local;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Image encoding of a captured screenshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
}

impl ImageFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
        }
    }
}

/// A captured screenshot ready to be persisted.
#[derive(Debug, Clone)]
pub struct CaptureItem {
    pub image: Vec<u8>,
    pub format: ImageFormat,
    pub timestamp: DateTime<Utc>,
    pub monitor_id: Option<String>,
}

/// Where a `CaptureItem` ended up after being saved (path, URL, object key, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageLocation(pub String);

/// Backend-agnostic destination for captured screenshots.
#[async_trait]
pub trait Storage: Send + Sync {
    async fn save(&self, item: &CaptureItem) -> anyhow::Result<StorageLocation>;
}
