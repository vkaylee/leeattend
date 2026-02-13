use crate::models::AttendanceData;
use tokio::sync::mpsc;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct WorkerConfig {
    pub id: String,
    pub name: String,
    pub brand: String,
    pub ip: String,
    pub port: u32,
    pub sidecar_script: String,
}

/// Process fetched logs: Deduplicate and send to queue using correct Device ID
pub async fn process_fetched_logs(
    config: &WorkerConfig,
    logs: Vec<AttendanceData>,
    seen_logs: &mut HashSet<String>,
    log_tx: &mpsc::Sender<(String, AttendanceData)>,
) -> (usize, usize) { // Returns (new_count, duplicate_count)
    let mut new_count = 0;
    let mut duplicate_count = 0;

    for attendance in logs {
        // Create unique key: user_id:timestamp
        let key = format!("{}:{}", attendance.user_id, attendance.timestamp);

        if seen_logs.insert(key) {
            // New log - send to queue using stable ID (not Name)
            // This is the CRITICAL fix: ensure config.id is used
            let _ = log_tx.send((config.id.clone(), attendance)).await;
            new_count += 1;
        } else {
            // Duplicate - skip
            duplicate_count += 1;
        }
    }

    (new_count, duplicate_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[tokio::test]
    async fn test_process_logs_uses_correct_id() {
        let (tx, mut rx) = mpsc::channel(10);
        
        let config = WorkerConfig {
            id: "UUID-1234".to_string(),
            name: "OFFICE-PC".to_string(),
            brand: "test".to_string(),
            ip: "1.2.3.4".to_string(),
            port: 80,
            sidecar_script: "script.py".to_string(),
        };

        let mut seen = HashSet::new();
        let logs = vec![
            AttendanceData {
                user_id: "U1".to_string(),
                timestamp: "2023-01-01 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            }
        ];

        // Process
        let (new, dup) = process_fetched_logs(&config, logs, &mut seen, &tx).await;

        // Verify counts
        assert_eq!(new, 1);
        assert_eq!(dup, 0);

        // Verify sent message uses UUID, NOT Name
        let msg = rx.recv().await.expect("Should receive message");
        assert_eq!(msg.0, "UUID-1234", "Must use Device ID (UUID) for storage, not Name");
        assert_ne!(msg.0, "OFFICE-PC");
    }

    #[tokio::test]
    async fn test_deduplication_logic() {
        let (tx, _rx) = mpsc::channel(10);
        let config = WorkerConfig {
            id: "id".into(), name: "name".into(), brand: "b".into(), ip: "i".into(), port: 0, sidecar_script: "s".into() 
        };
        let mut seen = HashSet::new();

        let log1 = AttendanceData { user_id: "U1".into(), timestamp: "T1".into(), status: None, punch: None };
        let log2 = AttendanceData { user_id: "U1".into(), timestamp: "T1".into(), status: None, punch: None }; // Duplicate

        let (n1, d1) = process_fetched_logs(&config, vec![log1], &mut seen, &tx).await;
        assert_eq!(n1, 1);
        assert_eq!(d1, 0);

        let (n2, d2) = process_fetched_logs(&config, vec![log2], &mut seen, &tx).await;
        assert_eq!(n2, 0);
        assert_eq!(d2, 1);
    }
}
