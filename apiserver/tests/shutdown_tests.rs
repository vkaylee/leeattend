use apiserver::agent_manager::AgentManager;
use tokio::sync::mpsc;
use axum::extract::ws::Message;

#[tokio::test]
async fn test_agent_manager_shutdown_notification() {
    let manager = AgentManager::new();
    let (tx1, mut rx1) = mpsc::unbounded_channel();
    let (tx2, mut rx2) = mpsc::unbounded_channel();

    manager.add_agent("agent_1".to_string(), tx1).await;
    manager.add_agent("agent_2".to_string(), tx2).await;

    // Trigger shutdown notification
    manager.shutdown().await;

    // Verify agent 1 received shutdown message
    let msg1 = rx1.recv().await.unwrap();
    if let Message::Text(text) = msg1 {
        assert!(text.contains("SERVER_SHUTDOWN"));
    } else {
        panic!("Expected text message for agent 1");
    }

    // Verify agent 2 received shutdown message
    let msg2 = rx2.recv().await.unwrap();
    if let Message::Text(text) = msg2 {
        assert!(text.contains("SERVER_SHUTDOWN"));
    } else {
        panic!("Expected text message for agent 2");
    }
}
