//! Manual trigger: a loopback-only TCP listener that requests one capture per
//! connection. Exists for platforms where OS-level global hotkeys don't work
//! (see README's "Known limitations", e.g. ChromeOS Crostini). Run with
//! `--capture` to connect and trigger a capture from an already-running instance.
//!
//! No authentication: any local process can connect, but the socket is bound to
//! 127.0.0.1 only, so it's unreachable from other machines.

use std::net::{Ipv4Addr, SocketAddr};

use anyhow::{Context, Result};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tracing::{info, warn};

use super::CaptureEvent;

/// Bind the loopback listener synchronously (so bind failures surface as a
/// startup error) and spawn a task to accept connections and forward events.
pub fn spawn(port: u16, tx: Sender<CaptureEvent>) -> Result<()> {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let std_listener = std::net::TcpListener::bind(addr)
        .with_context(|| format!("failed to bind manual trigger listener on 127.0.0.1:{port}"))?;
    std_listener
        .set_nonblocking(true)
        .context("failed to set manual trigger listener non-blocking")?;
    let listener = TcpListener::from_std(std_listener)
        .context("failed to hand off manual trigger listener to the async runtime")?;
    info!(
        port,
        "manual trigger listening on 127.0.0.1 (run with --capture to trigger)"
    );

    tokio::spawn(async move {
        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(err) => {
                    warn!(error = %err, "manual trigger accept failed");
                    continue;
                }
            };
            // Content doesn't matter; any connection requests one capture.
            let mut buf = [0u8; 16];
            let _ = socket.read(&mut buf).await;
            if tx.send(CaptureEvent::Manual).await.is_err() {
                break;
            }
        }
    });

    Ok(())
}
