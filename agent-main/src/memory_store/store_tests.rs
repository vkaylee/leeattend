#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_store::{InMemoryStore, StoreError};
    use crate::memory_store::types::MemoryLog;
    use crate::models::{AttendanceData, RawAttendanceLog, SignatureRow};
    use chrono::NaiveDateTime;
    use std::sync::Arc;

    fn ts(timestamp_str: &str) -> i64 {
        NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
            .expect("Invalid timestamp format")
            .and_utc()
            .timestamp()
    }

    #[test]
    fn test_new_store() {
        let store = InMemoryStore::new(1000);
        assert_eq!(store.count_total(), 0);
        assert_eq!(store.count_unsynced(), 0);
        assert!(!store.is_dirty());
    }

    #[test]
    fn test_insert() {
        let store = InMemoryStore::new(1000);
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };

        let id = store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        assert_eq!(id, 1);
        assert_eq!(store.count_total(), 1);
        assert_eq!(store.count_unsynced(), 1);
        assert!(store.is_dirty());
    }

    #[test]
    fn test_insert_capacity_exceeded() {
        let store = InMemoryStore::new(2);

        // Insert 2 logs (should succeed)
        for i in 1..=2 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        }

        // Insert 3rd log (should fail)
        let data = AttendanceData {
            user_id: "U3".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let result = store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00"));
        assert!(matches!(result, Err(StoreError::CapacityExceeded)));
    }

    #[test]
    fn test_get_unsynced_fifo_order() {
        let store = InMemoryStore::new(1000);

        // Insert logs in order
        let users = vec!["FIRST", "SECOND", "THIRD"];
        for user in &users {
            let data = AttendanceData {
                user_id: user.to_string(),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        }

        // Get unsynced logs
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 3);

        // Verify FIFO order
        assert_eq!(&*logs[0].user_id, "FIRST");
        assert_eq!(&*logs[1].user_id, "SECOND");
        assert_eq!(&*logs[2].user_id, "THIRD");

        // Verify IDs are ascending
        assert!(logs[1].id > logs[0].id);
        assert!(logs[2].id > logs[1].id);
    }

    #[test]
    fn test_mark_in_flight() {
        let store = InMemoryStore::new(1000);

        // Insert 3 logs
        for i in 1..=3 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        }

        // Get logs and mark as in-flight
        let logs = store.get_unsynced(2);
        assert_eq!(logs.len(), 2);
        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();

        store.mark_in_flight(&ids);

        // Verify unsynced count decreased
        assert_eq!(store.count_unsynced(), 1);

        // Verify in-flight logs are not returned
        let remaining = store.get_unsynced(10);
        assert_eq!(remaining.len(), 1);
    }

    #[test]
    fn test_mark_synced() {
        let store = InMemoryStore::new(1000);

        // Insert and mark as in-flight
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        assert_eq!(store.count_total(), 1);
        store.mark_in_flight(&[id]);

        // Persist to disk first (required before sync removal)
        store.mark_persisted(&[id]);

        // Mark as synced - log should be REMOVED from memory
        store.mark_synced(&[id]);

        // Verify log is removed (not just synced)
        assert!(store.logs.get(&id).is_none(), "Log should be removed after sync");
        assert_eq!(store.count_total(), 0, "Count should be 0 after sync");
        assert_eq!(store.count_unsynced(), 0);

        // Signature should still exist for dedup
        assert_eq!(store.count_signatures(), 1, "Signature preserved for dedup");
    }

    #[test]
    fn test_reset_to_unsynced() {
        let store = InMemoryStore::new(1000);

        // Insert and mark as in-flight
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        store.mark_in_flight(&[id]);
        assert_eq!(store.count_unsynced(), 0);

        // Reset to unsynced
        store.reset_to_unsynced(id);

        // Verify log is back to unsynced
        let log = store.logs.get(&id).unwrap();
        assert_eq!(log.is_synced, 0);
        assert_eq!(store.count_unsynced(), 1);
    }

    #[test]
    fn test_insert_batch() {
        let store = InMemoryStore::new(1000);

        let batch = vec![
            (Arc::new("D1".to_string()), AttendanceData {
                user_id: "U1".to_string(),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            }, ts("2026-01-29 10:00:00")),
            (Arc::new("D1".to_string()), AttendanceData {
                user_id: "U2".to_string(),
                timestamp: "2026-01-29 10:01:00".to_string(),
                status: Some(1),
                punch: Some(1),
            }, ts("2026-01-29 10:01:00")),
            (Arc::new("D1".to_string()), AttendanceData {
                user_id: "U3".to_string(),
                timestamp: "2026-01-29 10:02:00".to_string(),
                status: Some(1),
                punch: Some(1),
            }, ts("2026-01-29 10:02:00")),
        ];

        let result = store.insert_batch(batch);
        assert_eq!(result.unwrap(), 3);
        assert_eq!(store.count_total(), 3);
        assert_eq!(store.count_unsynced(), 3);
    }

    #[test]
    fn test_get_unpersisted() {
        let store = InMemoryStore::new(1000);

        // Insert 2 logs (unpersisted)
        for i in 1..=2 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        }

        // All logs should be unpersisted
        let unpersisted = store.get_unpersisted();
        assert_eq!(unpersisted.len(), 2);

        // Mark one as persisted
        store.mark_persisted(&[1]);
        let unpersisted = store.get_unpersisted();
        assert_eq!(unpersisted.len(), 1);
    }

    #[test]
    fn test_cleanup_synced() {
        // With immediate removal in mark_synced, cleanup_synced becomes a no-op
        // This test verifies that cleanup handles the new behavior correctly
        let store = InMemoryStore::new(1000);

        // Insert 3 logs
        for i in 1..=3 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        }

        assert_eq!(store.count_total(), 3);

        // Mark first 2 as synced - they are IMMEDIATELY removed
        store.mark_in_flight(&[1, 2]);
        store.mark_persisted(&[1, 2]); // Persist before sync
        store.mark_synced(&[1, 2]);

        // Logs are already removed by mark_synced
        assert_eq!(store.count_total(), 1, "Only 1 unsynced log remains");

        // cleanup_synced should find nothing to clean
        let removed = store.cleanup_synced(0);
        assert_eq!(removed, 0, "No synced logs to clean (already removed by mark_synced)");

        // Only unsynced log remains
        assert_eq!(store.count_total(), 1);
    }

    #[test]
    fn test_load_from_disk() {
        let store = InMemoryStore::new(1000);

        let logs = vec![
            RawAttendanceLog {
                id: 1,
                device_id: "D1".to_string(),
                user_id: "U1".to_string(),
                timestamp: ts("2026-01-29 10:00:00"),
                status: Some(1),
                punch: Some(1),
                is_synced: 0,
                created_at: Some(chrono::Utc::now().naive_utc()),
                updated_at: None,
                retry_count: 0,
            },
            RawAttendanceLog {
                id: 2,
                device_id: "D1".to_string(),
                user_id: "U2".to_string(),
                timestamp: ts("2026-01-29 10:01:00"),
                status: Some(1),
                punch: Some(1),
                is_synced: 2,
                created_at: Some(chrono::Utc::now().naive_utc()),
                updated_at: Some(chrono::Utc::now().naive_utc()),
                retry_count: 1,
            },
        ];

        store.load_from_disk(logs);

        assert_eq!(store.count_total(), 2);
        assert_eq!(store.count_unsynced(), 1); // Only first log is unsynced

        // Verify next_id is set correctly
        let data = AttendanceData {
            user_id: "U3".to_string(),
            timestamp: "2026-01-29 10:02:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        assert_eq!(id, 3); // Should continue from max_id + 1
    }

    #[test]
    fn test_concurrent_insert_and_mark() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(InMemoryStore::new(10000));
        let mut handles = vec![];

        // Insert thread
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            for i in 0..50 {
                let data = AttendanceData {
                    user_id: format!("U{i:03}"),
                    timestamp: "2026-01-29 10:00:00".to_string(),
                    status: Some(1),
                    punch: Some(1),
                };
                store_clone.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
            }
        });
        handles.push(handle);

        // Mark in-flight thread
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let logs = store_clone.get_unsynced(25);
            let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
            store_clone.mark_in_flight(&ids);
        });
        handles.push(handle);

        // Mark synced thread (removes synced logs from memory)
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            let logs = store_clone.get_unsynced(10);
            let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
            store_clone.mark_synced(&ids);
        });
        handles.push(handle);

        for handle in handles {
            handle.join().unwrap();
        }

        // Total = 50 inserted minus synced (up to 10)
        // Due to race conditions, synced count varies (0-10)
        let total = store.count_total();
        assert!((40..=50).contains(&total), "Expected 40-50 logs, got {total}");
    }

    #[test]
    fn test_reset_stale_inflight() {
        let store = InMemoryStore::new(1000);

        // Insert and mark in-flight
        for i in 1..=3 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        }

        let logs = store.get_unsynced(3);
        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        store.mark_in_flight(&ids);

        assert_eq!(store.count_unsynced(), 0);

        // Reset stale (with 0 timeout)
        let reset = store.reset_stale_inflight(std::time::Duration::from_secs(0));

        assert_eq!(reset, 3);
        assert_eq!(store.count_unsynced(), 3);
    }

    #[test]
    fn test_seed_id() {
        let store = InMemoryStore::new(1000);

        // Seed with starting ID 1000
        store.seed_id(1000);

        // Insert first log
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        // Should be at least 1000 + gap (1000 ID gap)
        assert!(id >= 2000, "ID {id} should be >= 2000");
    }

    #[test]
    fn test_multiple_mark_synced_calls() {
        let store = InMemoryStore::new(1000);

        // Insert 5 logs
        let mut ids = Vec::new();
        for i in 1..=5 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            let id = store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
            ids.push(id);
        }

        // Mark in-flight
        store.mark_in_flight(&ids);

        // Persist all logs first
        store.mark_persisted(&ids);

        // Mark synced in two batches - logs are immediately removed
        store.mark_synced(&ids[0..2]);
        assert_eq!(store.count_total(), 3, "2 synced logs removed");

        store.mark_synced(&ids[2..5]);
        assert_eq!(store.count_total(), 0, "All logs removed after sync");

        // All signatures preserved
        assert_eq!(store.count_signatures(), 5, "All signatures preserved");
    }

    #[test]
    fn test_get_unsynced_limit() {
        let store = InMemoryStore::new(1000);

        // Insert 10 logs
        for i in 1..=10 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        }

        // Get with limit
        let logs_3 = store.get_unsynced(3);
        let logs_5 = store.get_unsynced(5);
        let logs_all = store.get_unsynced(100);

        assert_eq!(logs_3.len(), 3);
        assert_eq!(logs_5.len(), 5);
        assert_eq!(logs_all.len(), 10);
    }

    #[test]
    fn test_invalid_timestamp_iso_format() {
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29T10:00:00Z".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        // Should accept ISO format
        let result = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_dirty_flag() {
        let store = InMemoryStore::new(1000);

        // Insert log (sets dirty)
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        assert!(store.is_dirty());

        // Clear dirty
        store.clear_dirty();

        assert!(!store.is_dirty());
    }

    #[test]
    fn test_get_all_logs() {
        let store = InMemoryStore::new(1000);

        // Insert 3 logs in different states
        let id1 = {
            let data = AttendanceData {
                user_id: "U1".to_string(),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap()
        };

        let id2 = {
            let data = AttendanceData {
                user_id: "U2".to_string(),
                timestamp: "2026-01-29 10:01:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap()
        };

        // Mark one as in-flight, one as synced (which removes it immediately)
        store.mark_in_flight(&[id1]);
        store.mark_persisted(&[id1]); // Persist before sync
        store.mark_synced(&[id1]); // Removed from memory
        store.mark_in_flight(&[id2]);

        // Get all should return only unsynced (synced logs removed immediately)
        let all = store.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, id2);
    }

    // ===== Exponential Backoff Tests =====

    #[test]
    fn test_backoff_new_logs_immediate() {
        let store = InMemoryStore::new(1000);

        // Insert new log (retry_count=0, updated_at=None)
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        // New log should be returned immediately
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_backoff_skips_recent_retry() {
        let store = InMemoryStore::new(1000);

        // Insert and simulate first retry
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        // Mark in-flight (sets retry_count=1, updated_at=now)
        store.mark_in_flight(&[id]);
        store.reset_to_unsynced(id);

        // Should NOT be returned (within 2s backoff)
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 0);
        assert_eq!(store.count_in_backoff(), 1);
    }

    #[test]
    fn test_backoff_allows_after_delay() {
        let store = InMemoryStore::new(1000);

        // Insert log
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        // Simulate retry with old updated_at (3 seconds ago)
        if let Some(mut log) = store.logs.get_mut(&id) {
            log.retry_count = 1;
            log.updated_at = Some(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(3));
        }

        // Should be returned (past 2s backoff for retry_count=1)
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_backoff_cap_at_300_seconds() {
        let store = InMemoryStore::new(1000);

        // Insert log with high retry count
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        // Simulate many retries (retry_count=10, beyond 2^8=256s)
        // With jitter, delay can be 300 ± 10% = 270-330s
        // Use 200s ago - definitely still in backoff (less than min 270s)
        if let Some(mut log) = store.logs.get_mut(&id) {
            log.retry_count = 10;
            log.updated_at = Some(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(200));
        }

        // Should NOT be returned (200s < min backoff 270s)
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 0);

        // After 400 seconds (well past max 330s with jitter), should be eligible
        if let Some(mut log) = store.logs.get_mut(&id) {
            log.updated_at = Some(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(400));
        }
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_backoff_new_logs_prioritized() {
        let store = InMemoryStore::new(1000);

        // Insert retry log (in backoff)
        let data1 = AttendanceData {
            user_id: "RETRY".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id1 = store.insert("D1".to_string(), data1, ts("2026-01-29 10:00:00")).unwrap();
        store.mark_in_flight(&[id1]);
        store.reset_to_unsynced(id1);

        // Insert new log
        let data2 = AttendanceData {
            user_id: "NEW".to_string(),
            timestamp: "2026-01-29 10:00:01".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        store.insert("D1".to_string(), data2, ts("2026-01-29 10:00:00")).unwrap();

        // Only NEW should be returned (RETRY is in backoff)
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 1);
        assert_eq!(&*logs[0].user_id, "NEW");
    }

    // ===== Phase 1: Signature Tests =====

    #[test]
    fn test_compute_signature_deterministic() {
        let timestamp_unix = ts("2026-01-29 10:00:00");

        // Multiple calls with same input
        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", timestamp_unix, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV1", "USER001", timestamp_unix, Some(0), Some(1));
        let sig3 = InMemoryStore::compute_signature("DEV1", "USER001", timestamp_unix, Some(0), Some(1));

        assert_eq!(sig1, sig2);
        assert_eq!(sig2, sig3);
    }

    #[test]
    fn test_compute_signature_different_devices() {
        let timestamp_unix = ts("2026-01-29 10:00:00");

        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", timestamp_unix, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV2", "USER001", timestamp_unix, Some(0), Some(1));

        assert_ne!(sig1, sig2, "Different device should produce different signature");
    }

    #[test]
    fn test_compute_signature_different_users() {
        let timestamp_unix = ts("2026-01-29 10:00:00");

        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", timestamp_unix, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV1", "USER002", timestamp_unix, Some(0), Some(1));

        assert_ne!(sig1, sig2, "Different user should produce different signature");
    }

    #[test]
    fn test_compute_signature_different_timestamps() {
        let ts1 = ts("2026-01-29 10:00:00");
        let ts2 = ts("2026-01-29 10:00:01");

        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", ts1, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV1", "USER001", ts2, Some(0), Some(1));

        assert_ne!(sig1, sig2, "Different timestamp should produce different signature");
    }

    #[test]
    fn test_compute_signature_different_punch() {
        let timestamp_unix = ts("2026-01-29 10:00:00");

        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", timestamp_unix, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV1", "USER001", timestamp_unix, Some(1), Some(1));

        assert_ne!(sig1, sig2, "Different punch should produce different signature");
    }

    #[test]
    fn test_compute_signature_different_status() {
        let timestamp_unix = ts("2026-01-29 10:00:00");

        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", timestamp_unix, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV1", "USER001", timestamp_unix, Some(0), Some(2));

        assert_ne!(sig1, sig2, "Different status should produce different signature");
    }

    #[test]
    fn test_signature_count() {
        let store = InMemoryStore::new(1000);
        assert_eq!(store.count_signatures(), 0);
    }

    // ===== Phase 2: Deduplication Tests =====

    #[test]
    fn test_insert_rejects_exact_duplicate() {
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        // First insert succeeds
        let result1 = store.insert("DEV1".to_string(), data.clone(), ts("2026-01-29 10:00:00"));
        assert!(result1.is_ok());

        // Second insert fails
        let result2 = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00"));
        assert!(matches!(result2, Err(StoreError::Duplicate)));

        // Only 1 log in store
        assert_eq!(store.count_total(), 1);
        assert_eq!(store.count_signatures(), 1);
    }

    #[test]
    fn test_insert_allows_different_timestamp() {
        let store = InMemoryStore::new(1000);

        let data1 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let data2 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 10:00:01".to_string(), // 1 second later
            status: Some(1),
            punch: Some(0),
        };

        // Both should succeed
        assert!(store.insert("DEV1".to_string(), data1, ts("2026-01-29 10:00:00")).is_ok());
        assert!(store.insert("DEV1".to_string(), data2, ts("2026-01-29 10:00:01")).is_ok());

        assert_eq!(store.count_total(), 2);
        assert_eq!(store.count_signatures(), 2);
    }

    #[test]
    fn test_insert_allows_different_punch_same_timestamp() {
        let store = InMemoryStore::new(1000);

        // Same device, user, timestamp, status - but different punch
        let data1 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 08:00:00".to_string(),
            status: Some(1),
            punch: Some(0), // check-in
        };
        let data2 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 08:00:00".to_string(), // Same timestamp
            status: Some(1),
            punch: Some(1), // check-out
        };

        // Both should succeed - this is the critical bug fix
        let result1 = store.insert("DEV1".to_string(), data1, ts("2026-01-29 10:00:00"));
        assert!(result1.is_ok(), "First insert (punch=0) should succeed");

        let result2 = store.insert("DEV1".to_string(), data2, ts("2026-01-29 10:00:00"));
        assert!(result2.is_ok(), "Second insert (punch=1) should succeed - NOT duplicate");

        assert_eq!(store.count_total(), 2, "Both logs should be stored");
        assert_eq!(store.count_signatures(), 2, "Both signatures should be stored");
    }

    #[test]
    fn test_insert_allows_different_status_same_timestamp() {
        let store = InMemoryStore::new(1000);

        // Same device, user, timestamp, punch - but different status
        let data1 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 08:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let data2 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 08:00:00".to_string(), // Same timestamp
            status: Some(2), // Different status
            punch: Some(0),
        };

        // Both should succeed
        assert!(store.insert("DEV1".to_string(), data1, ts("2026-01-29 10:00:00")).is_ok());
        assert!(store.insert("DEV1".to_string(), data2, ts("2026-01-29 10:00:00")).is_ok());

        assert_eq!(store.count_total(), 2);
        assert_eq!(store.count_signatures(), 2);
    }

    #[test]
    fn test_mark_synced_preserves_signatures() {
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        // Insert and sync (immediate removal)
        let id = store.insert("DEV1".to_string(), data.clone(), ts("2026-01-29 10:00:00")).unwrap();
        store.mark_in_flight(&[id]);
        store.mark_persisted(&[id]); // Persist before sync
        store.mark_synced(&[id]);

        // Log immediately removed by mark_synced
        assert_eq!(store.count_total(), 0);
        assert_eq!(store.count_signatures(), 1, "Signature preserved");

        // Duplicate still rejected
        let result = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00"));
        assert!(matches!(result, Err(StoreError::Duplicate)));
    }

    #[test]
    fn test_batch_insert_handles_duplicates() {
        let store = InMemoryStore::new(1000);

        let batch = vec![
            (Arc::new("D1".to_string()), AttendanceData {
                user_id: "U1".to_string(),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(0),
            }, ts("2026-01-29 10:00:00")),
            (Arc::new("D1".to_string()), AttendanceData {
                user_id: "U1".to_string(),
                timestamp: "2026-01-29 10:00:00".to_string(), // Duplicate of first
                status: Some(1),
                punch: Some(0),
            }, ts("2026-01-29 10:00:00")),
            (Arc::new("D1".to_string()), AttendanceData {
                user_id: "U3".to_string(),
                timestamp: "2026-01-29 10:02:00".to_string(),
                status: Some(1),
                punch: Some(1),
            }, ts("2026-01-29 10:02:00")),
        ];

        let result = store.insert_batch(batch);
        assert_eq!(result.unwrap(), 2, "Should insert 2 unique logs, skipping 1 duplicate");
        assert_eq!(store.count_total(), 2);
        assert_eq!(store.count_unsynced(), 2);
    }

    #[test]


    #[test]
    fn test_load_from_disk_populates_signatures() {
        let store = InMemoryStore::new(1000);
        let timestamp_unix = ts("2026-01-29 10:00:00");

        let logs = vec![
            RawAttendanceLog {
                id: 1,
                device_id: "D1".to_string(),
                user_id: "U1".to_string(),
                timestamp: timestamp_unix,
                status: Some(1),
                punch: Some(1),
                is_synced: 0,
                created_at: Some(chrono::Utc::now().naive_utc()),
                updated_at: None,
                retry_count: 0,
            },
        ];

        store.load_from_disk(logs);

        // Signature should be populated
        assert_eq!(store.count_signatures(), 1);

        // Insert same log should be rejected
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let result = store.insert("D1".to_string(), data, ts("2026-01-29 10:00:00"));
        assert!(matches!(result, Err(StoreError::Duplicate)));
    }

    #[test]
    fn test_concurrent_inserts_no_duplicates() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(InMemoryStore::new(10000));
        let mut handles = vec![];

        // 10 threads trying to insert same log
        for _ in 0..10 {
            let store_clone = store.clone();
            let handle = thread::spawn(move || {
                let data = AttendanceData {
                    user_id: "CONCURRENT_USER".to_string(),
                    timestamp: "2026-01-29 12:00:00".to_string(),
                    status: Some(1),
                    punch: Some(0),
                };
                store_clone.insert("CONCURRENT_DEV".to_string(), data, ts("2026-01-29 10:00:00"))
            });
            handles.push(handle);
        }

        let mut success_count = 0;
        let mut duplicate_count = 0;

        for handle in handles {
            match handle.join().unwrap() {
                Ok(_) => success_count += 1,
                Err(StoreError::Duplicate) => duplicate_count += 1,
                Err(e) => panic!("Unexpected error: {e:?}"),
            }
        }

        // Exactly 1 should succeed, 9 should be duplicates
        assert_eq!(success_count, 1, "Exactly 1 insert should succeed");
        assert_eq!(duplicate_count, 9, "9 should be duplicates");
        assert_eq!(store.count_total(), 1);
        assert_eq!(store.count_signatures(), 1);
    }

    #[test]
    fn test_composite_key_format() {
        let log = MemoryLog {
            id: 1,
            device_id: "DEV1".into(),
            user_id: "U001".into(),
            timestamp: ts("2026-01-31 10:00:00"),
            status: Some(1),
            punch: Some(0),
            is_synced: 0,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: None,
            retry_count: 0,
            persisted: false,
        };

        let key = log.composite_key();
        assert!(key.starts_with("DEV1:U001:"));
        assert!(key.contains(&log.timestamp.to_string()));
        assert!(key.ends_with(":0:1"));
    }

    #[test]
    fn test_composite_index_lookup() {
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-31 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        // Construct expected composite key
        let timestamp_unix = ts("2026-01-29 10:00:00");
        let expected_key = format!("DEV1:U001:{timestamp_unix}:0:1");

        // Lookup should return the inserted ID
        assert_eq!(store.get_id_by_composite_key(&expected_key), Some(id));

        // Lookup non-existent key should return None
        assert_eq!(store.get_id_by_composite_key("INVALID:KEY:0"), None);
    }

    #[test]
    fn test_mark_synced_by_key() {
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-31 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        // Mark in-flight first
        store.mark_in_flight(&[id]);

        // Persist before sync
        store.mark_persisted(&[id]);

        // Construct composite key
        let timestamp_unix = ts("2026-01-29 10:00:00");
        let key = format!("DEV1:U001:{timestamp_unix}:0:1");

        // Mark synced by key should return true
        assert!(store.mark_synced_by_key(&key));

        // Verify log is REMOVED after sync (not just marked)
        assert!(store.logs.get(&id).is_none(), "Log should be removed after sync");
        assert_eq!(store.count_total(), 0);
    }

    #[test]
    fn test_reset_to_unsynced_by_key() {
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-31 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        // Mark in-flight
        store.mark_in_flight(&[id]);
        assert_eq!(store.count_unsynced(), 0);  // In-flight, not unsynced

        // Construct composite key
        let timestamp_unix = ts("2026-01-29 10:00:00");
        let key = format!("DEV1:U001:{timestamp_unix}:0:1");

        // Reset to unsynced by key
        assert!(store.reset_to_unsynced_by_key(&key));

        // Should be back in unsynced queue
        assert_eq!(store.count_unsynced(), 1);
    }

    #[test]
    fn test_composite_key_hash_consistency() {
        // Verify hash computation is deterministic
        let timestamp = ts("2026-01-31 10:00:00");

        let hash1 = MemoryLog::compute_composite_hash("DEV1", "U001", timestamp, Some(0), Some(1));
        let hash2 = MemoryLog::compute_composite_hash("DEV1", "U001", timestamp, Some(0), Some(1));

        assert_eq!(hash1, hash2, "Hash must be deterministic");
    }

    #[test]
    fn test_composite_key_hash_uniqueness() {
        // Verify different components produce different hashes
        let timestamp = ts("2026-01-31 10:00:00");

        let hash1 = MemoryLog::compute_composite_hash("DEV1", "U001", timestamp, Some(0), Some(1));
        let hash2 = MemoryLog::compute_composite_hash("DEV2", "U001", timestamp, Some(0), Some(1));  // Different device
        let hash3 = MemoryLog::compute_composite_hash("DEV1", "U002", timestamp, Some(0), Some(1));  // Different user

        assert_ne!(hash1, hash2, "Different devices must have different hashes");
        assert_ne!(hash1, hash3, "Different users must have different hashes");
        assert_ne!(hash2, hash3, "All combinations must be unique");
    }

    #[test]
    fn test_composite_key_hash_timestamp_sensitivity() {
        // Verify timestamp differences produce different hashes
        let ts1 = ts("2026-01-31 10:00:00");
        let ts2 = ts("2026-01-31 10:00:01");

        let hash1 = MemoryLog::compute_composite_hash("DEV1", "U001", ts1, Some(0), Some(1));
        let hash2 = MemoryLog::compute_composite_hash("DEV1", "U001", ts2, Some(0), Some(1));

        assert_ne!(hash1, hash2, "Different timestamps must have different hashes");
    }

    #[test]
    fn test_hash_based_lookup_after_insert() {
        // Verify hash-based lookup works correctly
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-31 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        // Construct composite key string
        let timestamp_unix = ts("2026-01-29 10:00:00");
        let key = format!("DEV1:U001:{timestamp_unix}:0:1");

        // Lookup by string should parse and compute hash internally
        assert_eq!(store.get_id_by_composite_key(&key), Some(id));
    }

    #[test]
    fn test_composite_key_with_colon_in_device_id() {
        // Verify device_id containing colons is handled correctly
        let store = InMemoryStore::new(1000);

        // Device name with colon (common in real deployments)
        let device_id = "Office:Main:Gate".to_string();
        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-31 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        let id = store.insert(device_id.clone(), data, ts("2026-01-29 10:00:00")).unwrap();

        // Construct composite key (will have multiple colons)
        let timestamp_unix = ts("2026-01-29 10:00:00");
        let key = format!("{device_id}:U001:{timestamp_unix}:0:1");
        // key = "Office:Main:Gate:U001:1738314000:0:1"

        // Lookup should work correctly with rsplit parsing
        assert_eq!(store.get_id_by_composite_key(&key), Some(id),
            "Device ID with colons should be parsed correctly");
    }

    #[test]
    fn test_hash_based_lookup_multiple_logs() {
        // Verify no hash collisions in typical usage
        let store = InMemoryStore::new(1000);

        let mut inserted_ids = Vec::new();
        let mut composite_keys = Vec::new();

        // Insert 100 logs with different device/user/timestamp combinations
        for i in 0..100 {
            let device = format!("DEV{}", i % 10);
            let user = format!("U{i:03}");
            let timestamp = format!("2026-01-31 10:{:02}:00", i % 60);

            let data = AttendanceData {
                user_id: user.clone(),
                timestamp: timestamp.clone(),
                status: Some(1),
                punch: Some(0),
            };

            let timestamp_unix = ts(&timestamp);
            let id = store.insert(device.clone(), data, timestamp_unix).unwrap();
            inserted_ids.push(id);

            let key = format!("{device}:{user}:{timestamp_unix}:0:1");
            composite_keys.push(key);
        }

        // Verify all lookups return correct IDs
        for (i, key) in composite_keys.iter().enumerate() {
            let found_id = store.get_id_by_composite_key(key);
            assert_eq!(found_id, Some(inserted_ids[i]),
                "Lookup for key {} should return ID {}, got {:?}",
                key, inserted_ids[i], found_id);
        }
    }

    #[test]
    fn test_hash_collision_detection() {
        // Verify we can detect if hashes ever collide (should never happen)
        let mut hashes = std::collections::HashSet::new();

        // Generate 1000 different composite keys
        let base_ts = ts("2026-01-31 10:00:00");
        for i in 0..1000 {
            let device = format!("DEVICE_{i}");
            let user = format!("USER_{i}");
            let timestamp_unix = base_ts + i;

            let hash = MemoryLog::compute_composite_hash(&device, &user, timestamp_unix, Some(0), Some(1));

            assert!(hashes.insert(hash),
                "Hash collision detected for device={device}, user={user}, ts={timestamp_unix}");
        }

        assert_eq!(hashes.len(), 1000, "All hashes should be unique");
    }

    #[test]
    fn test_load_from_disk_uses_hash_index() {
        // Verify load_from_disk correctly populates hash-based index
        let store = InMemoryStore::new(1000);

        let logs = vec![
            RawAttendanceLog {
                id: 1,
                device_id: "DEV1".to_string(),
                user_id: "U001".to_string(),
                timestamp: ts("2026-01-31 10:00:00"),
                status: Some(1),
                punch: Some(0),
                is_synced: 0,
                created_at: Some(chrono::Utc::now().naive_utc()),
                updated_at: None,
                retry_count: 0,
            },
            RawAttendanceLog {
                id: 2,
                device_id: "DEV2".to_string(),
                user_id: "U002".to_string(),
                timestamp: ts("2026-01-31 11:00:00"),
                status: Some(1),
                punch: Some(0),
                is_synced: 2,
                created_at: Some(chrono::Utc::now().naive_utc()),
                updated_at: None,
                retry_count: 0,
            },
        ];

        store.load_from_disk(logs);

        // Verify lookups work for loaded logs
        let key1 = format!("DEV1:U001:{}:0:1",
            NaiveDateTime::parse_from_str("2026-01-31 10:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap().and_utc().timestamp());
        let key2 = format!("DEV2:U002:{}:0:1",
            NaiveDateTime::parse_from_str("2026-01-31 11:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap().and_utc().timestamp());

        assert_eq!(store.get_id_by_composite_key(&key1), Some(1));
        assert_eq!(store.get_id_by_composite_key(&key2), Some(2));
    }

    #[test]
    fn test_mark_synced_removes_from_hash_index() {
        // Verify mark_synced immediately removes log and composite index entry
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-31 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        // Construct key
        let timestamp_unix = ts("2026-01-29 10:00:00");
        let key = format!("DEV1:U001:{timestamp_unix}:0:1");

        // Before sync, lookup works
        assert_eq!(store.get_id_by_composite_key(&key), Some(id));

        // Mark synced - should immediately remove
        store.mark_in_flight(&[id]);
        store.mark_persisted(&[id]); // Persist before sync
        store.mark_synced(&[id]);

        // After sync, lookup fails (log removed)
        assert_eq!(store.get_id_by_composite_key(&key), None);
        assert_eq!(store.count_total(), 0);
    }

    #[test]
    fn test_mark_synced_immediate_removal() {
        // Verify synced logs are immediately removed, not waiting for cleanup
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-31 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        assert_eq!(store.count_total(), 1);

        // Mark synced - should remove immediately
        store.mark_in_flight(&[id]);
        store.mark_persisted(&[id]); // Persist before sync
        store.mark_synced(&[id]);

        // Log is gone
        assert_eq!(store.count_total(), 0, "Synced log removed immediately");

        // cleanup_synced has nothing to do
        let removed = store.cleanup_synced(0);
        assert_eq!(removed, 0, "No synced logs to clean");
    }

    #[test]
    fn test_mark_synced_preserves_unsynced_and_inflight() {
        // Verify only synced logs are removed, others remain
        let store = InMemoryStore::new(1000);

        let data1 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-31 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let data2 = AttendanceData {
            user_id: "U002".to_string(),
            timestamp: "2026-01-31 10:01:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let data3 = AttendanceData {
            user_id: "U003".to_string(),
            timestamp: "2026-01-31 10:02:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        let id1 = store.insert("DEV1".to_string(), data1, ts("2026-01-29 10:00:00")).unwrap();
        let id2 = store.insert("DEV1".to_string(), data2, ts("2026-01-29 10:00:00")).unwrap();
        let _id3 = store.insert("DEV1".to_string(), data3, ts("2026-01-29 10:00:00")).unwrap();

        // Mark id1 synced (removed), id2 in-flight (kept), id3 stays unsynced
        store.mark_in_flight(&[id1]);
        store.mark_persisted(&[id1]); // Persist before sync
        store.mark_synced(&[id1]);
        store.mark_in_flight(&[id2]);

        // Only synced log removed
        assert_eq!(store.count_total(), 2, "Unsynced and in-flight remain");
    }

    #[test]
    fn test_capacity_recovery_after_sync() {
        // Test that capacity is freed when logs are synced
        let store = InMemoryStore::new(5);

        // Fill to capacity
        let mut ids = Vec::new();
        for i in 0..5 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: format!("2026-01-31 10:{i:02}:00"),
                status: Some(1),
                punch: Some(0),
            };
            let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
            ids.push(id);
        }

        assert_eq!(store.count_total(), 5);

        // Should fail at capacity
        let extra_data = AttendanceData {
            user_id: "U999".to_string(),
            timestamp: "2026-01-31 11:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        assert!(store.insert("DEV1".to_string(), extra_data.clone(), ts("2026-01-29 10:00:00")).is_err());

        // Sync all logs - they're immediately removed
        store.mark_in_flight(&ids);
        store.mark_persisted(&ids); // Persist before sync
        store.mark_synced(&ids);

        assert_eq!(store.count_total(), 0, "All synced logs removed");

        // Now insert works
        assert!(store.insert("DEV1".to_string(), extra_data, ts("2026-01-29 10:00:00")).is_ok());
    }

    #[test]
    fn test_signature_preserved_after_sync() {
        // Critical: verify signatures survive mark_synced removal
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-31 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        // Insert and sync
        let id = store.insert("DEV1".to_string(), data.clone(), ts("2026-01-29 10:00:00")).unwrap();
        store.mark_in_flight(&[id]);
        store.mark_persisted(&[id]); // Persist before sync
        store.mark_synced(&[id]);

        // Log removed but signature preserved
        assert_eq!(store.count_total(), 0);
        assert_eq!(store.count_signatures(), 1, "Signature preserved for dedup");

        // Duplicate should be rejected
        let result = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00"));
        assert!(result.is_err(), "Duplicate rejected even after sync removal");
    }

    #[test]
    fn test_capacity_percentage_calculation() {
        // Test for monitoring logic
        let store = InMemoryStore::new(100);

        // Fill to 80%
        for i in 0..80 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: format!("2026-01-31 {:02}:{:02}:00", i / 60, i % 60),
                status: Some(1),
                punch: Some(0),
            };
            store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        }

        let total = store.count_total();
        let capacity = 100;
        let percent = (total * 100) / capacity;

        assert_eq!(total, 80);
        assert_eq!(percent, 80, "Should be at 80% capacity");
    }

    #[test]
    fn test_batch_sync_performance() {
        // Test mark_synced handles large batches efficiently
        let store = InMemoryStore::new(10000);

        // Insert 1000 logs
        for i in 0..1000 {
            let data = AttendanceData {
                user_id: format!("U{i:04}"),
                timestamp: format!("2026-01-31 {:02}:{:02}:{:02}",
                    (i / 3600) % 24, (i / 60) % 60, i % 60),
                status: Some(1),
                punch: Some(0),
            };
            store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        }

        assert_eq!(store.count_total(), 1000);

        // Mark all synced (immediate removal)
        let logs = store.get_unsynced(1000);
        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();

        store.mark_in_flight(&ids);
        store.mark_persisted(&ids); // Persist before sync

        let start = std::time::Instant::now();
        store.mark_synced(&ids);
        let elapsed = start.elapsed();

        // All logs removed immediately by mark_synced
        assert_eq!(store.count_total(), 0);
        assert!(elapsed.as_millis() < 100, "Sync removal should be fast (<100ms for 1000 logs)");

        // Signatures preserved
        assert_eq!(store.count_signatures(), 1000);
    }

    #[test]
    fn test_memory_stats_during_sync_cycle() {
        // Simulate production sync cycle behavior
        let store = InMemoryStore::new(1000);

        // Phase 1: Initial inserts
        for i in 0..100 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: format!("2026-01-31 10:{:02}:00", i % 60),
                status: Some(1),
                punch: Some(0),
            };
            store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
        }

        let stats1 = (store.count_total(), store.count_unsynced(), store.count_signatures());
        assert_eq!(stats1, (100, 100, 100), "All should be unsynced initially");

        // Phase 2: Sync half (logs immediately removed)
        let logs = store.get_unsynced(50);
        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        store.mark_in_flight(&ids);
        store.mark_persisted(&ids); // Persist before sync
        store.mark_synced(&ids);

        let stats2 = (store.count_total(), store.count_unsynced(), store.count_signatures());
        assert_eq!(stats2, (50, 50, 100), "50 removed by sync, 50 unsynced remain");

        // Phase 3: cleanup_synced is now no-op (nothing to clean)
        let removed = store.cleanup_synced(0);
        assert_eq!(removed, 0, "No synced logs to clean (removed immediately)");

        let stats3 = (store.count_total(), store.count_unsynced(), store.count_signatures());
        assert_eq!(stats3, (50, 50, 100), "50 remaining, signatures preserved");
    }

    #[test]
    fn test_mark_synced_only_removes_persisted_logs() {
        // Critical safety test: verify logs are only removed after disk persistence
        let store = InMemoryStore::new(1000);

        // Insert 3 logs
        let mut ids = Vec::new();
        for i in 0..3 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: format!("2026-01-31 10:{i:02}:00"),
                status: Some(1),
                punch: Some(0),
            };
            let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
            ids.push(id);
        }

        // Verify all logs are in memory
        assert_eq!(store.count_total(), 3);

        // Mark first 2 as in-flight (simulating send to server)
        store.mark_in_flight(&[ids[0], ids[1]]);

        // Simulate disk checkpoint for only the first log
        store.mark_persisted(&[ids[0]]);

        // Verify persistence status
        assert!(store.logs.get(&ids[0]).unwrap().persisted, "Log 0 should be persisted");
        assert!(!store.logs.get(&ids[1]).unwrap().persisted, "Log 1 should NOT be persisted");
        assert!(!store.logs.get(&ids[2]).unwrap().persisted, "Log 2 should NOT be persisted");

        // Try to mark both in-flight logs as synced
        // Only the persisted log should be removed from memory
        store.mark_synced(&[ids[0], ids[1]]);

        // Critical assertion: only persisted log removed
        assert!(store.logs.get(&ids[0]).is_none(), "Persisted log should be removed");
        assert!(store.logs.get(&ids[1]).is_some(), "Non-persisted log should remain in memory");
        assert!(store.logs.get(&ids[2]).is_some(), "Unsynced log should remain in memory");

        // Verify counts
        assert_eq!(store.count_total(), 2, "Only 1 log removed (the persisted one)");

        // Now persist the second log and mark it synced
        store.mark_persisted(&[ids[1]]);
        store.mark_synced(&[ids[1]]);

        // Now it should be removed
        assert!(store.logs.get(&ids[1]).is_none(), "Now persisted, should be removed");
        assert_eq!(store.count_total(), 1, "2 logs removed total");

        // Signatures should still exist for all 3
        assert_eq!(store.count_signatures(), 3, "All signatures preserved for dedup");
    }

    #[test]
    fn test_mark_synced_logs_warning_for_unpersisted() {
        // Test that attempting to sync unpersisted logs is handled safely
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-31 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();

        // Mark as in-flight
        store.mark_in_flight(&[id]);

        // Try to mark as synced WITHOUT persisting first
        // This should NOT remove the log (safety check)
        store.mark_synced(&[id]);

        // Log should still be in memory (not removed)
        assert!(store.logs.get(&id).is_some(), "Unpersisted log should NOT be removed");
        assert_eq!(store.count_total(), 1, "Log count should remain 1");

        // Now persist and sync - should work
        store.mark_persisted(&[id]);
        store.mark_synced(&[id]);

        // Now it should be removed
        assert!(store.logs.get(&id).is_none(), "Persisted log should be removed");
        assert_eq!(store.count_total(), 0);
    }

    #[test]
    fn test_memory_cleanup_requires_both_synced_and_persisted() {
        // Comprehensive test: Memory cleanup only when BOTH synced AND persisted
        let store = InMemoryStore::new(1000);

        // Insert 4 test logs
        let mut ids = Vec::new();
        for i in 0..4 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: format!("2026-01-31 10:{i:02}:00"),
                status: Some(1),
                punch: Some(0),
            };
            let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
            ids.push(id);
        }

        assert_eq!(store.count_total(), 4);

        // Scenario 1: NOT synced + NOT persisted (baseline)
        // Log 0: Stays unsynced, not persisted
        // Expected: Kept in memory ✓

        // Scenario 2: NOT synced + persisted
        // Log 1: Persisted but not synced yet
        store.mark_persisted(&[ids[1]]);
        // Expected: Kept in memory (waiting for sync)

        // Scenario 3: Synced + NOT persisted (should warn and keep)
        // Log 2: Mark in-flight and try to sync WITHOUT persisting
        store.mark_in_flight(&[ids[2]]);
        store.mark_synced(&[ids[2]]);
        // Expected: Kept in memory (not persisted, logged warning)

        // Scenario 4: Synced + persisted (correct flow)
        // Log 3: Persisted AND synced
        store.mark_in_flight(&[ids[3]]);
        store.mark_persisted(&[ids[3]]);
        store.mark_synced(&[ids[3]]);
        // Expected: REMOVED from memory ✓

        // Verify results
        assert!(store.logs.get(&ids[0]).is_some(), "Log 0: NOT synced + NOT persisted = KEPT");
        assert!(store.logs.get(&ids[1]).is_some(), "Log 1: NOT synced + persisted = KEPT");
        assert!(store.logs.get(&ids[2]).is_some(), "Log 2: synced + NOT persisted = KEPT");
        assert!(store.logs.get(&ids[3]).is_none(), "Log 3: synced + persisted = REMOVED");

        // Verify counts
        assert_eq!(store.count_total(), 3, "Only 1 log removed (synced + persisted)");

        // Verify persistence flags
        assert!(!store.logs.get(&ids[0]).unwrap().persisted, "Log 0 not persisted");
        assert!(store.logs.get(&ids[1]).unwrap().persisted, "Log 1 persisted but not synced");
        assert!(!store.logs.get(&ids[2]).unwrap().persisted, "Log 2 synced but not persisted");

        // All signatures preserved
        assert_eq!(store.count_signatures(), 4, "All signatures preserved for dedup");
    }

    #[test]
    fn test_persisted_logs_not_synced_remain_in_memory() {
        // Test that persisted logs awaiting sync are NOT removed
        let store = InMemoryStore::new(1000);

        // Insert 3 logs
        let mut ids = Vec::new();
        for i in 0..3 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: format!("2026-01-31 10:{i:02}:00"),
                status: Some(1),
                punch: Some(0),
            };
            let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
            ids.push(id);
        }

        // Persist all logs (simulating successful disk checkpoint)
        store.mark_persisted(&ids);

        // Verify all are persisted
        for &id in &ids {
            assert!(store.logs.get(&id).unwrap().persisted, "Log should be persisted");
        }

        // Mark only first log as in-flight and synced
        store.mark_in_flight(&[ids[0]]);
        store.mark_synced(&[ids[0]]);

        // Only the synced log should be removed
        assert!(store.logs.get(&ids[0]).is_none(), "Synced log removed");
        assert!(store.logs.get(&ids[1]).is_some(), "Persisted but not synced = kept");
        assert!(store.logs.get(&ids[2]).is_some(), "Persisted but not synced = kept");

        assert_eq!(store.count_total(), 2, "Only synced log removed");

        // The remaining persisted logs are still available for sync
        let unsynced = store.get_unsynced(10);
        assert_eq!(unsynced.len(), 2, "2 persisted logs awaiting sync");
    }

    #[test]
    fn test_checkpoint_then_sync_flow() {
        // Test production flow: checkpoint to disk -> send to server -> receive ACK -> cleanup
        let store = InMemoryStore::new(1000);

        // Phase 1: Insert logs (memory only, not persisted)
        let mut ids = Vec::new();
        for i in 0..5 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: format!("2026-01-31 10:{i:02}:00"),
                status: Some(1),
                punch: Some(0),
            };
            let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
            ids.push(id);
        }

        assert_eq!(store.count_total(), 5);

        // Verify all are unpersisted
        for &id in &ids {
            assert!(!store.logs.get(&id).unwrap().persisted, "Should be unpersisted initially");
        }

        // Phase 2: Checkpoint to disk (simulate batch checkpoint)
        store.mark_persisted(&ids);

        // Verify all are now persisted
        for &id in &ids {
            assert!(store.logs.get(&id).unwrap().persisted, "Should be persisted after checkpoint");
        }

        // All logs still in memory (not synced yet)
        assert_eq!(store.count_total(), 5, "All logs remain after checkpoint");

        // Phase 3: Send to server (mark in-flight)
        store.mark_in_flight(&ids);

        // All logs still in memory (awaiting ACK)
        assert_eq!(store.count_total(), 5, "All logs remain while in-flight");

        // Phase 4: Receive ACK from server (mark synced)
        store.mark_synced(&ids);

        // NOW all logs are removed (synced + persisted)
        assert_eq!(store.count_total(), 0, "All logs removed after sync");

        // Verify all removed
        for &id in &ids {
            assert!(store.logs.get(&id).is_none(), "Log should be removed");
        }

        // Signatures preserved
        assert_eq!(store.count_signatures(), 5, "All signatures preserved");
    }

    #[test]
    fn test_partial_checkpoint_partial_sync() {
        // Test mixed scenario: some persisted, some synced
        let store = InMemoryStore::new(1000);

        // Insert 6 logs
        let mut ids = Vec::new();
        for i in 0..6 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: format!("2026-01-31 10:{i:02}:00"),
                status: Some(1),
                punch: Some(0),
            };
            let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
            ids.push(id);
        }

        // Checkpoint first 4 logs
        store.mark_persisted(&ids[0..4]);

        // Mark all 6 as in-flight
        store.mark_in_flight(&ids);

        // Try to sync all 6 logs
        store.mark_synced(&ids);

        // Only the 4 persisted logs should be removed
        assert!(store.logs.get(&ids[0]).is_none(), "Persisted + synced = removed");
        assert!(store.logs.get(&ids[1]).is_none(), "Persisted + synced = removed");
        assert!(store.logs.get(&ids[2]).is_none(), "Persisted + synced = removed");
        assert!(store.logs.get(&ids[3]).is_none(), "Persisted + synced = removed");
        assert!(store.logs.get(&ids[4]).is_some(), "NOT persisted + synced = kept");
        assert!(store.logs.get(&ids[5]).is_some(), "NOT persisted + synced = kept");

        assert_eq!(store.count_total(), 2, "4 removed, 2 kept (not persisted)");

        // The 2 unpersisted logs are still in inflight state
        let unpersisted = store.get_unpersisted();
        assert_eq!(unpersisted.len(), 2, "2 logs awaiting disk checkpoint");
    }

    #[test]
    fn test_disk_checkpoint_failure_recovery() {
        // Test scenario: checkpoint fails, logs retry on next checkpoint
        let store = InMemoryStore::new(1000);

        // Insert logs
        let mut ids = Vec::new();
        for i in 0..3 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: format!("2026-01-31 10:{i:02}:00"),
                status: Some(1),
                punch: Some(0),
            };
            let id = store.insert("DEV1".to_string(), data, ts("2026-01-29 10:00:00")).unwrap();
            ids.push(id);
        }

        // Send to server before checkpoint (edge case)
        store.mark_in_flight(&ids);

        // Server sends ACK but logs not persisted yet
        store.mark_synced(&ids);

        // All logs should be KEPT (not persisted)
        assert_eq!(store.count_total(), 3, "All logs kept (not persisted)");

        // Logs are still marked as in-flight, waiting for checkpoint
        for &id in &ids {
            assert!(store.logs.get(&id).is_some(), "Log should remain in memory");
            assert!(!store.logs.get(&id).unwrap().persisted, "Should not be persisted");
        }

        // Now checkpoint succeeds
        store.mark_persisted(&ids);

        // Try to sync again (re-send ACK after checkpoint)
        store.mark_synced(&ids);

        // NOW all logs are removed (persisted + synced)
        assert_eq!(store.count_total(), 0, "All logs removed after checkpoint + sync");
    }

    // ===== Signature Eviction (TTL) Tests =====

    #[test]
    fn test_evict_old_signatures_removes_expired() {
        let store = InMemoryStore::new(1000);

        // Insert a signature with old timestamp (200 days ago)
        let old_ts = chrono::Utc::now().timestamp() - (200 * 86400);
        let data_old = AttendanceData {
            user_id: "OLD_USER".to_string(),
            timestamp: "2025-01-01 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        store.insert("D1".to_string(), data_old, old_ts).unwrap();

        // Insert a signature with recent timestamp (1 day ago)
        let recent_ts = chrono::Utc::now().timestamp() - 86400;
        let data_recent = AttendanceData {
            user_id: "RECENT_USER".to_string(),
            timestamp: "2026-02-13 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        store.insert("D1".to_string(), data_recent, recent_ts).unwrap();

        assert_eq!(store.count_signatures(), 2);

        // Evict signatures older than 90 days
        let ttl_secs = 90 * 86400;
        let (evicted, before, after) = store.evict_old_signatures(ttl_secs);

        assert_eq!(evicted, 1, "Should evict 1 old signature");
        assert_eq!(before, 2);
        assert_eq!(after, 1);
        assert_eq!(store.count_signatures(), 1);
    }

    #[test]
    fn test_evict_old_signatures_keeps_all_when_within_ttl() {
        let store = InMemoryStore::new(1000);

        // Insert signatures with recent timestamps
        let ts1 = chrono::Utc::now().timestamp() - 3600; // 1 hour ago
        let ts2 = chrono::Utc::now().timestamp() - 7200; // 2 hours ago

        let data1 = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-02-14 09:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let data2 = AttendanceData {
            user_id: "U2".to_string(),
            timestamp: "2026-02-14 08:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        store.insert("D1".to_string(), data1, ts1).unwrap();
        store.insert("D1".to_string(), data2, ts2).unwrap();

        // Evict with 90-day TTL — nothing should be evicted
        let (evicted, _, _) = store.evict_old_signatures(90 * 86400);
        assert_eq!(evicted, 0);
        assert_eq!(store.count_signatures(), 2);
    }

    #[test]
    fn test_evict_old_signatures_empty_store() {
        let store = InMemoryStore::new(1000);

        let (evicted, before, after) = store.evict_old_signatures(90 * 86400);

        assert_eq!(evicted, 0);
        assert_eq!(before, 0);
        assert_eq!(after, 0);
    }

    #[test]
    fn test_evict_old_signatures_evicts_all_when_all_expired() {
        let store = InMemoryStore::new(1000);

        // Insert signatures all older than 100 days
        for i in 0..5 {
            let old_ts = chrono::Utc::now().timestamp() - (200 * 86400) - i;
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2025-01-01 10:00:00".to_string(),
                status: Some(1),
                punch: Some(0),
            };
            store.insert("D1".to_string(), data, old_ts).unwrap();
        }

        assert_eq!(store.count_signatures(), 5);

        let (evicted, _, after) = store.evict_old_signatures(90 * 86400);

        assert_eq!(evicted, 5);
        assert_eq!(after, 0);
    }

    #[test]
    fn test_evict_allows_reinsertion_of_evicted_signature() {
        let store = InMemoryStore::new(1000);

        // Insert with old timestamp
        let old_ts = chrono::Utc::now().timestamp() - (200 * 86400);
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2025-01-01 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        store.insert("D1".to_string(), data.clone(), old_ts).unwrap();

        // Verify duplicate rejected before eviction
        let dup_result = store.insert("D1".to_string(), data.clone(), old_ts);
        assert!(matches!(dup_result, Err(StoreError::Duplicate)));

        // Evict
        store.evict_old_signatures(90 * 86400);
        assert_eq!(store.count_signatures(), 0);

        // Re-insertion should succeed after eviction
        let result = store.insert("D1".to_string(), data, old_ts);
        assert!(result.is_ok(), "Should allow reinsertion after eviction");
        assert_eq!(store.count_signatures(), 1);
    }

    #[test]
    fn test_evict_does_not_affect_active_logs() {
        let store = InMemoryStore::new(1000);

        // Insert a log with old timestamp (will have signature evicted)
        let old_ts = chrono::Utc::now().timestamp() - (200 * 86400);
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2025-01-01 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let id = store.insert("D1".to_string(), data, old_ts).unwrap();

        // Log is in memory and unsynced
        assert_eq!(store.count_total(), 1);
        assert_eq!(store.count_unsynced(), 1);

        // Evict old signatures
        store.evict_old_signatures(90 * 86400);

        // Signature evicted but log still present
        assert_eq!(store.count_signatures(), 0);
        assert_eq!(store.count_total(), 1, "Log must not be removed by eviction");
        assert_eq!(store.count_unsynced(), 1);

        // Log can still be fetched and synced
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, id);
    }

    #[test]
    fn test_dashmap_signature_stores_timestamp() {
        let store = InMemoryStore::new(1000);

        let ts_val = chrono::Utc::now().timestamp() - 3600;
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-02-14 09:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        store.insert("D1".to_string(), data, ts_val).unwrap();

        // Verify the timestamp is stored in the DashMap value
        let sig = InMemoryStore::compute_signature("D1", "U1", ts_val, Some(0), Some(1));
        let stored_ts = store.signatures.get(&sig).map(|entry| *entry.value());
        assert_eq!(stored_ts, Some(ts_val), "Timestamp should be stored as signature value");
    }

    #[test]
    fn test_load_signatures_stores_timestamps() {
        let store = InMemoryStore::new(1000);
        let ts_val = 1700000000i64;

        let rows = vec![
            SignatureRow {
                device_id: "D1".to_string(),
                user_id: "U1".to_string(),
                timestamp: ts_val,
                punch: Some(0),
                status: Some(1),
            },
        ];

        store.load_signatures(rows);
        assert_eq!(store.count_signatures(), 1);

        // Verify the timestamp was stored
        let sig = InMemoryStore::compute_signature("D1", "U1", ts_val, Some(0), Some(1));
        let stored_ts = store.signatures.get(&sig).map(|entry| *entry.value());
        assert_eq!(stored_ts, Some(ts_val));
    }

    #[test]
    fn test_batch_insert_ack_correlation() {
        let store = InMemoryStore::new(1000);

        let timestamp_unix = ts("2026-01-31 10:00:00");
        let batch = vec![
            (Arc::new("DEV1".to_string()), AttendanceData {
                user_id: "U001".to_string(),
                timestamp: "2026-01-31 10:00:00".to_string(),
                status: Some(1),
                punch: Some(0), // check-in
            }, timestamp_unix),
            (Arc::new("DEV1".to_string()), AttendanceData {
                user_id: "U001".to_string(),
                timestamp: "2026-01-31 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1), // check-out at the SAME second
            }, timestamp_unix),
        ];

        // 1. Insert batch
        let inserted = store.insert_batch(batch).unwrap();
        assert_eq!(inserted, 2, "Both logs should be inserted");

        // 2. Verify both are in composite_index with unique keys
        let key1 = format!("DEV1:U001:{timestamp_unix}:0:1");
        let key2 = format!("DEV1:U001:{timestamp_unix}:1:1");

        let id1 = store.get_id_by_composite_key(&key1);
        let id2 = store.get_id_by_composite_key(&key2);

        assert!(id1.is_some(), "First log should be found by composite key");
        assert!(id2.is_some(), "Second log should be found by composite key");
        assert_ne!(id1, id2, "IDs must be unique");

        // 3. Verify sync works for batch-inserted logs
        store.mark_persisted(&[id1.unwrap(), id2.unwrap()]);
        
        assert!(store.mark_synced_by_key(&key1), "Should successfully mark first log as synced");
        assert!(store.mark_synced_by_key(&key2), "Should successfully mark second log as synced");

        // 4. Verify memory cleared
        assert_eq!(store.count_total(), 0, "Both logs should be removed from memory after sync");
    }

    #[test]
    fn test_composite_key_parsing_edge_cases() {
        let store = InMemoryStore::new(100);
        
        // Test with UUID device_id
        let uuid = "786b2a80-c6a0-4681-a27d-dc1fc876bcf8";
        let key = format!("{uuid}:USER1:1738314000:0:1");
        
        let timestamp_unix = 1738314000;
        let data = AttendanceData {
            user_id: "USER1".to_string(),
            timestamp: "ignored".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        
        let id = store.insert(uuid.to_string(), data, timestamp_unix).unwrap();
        assert_eq!(store.get_id_by_composite_key(&key), Some(id));

        // Test with device_id containing colons (non-UUID)
        let device_id = "MAC:AA:BB:CC:DD:EE:FF";
        let key2 = format!("{device_id}:USER2:1738314000:-1:-1");
        let data2 = AttendanceData {
            user_id: "USER2".to_string(),
            timestamp: "ignored".to_string(),
            status: None,
            punch: None,
        };
        
        let id2 = store.insert(device_id.to_string(), data2, timestamp_unix).unwrap();
        assert_eq!(store.get_id_by_composite_key(&key2), Some(id2), "Should handle device_id with colons");
    }
}
