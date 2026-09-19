//! Local filesystem storage backend: saves each capture as a timestamped file
//! in a pre-configured folder.

use std::path::PathBuf;

use anyhow::{Context, Result};
use async_trait::async_trait;

use super::{CaptureItem, Storage, StorageLocation};

pub struct LocalStorage {
    folder: PathBuf,
}

impl LocalStorage {
    /// Create a backend rooted at `folder`, creating it if it doesn't exist yet.
    pub fn new(folder: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&folder)
            .with_context(|| format!("failed to create storage folder {}", folder.display()))?;
        Ok(Self { folder })
    }

    fn file_path(&self, item: &CaptureItem) -> PathBuf {
        let monitor_suffix = item
            .monitor_id
            .as_deref()
            .map(|id| format!("_{id}"))
            .unwrap_or_default();
        let file_name = format!(
            "{}{}.{}",
            item.timestamp.format("%Y-%m-%d_%H-%M-%S%.3f"),
            monitor_suffix,
            item.format.extension()
        );
        self.folder.join(file_name)
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn save(&self, item: &CaptureItem) -> Result<StorageLocation> {
        let path = self.file_path(item);
        let bytes = item.image.clone();
        tokio::fs::write(&path, bytes)
            .await
            .with_context(|| format!("failed to write screenshot to {}", path.display()))?;
        Ok(StorageLocation(path.display().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::ImageFormat;
    use chrono::{TimeZone, Utc};

    fn sample_item() -> CaptureItem {
        CaptureItem {
            image: vec![1, 2, 3, 4],
            format: ImageFormat::Png,
            timestamp: Utc.with_ymd_and_hms(2026, 9, 15, 14, 30, 5).unwrap(),
            monitor_id: None,
        }
    }

    #[tokio::test]
    async fn saves_file_with_timestamped_name() {
        let dir = tempfile_dir();
        let storage = LocalStorage::new(dir.clone()).unwrap();
        let item = sample_item();

        let location = storage.save(&item).await.unwrap();

        let expected_path = dir.join("2026-09-15_14-30-05.000.png");
        assert_eq!(location.0, expected_path.display().to_string());
        assert_eq!(std::fs::read(&expected_path).unwrap(), item.image);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn includes_monitor_id_in_file_name() {
        let dir = tempfile_dir();
        let storage = LocalStorage::new(dir.clone()).unwrap();
        let mut item = sample_item();
        item.monitor_id = Some("monitor-0".to_string());

        let location = storage.save(&item).await.unwrap();

        assert!(location.0.ends_with("_monitor-0.png"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn creates_folder_if_missing() {
        let root = tempfile_dir();
        let dir = root.join("nested").join("path");
        assert!(!dir.exists());
        LocalStorage::new(dir.clone()).unwrap();
        assert!(dir.exists());

        std::fs::remove_dir_all(&root).ok();
    }

    fn tempfile_dir() -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "chronokeysnap_test_{}_{n}",
            std::process::id()
        ))
    }
}
