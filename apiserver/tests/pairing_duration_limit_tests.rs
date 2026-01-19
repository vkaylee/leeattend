use apiserver::services::pairing_algo::{
    AnchorFirstStrategy, LogEvent, PairingStrategy,
};
use uuid::Uuid;

#[test]
fn test_anchor_first_12h_limit() {
    let strategy = AnchorFirstStrategy;
    
    // Scenario:
    // Log 1: 08:00 (480 min)
    // Log 2: 19:00 (1140 min) -> Diff 660 min (11h) - SHOULD PAIR
    let logs_pair = vec![
        LogEvent { id: Uuid::new_v4(), minutes: 480, punch_type: None },
        LogEvent { id: Uuid::new_v4(), minutes: 1140, punch_type: None },
    ];
    let sessions = strategy.pair(&logs_pair, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].end_minutes, Some(1140));

    // Scenario:
    // Log 1: 08:00 (480 min)
    // Log 2: 21:00 (1260 min) -> Diff 780 min (13h) - SHOULD NOT PAIR (>12h)
    let logs_steal = vec![
        LogEvent { id: Uuid::new_v4(), minutes: 480, punch_type: None },
        LogEvent { id: Uuid::new_v4(), minutes: 1260, punch_type: None },
    ];
    let sessions = strategy.pair(&logs_steal, None);
    assert_eq!(sessions.len(), 1);
    assert!(sessions[0].end_minutes.is_none(), "Should not pair logs with >12h gap");
}