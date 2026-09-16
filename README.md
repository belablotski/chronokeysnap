# ChronoKeySnap

A cross-platform Rust application that captures screenshots on a hotkey press or a
timer, and saves them to a configurable storage backend.

## Status

Early design stage. This document describes the intended functionality and
architecture before implementation begins.

## Functionality

- **Trigger a capture** in one of two ways:
  - **Hotkey** — a global, system-wide keyboard shortcut (works even when the app
    is in the background / minimized to the tray).
  - **Timer** — capture automatically every N seconds/minutes.
  - Both triggers can be enabled at the same time.
- **Capture the screen(s)** — the full display, a specific monitor, or (later) a
  selected region — as an in-memory image.
- **Save the screenshot** to a configured **storage backend**. The first version
  only supports the local filesystem; the storage layer is designed so that
  additional backends can be added without touching the capture/trigger logic.
- Run cross-platform: Windows, macOS, Linux.

## Non-goals (for v1)

- No image editing/annotation.
- No cloud backends yet (Azure/AWS/GCP are planned, not implemented).
- No GUI beyond a minimal system tray icon / config file (CLI + config-driven).

## Architecture

```mermaid
flowchart LR
    subgraph Triggers
        H[Hotkey Listener]
        T[Timer]
    end
    H --> E[Capture Event]
    T --> E
    E --> C[Screen Capture]
    C --> I[CaptureItem<br/>bytes + metadata]
    I --> S[Storage trait]
    S --> L[Local Filesystem]
    S -.future.-> A[Azure Blob Storage]
    S -.future.-> W[AWS S3]
    S -.future.-> G[Google Cloud Storage]
```

### Triggers

Hotkeys and the timer are independent sources that both emit the same internal
event when a capture should happen. This keeps the capture/storage pipeline
agnostic of *why* it was invoked.

- **Hotkey**: a global keyboard shortcut registered with the OS, configurable
  (e.g. `Ctrl+Shift+S`).
- **Timer**: a fixed interval, configurable (e.g. every 60s).

### Capture

Grabs the current screen contents into memory (no temp files) and produces a
`CaptureItem`:

```rust
struct CaptureItem {
    image: Vec<u8>,      // encoded image bytes (e.g. PNG)
    format: ImageFormat,  // Png, Jpeg, ...
    timestamp: DateTime<Utc>,
    monitor_id: Option<String>,
}
```

### Storage abstraction

The key extensibility point of the app. Storage is defined as a trait so any
backend can be plugged in behind it:

```rust
#[async_trait]
trait Storage: Send + Sync {
    /// Persist a captured item and return where it ended up (path, URL, key...).
    async fn save(&self, item: &CaptureItem) -> Result<StorageLocation>;
}
```

- **v1 — Local filesystem**: saves each `CaptureItem` as a file in a
  configurable, pre-defined folder, named from its timestamp
  (e.g. `2026-09-15_14-30-05.png`).
- **Planned — Azure Blob Storage**: uploads to a configured container.
- **Planned — AWS S3**: uploads to a configured bucket.
- **Planned — Google Cloud Storage**: uploads to a configured bucket.

Each backend lives behind its own Cargo feature flag (`storage-local` on by
default, `storage-azure`, `storage-aws`, `storage-gcs` opt-in) so cloud SDKs are
only compiled in when needed. The active backend is selected via configuration.

### Configuration

A single config file (TOML) controls trigger and storage settings, e.g.:

```toml
[trigger.hotkey]
enabled = true
combination = "Ctrl+Shift+S"

[trigger.timer]
enabled = false
interval_seconds = 60

[storage]
backend = "local"

[storage.local]
folder = "~/Pictures/ChronoKeySnap"
```

Future backend sections (`[storage.azure]`, `[storage.aws]`, `[storage.gcs]`)
will hold backend-specific settings (container/bucket name, region, credential
references, etc.) without changing the overall shape of the config.

## Planned project structure

```
src/
  main.rs        # orchestration / event loop
  config.rs      # app configuration (serde + toml)
  capture.rs     # screenshot capture (cross-platform)
  trigger/
    hotkey.rs    # global hotkey listener
    timer.rs     # interval-based trigger
  storage/
    mod.rs       # Storage trait + CaptureItem + StorageLocation
    local.rs     # local filesystem backend
```

As cloud backends are added, `storage/` may be split into a Cargo workspace
(`storage-core`, `storage-local`, `storage-azure`, `storage-aws`, `storage-gcs`)
if the crate grows large enough to warrant it.

## Candidate crates (to be validated during implementation)

- Screen capture: `xcap`
- Global hotkeys: `global-hotkey`
- Async runtime: `tokio`
- Image encoding: `image`
- Config parsing: `serde`, `toml`
- CLI: `clap`
- Logging: `tracing`

## License

Apache License 2.0 — see [LICENSE](LICENSE).
