use agent::mock_worker::MockWorker;
use agent::worker::Worker;
use agent::time_sync::TimeSync;

#[tokio::test]
async fn test_mock_worker_get_info_contains_logs_capacity() {
    let mut worker = MockWorker::new();
    let info = worker.get_info(&TimeSync::new()).await.unwrap();

    // Verify logs_capacity field exists and is a positive number
    assert!(
        info.get("logs_capacity").is_some(),
        "get_info() must include logs_capacity field"
    );
    assert!(
        info["logs_capacity"].as_i64().unwrap() > 0,
        "logs_capacity must be a positive number"
    );
}

#[tokio::test]
async fn test_mock_worker_get_info_contains_logs_count() {
    let mut worker = MockWorker::new();
    let info = worker.get_info(&TimeSync::new()).await.unwrap();

    assert!(
        info.get("logs_count").is_some(),
        "get_info() must include logs_count field"
    );
    assert!(
        info["logs_count"].as_i64().unwrap() >= 0,
        "logs_count must be non-negative"
    );
}

#[tokio::test]
async fn test_mock_worker_get_info_capacity_greater_than_count() {
    let mut worker = MockWorker::new();
    let info = worker.get_info(&TimeSync::new()).await.unwrap();

    let logs_count = info["logs_count"].as_i64().unwrap();
    let logs_capacity = info["logs_capacity"].as_i64().unwrap();

    assert!(
        logs_capacity > logs_count,
        "logs_capacity ({logs_capacity}) should be greater than logs_count ({logs_count})"
    );
}

#[tokio::test]
async fn test_mock_worker_get_info_all_required_fields() {
    let mut worker = MockWorker::new();
    let info = worker.get_info(&TimeSync::new()).await.unwrap();

    let required_fields = [
        "users_count",
        "logs_count",
        "logs_capacity",
        "device_time",
        "firmware",
        "mac",
        "serial",
        "platform",
        "hw_name",
        "last_sync",
    ];

    for field in required_fields {
        assert!(
            info.get(field).is_some(),
            "get_info() response missing required field: {field}"
        );
    }
}

