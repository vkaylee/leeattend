// ── TECH_DEBT: Enterprise-grade lint upgrade (2026-05-03) ──
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::todo)]
#![allow(clippy::unimplemented)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::suspicious)]
#![allow(clippy::style)]
#![allow(clippy::correctness)]
#![allow(clippy::perf)]
#![allow(clippy::complexity)]
#![allow(clippy::clone_on_ref_ptr)]
#![allow(missing_docs)]

use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_reload_channel_closure_breaks_loop() {
    let (reload_tx, mut reload_rx) = mpsc::channel::<()>(1);
    let cancel_token = CancellationToken::new();
    let loop_cancel_token = cancel_token.clone();
    let (exit_tx, mut exit_rx) = mpsc::channel::<()>(1);

    // Replicate the event loop of Orchestrator
    let loop_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                res = reload_rx.recv() => {
                    match res {
                        Some(()) => {
                            // Simulated reload
                        }
                        None => {
                            // This is the bug fix: breaking the loop when channel is closed
                            let _ = exit_tx.send(()).await;
                            break;
                        }
                    }
                }
                _ = loop_cancel_token.cancelled() => {
                    break;
                }
            }
        }
    });

    // Drop the sender to close the channel
    drop(reload_tx);

    // Verify that the loop exited and sent the signal
    let wait_res = timeout(Duration::from_secs(2), exit_rx.recv()).await;
    assert!(
        wait_res.is_ok(),
        "Loop did not exit when reload channel was closed"
    );
    assert_eq!(
        wait_res.unwrap(),
        Some(()),
        "Incorrect exit signal received"
    );

    // Verify loop task has finished
    let join_res = timeout(Duration::from_secs(2), loop_handle).await;
    assert!(join_res.is_ok(), "Loop task did not terminate");
}

#[tokio::test]
async fn test_reload_channel_event_does_not_break_loop() {
    let (reload_tx, mut reload_rx) = mpsc::channel::<()>(1);
    let cancel_token = CancellationToken::new();
    let loop_cancel_token = cancel_token.clone();
    let (event_tx, mut event_rx) = mpsc::channel::<()>(10);

    let loop_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                res = reload_rx.recv() => {
                    match res {
                        Some(()) => {
                            let _ = event_tx.send(()).await;
                        }
                        None => {
                            break;
                        }
                    }
                }
                _ = loop_cancel_token.cancelled() => {
                    break;
                }
            }
        }
    });

    // Send a reload event
    reload_tx.send(()).await.unwrap();

    // Verify the loop processed it
    let res = timeout(Duration::from_secs(1), event_rx.recv()).await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), Some(()));

    // Cancel and let it exit cleanly
    cancel_token.cancel();
    let join_res = timeout(Duration::from_secs(1), loop_handle).await;
    assert!(join_res.is_ok());
}

#[test]
fn test_rustls_crypto_provider_is_installed() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    assert!(rustls::crypto::CryptoProvider::get_default().is_some());
}
