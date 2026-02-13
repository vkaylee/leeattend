use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use agent::database::Database;
use agent::models::AttendanceData;

#[tokio::test]
async fn test_batch_worker_flushes_on_timeout() {
    let db = Database::new("sqlite::memory:").await.unwrap();
    let (log_tx, mut log_rx) = mpsc::channel::<(String, AttendanceData)>(100);
    let writer_db = db.clone();

    tokio::spawn(async move {
        let mut buffer = Vec::with_capacity(10);
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                msg = log_rx.recv() => {
                    match msg {
                        Some(item) => {
                            buffer.push(item);
                            if buffer.len() >= 10 {
                                let to_save = std::mem::replace(&mut buffer, Vec::with_capacity(10));
                                writer_db.save_raw_attendance_batch(to_save).await.unwrap();
                            }
                        }
                        None => break,
                    }
                }
                _ = interval.tick() => {
                    if !buffer.is_empty() {
                        let to_save = std::mem::replace(&mut buffer, Vec::with_capacity(10));
                        writer_db.save_raw_attendance_batch(to_save).await.unwrap();
                    }
                }
            }
        }
    });

    for i in 0..3 {
        log_tx.send(("D1".to_string(), AttendanceData {
            user_id: format!("U{i}"),
            timestamp: format!("2026-01-01 10:00:0{i}"),
            status: Some(1),
            punch: Some(1),
        })).await.unwrap();
    }

    sleep(Duration::from_millis(300)).await;

    let logs = db.get_unsynced_logs(100).await.unwrap();
    assert_eq!(logs.len(), 3, "Logs should be flushed even if batch is not full");
}
