// ── TECH_DEBT: Enterprise-grade lint upgrade (2026-05-03) ──
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::clone_on_ref_ptr)]
#![allow(missing_docs)]

//! Proves the sync loop rate-limits its maintenance sweeps.
//!
//! `move_expired_logs_to_dlq` scans and deserializes the entire active log
//! table. Before the time gate it ran on every iteration — up to 20×/second at
//! the backlogged sleep tier (`SYNC_BACKLOGGED_INTERVAL_MS=50`). These tests
//! drive the real `CentralClient` sync loop through a counting store and assert
//! the sweep fires exactly once despite many iterations.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use agent::central_client::CentralClient;
use agent::database_store::DatabaseStore;
use agent::hybrid_database::HybridDatabase;
use agent::models::AttendanceData;
use agent::redb_store::{StoreError, StoredDevice, StoredLog};
use agent::time_sync::TimeSync;
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::StreamExt;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_util::sync::CancellationToken;

/// Decorates a real store, tallying the calls the sync loop makes per iteration.
struct CountingStore {
    inner: HybridDatabase,
    dlq_calls: Arc<AtomicUsize>,
    stale_calls: Arc<AtomicUsize>,
    count_unsynced_calls: Arc<AtomicUsize>,
    get_unsynced_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl DatabaseStore for CountingStore {
    async fn save_raw_attendance(
        &self,
        device_id: String,
        data: AttendanceData,
    ) -> Result<i64, StoreError> {
        self.inner.save_raw_attendance(device_id, data).await
    }

    async fn save_raw_attendance_batch(
        &self,
        batch: Vec<(Arc<String>, AttendanceData)>,
    ) -> Result<usize, StoreError> {
        self.inner.save_raw_attendance_batch(batch).await
    }

    fn get_msg_id(&self, id: i64) -> Option<uuid::Uuid> {
        self.inner.store().get_msg_id(id)
    }

    async fn get_unsynced_logs(&self, limit: usize) -> Result<Vec<StoredLog>, StoreError> {
        self.get_unsynced_calls.fetch_add(1, Ordering::SeqCst);
        self.inner.get_unsynced_logs(limit).await
    }

    async fn count_unsynced_logs(&self) -> usize {
        self.count_unsynced_calls.fetch_add(1, Ordering::SeqCst);
        self.inner.count_unsynced_logs().await
    }

    async fn mark_in_flight(&self, ids: Vec<i64>) {
        self.inner.mark_in_flight(ids).await;
    }

    async fn mark_synced(&self, ids: Vec<i64>) {
        self.inner.mark_synced(ids).await;
    }

    async fn reset_to_unsynced(&self, id: i64) {
        self.inner.reset_to_unsynced(id).await;
    }

    async fn mark_synced_by_key(&self, composite_key: String) -> bool {
        self.inner.mark_synced_by_key(composite_key).await
    }

    async fn reset_to_unsynced_by_key(&self, composite_key: String) -> bool {
        self.inner.reset_to_unsynced_by_key(composite_key).await
    }

    async fn evict_old_signatures(&self) -> (usize, usize, usize) {
        self.inner.evict_old_signatures().await
    }

    async fn unload_device_memory(&self, device_id: String) -> Result<(), StoreError> {
        self.inner.unload_device_memory(device_id).await
    }

    fn get_active_devices(&self) -> Result<Vec<StoredDevice>, StoreError> {
        self.inner.get_active_devices()
    }

    fn get_agent_config(&self, key: &str) -> Result<Option<String>, StoreError> {
        self.inner.get_agent_config(key)
    }

    fn upsert_agent_config(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.inner.upsert_agent_config(key, value)
    }

    fn deactivate_devices_not_in(&self, device_ids: &[String]) -> Result<u64, StoreError> {
        self.inner.deactivate_devices_not_in(device_ids)
    }

    fn upsert_device(
        &self,
        id: &str,
        name: &str,
        ip: &str,
        port: i32,
        brand: &str,
        timezone: &str,
        password: Option<&str>,
        is_enabled: bool,
        now_ts: i64,
    ) -> Result<(), StoreError> {
        self.inner.upsert_device(
            id, name, ip, port, brand, timezone, password, is_enabled, now_ts,
        )
    }

    fn get_device_metadata(&self, device_id: &str) -> Result<Option<StoredDevice>, StoreError> {
        self.inner.get_device_metadata(device_id)
    }

    fn update_device_metadata(
        &self,
        device_id: &str,
        mac: &str,
        serial: &str,
        firmware: &str,
        platform: &str,
        hw_name: &str,
        users_count: i64,
        logs_count: i64,
        users_capacity: i64,
        logs_capacity: i64,
        device_time: &str,
        now_ts: i64,
    ) -> Result<(), StoreError> {
        self.inner.update_device_metadata(
            device_id,
            mac,
            serial,
            firmware,
            platform,
            hw_name,
            users_count,
            logs_count,
            users_capacity,
            logs_capacity,
            device_time,
            now_ts,
        )
    }

    fn update_device_checksum(
        &self,
        device_id: &str,
        use_legacy_checksum: bool,
    ) -> Result<(), StoreError> {
        self.inner
            .update_device_checksum(device_id, use_legacy_checksum)
    }

    fn update_device_counts(
        &self,
        device_id: &str,
        users_count: i64,
        logs_count: i64,
        now_ts: i64,
    ) -> Result<(), StoreError> {
        self.inner
            .update_device_counts(device_id, users_count, logs_count, now_ts)
    }

    fn move_expired_logs_to_dlq(&self, max_retries: i64) -> Result<u64, StoreError> {
        self.dlq_calls.fetch_add(1, Ordering::SeqCst);
        self.inner.move_expired_logs_to_dlq(max_retries)
    }

    fn reset_stale_syncing(&self, stale_timeout_secs: i64) -> Result<u64, StoreError> {
        self.stale_calls.fetch_add(1, Ordering::SeqCst);
        self.inner.reset_stale_syncing(stale_timeout_secs)
    }

    fn cleanup_old_synced_logs(&self, retention_days: i64) -> Result<u64, StoreError> {
        self.inner.cleanup_old_synced_logs(retention_days)
    }

    fn compact(&self) -> Result<bool, StoreError> {
        self.inner.compact()
    }

    fn recover_from_crash(&self) -> Result<u64, StoreError> {
        self.inner.recover_from_crash()
    }
}

/// Mock server that ACKs nothing, so the backlog persists and the loop keeps
/// spinning at its fastest tier.
async fn start_silent_ws_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                if let Ok(mut ws) = accept_async(stream).await {
                    // Drain incoming frames without ACKing.
                    while let Some(Ok(_)) = ws.next().await {}
                    let _ = ws.close(None).await;
                }
            });
        }
    });

    port
}

struct Harness {
    dlq_calls: Arc<AtomicUsize>,
    stale_calls: Arc<AtomicUsize>,
    count_unsynced_calls: Arc<AtomicUsize>,
    get_unsynced_calls: Arc<AtomicUsize>,
    cancel: CancellationToken,
}

/// Spin up the real sync loop against a silent server with `dlq_interval_secs`.
async fn run_sync_loop(dlq_interval_secs: u64, logs: usize, run_for: Duration) -> Harness {
    let port = start_silent_ws_server().await;
    let url = format!("ws://127.0.0.1:{port}");

    let hybrid = HybridDatabase::new(":memory:", 10_000, 90).unwrap();
    hybrid
        .store()
        .upsert_device(
            "DEV_GATE", "Gate", "1.1.1.1", 4370, "zkteco", "UTC", None, true, 123,
        )
        .unwrap();

    // Seed a backlog so the loop takes the "has work" path every iteration.
    for i in 0..logs {
        let data = AttendanceData {
            user_id: format!("U{i}"),
            timestamp: "2026-01-01 10:00:00".into(),
            status: Some(1),
            punch: Some(1),
        };
        hybrid
            .store()
            .insert("DEV_GATE", &data, 1_000 + i as i64)
            .unwrap();
    }

    let dlq_calls = Arc::new(AtomicUsize::new(0));
    let stale_calls = Arc::new(AtomicUsize::new(0));
    let count_unsynced_calls = Arc::new(AtomicUsize::new(0));
    let get_unsynced_calls = Arc::new(AtomicUsize::new(0));

    let store = CountingStore {
        inner: hybrid,
        dlq_calls: dlq_calls.clone(),
        stale_calls: stale_calls.clone(),
        count_unsynced_calls: count_unsynced_calls.clone(),
        get_unsynced_calls: get_unsynced_calls.clone(),
    };

    let cancel = CancellationToken::new();
    let (client, central_rx, _reload_rx) = CentralClient::new(
        url,
        "AGENT_GATE".to_string(),
        "TOKEN".to_string(),
        Arc::new(store) as Arc<dyn DatabaseStore>,
        30,
        Arc::new(DashMap::new()),
        Arc::new(TimeSync::new()),
        cancel.clone(),
        10, // small batch → many iterations to drain
        20, // idle interval (ms)
        20, // backlogged interval (ms)
        20, // moderate interval (ms)
        20, // normal interval (ms)
        dlq_interval_secs,
    );

    tokio::spawn(client.start(central_rx));

    tokio::time::sleep(run_for).await;
    cancel.cancel();
    tokio::time::sleep(Duration::from_millis(100)).await;

    Harness {
        dlq_calls,
        stale_calls,
        count_unsynced_calls,
        get_unsynced_calls,
        cancel,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dlq_sweep_not_called_on_second_iteration() {
    // 300s interval: only the very first iteration may sweep.
    let h = run_sync_loop(300, 60, Duration::from_millis(1200)).await;

    let iterations = h.get_unsynced_calls.load(Ordering::SeqCst);
    let dlq = h.dlq_calls.load(Ordering::SeqCst);

    assert!(
        iterations >= 3,
        "loop must iterate several times for this test to be meaningful (got {iterations})"
    );
    assert_eq!(
        dlq, 1,
        "DLQ sweep must run once at startup and never again within the interval \
         (ran {dlq}× across {iterations} iterations)"
    );
    drop(h.cancel);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_stale_reset_also_gated() {
    // The stale reset gate is 60s, so it likewise fires only once here.
    let h = run_sync_loop(300, 60, Duration::from_millis(1200)).await;

    let iterations = h.get_unsynced_calls.load(Ordering::SeqCst);
    assert!(iterations >= 3, "expected multiple iterations");
    assert_eq!(
        h.stale_calls.load(Ordering::SeqCst),
        1,
        "stale in-flight reset must stay gated to once per 60s"
    );
    drop(h.cancel);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_count_unsynced_read_at_most_once_per_iteration() {
    // Item 2: the "Pending" log line and the adaptive-sleep tier now share a
    // single read, so counts must not exceed one per iteration.
    let h = run_sync_loop(300, 60, Duration::from_millis(1200)).await;

    let iterations = h.get_unsynced_calls.load(Ordering::SeqCst);
    let counts = h.count_unsynced_calls.load(Ordering::SeqCst);

    assert!(iterations >= 3, "expected multiple iterations");
    assert!(
        counts <= iterations,
        "count_unsynced_logs called {counts}× over {iterations} iterations — \
         must be at most once per iteration (was twice before this fix)"
    );
    drop(h.cancel);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dlq_sweep_runs_again_after_interval_elapses() {
    // A 1-second interval over a ~2.5s run must fire more than once, proving the
    // gate delays the sweep rather than permanently suppressing it.
    let h = run_sync_loop(1, 40, Duration::from_millis(2500)).await;

    let dlq = h.dlq_calls.load(Ordering::SeqCst);
    let iterations = h.get_unsynced_calls.load(Ordering::SeqCst);

    assert!(
        dlq >= 2,
        "sweep must resume after the interval elapses (ran {dlq}× over {iterations} iterations)"
    );
    assert!(
        dlq < iterations,
        "sweep still ran every iteration ({dlq} of {iterations}) — gate ineffective"
    );
    drop(h.cancel);
}
