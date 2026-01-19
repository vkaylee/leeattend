use apiserver::services::pairing_algo::{
    LogEvent, PairingStrategy, SequentialStrategy,
};

#[test]
fn test_sequential_gap_limit() {
    let strategy = SequentialStrategy;

    // Scenario:
    // Log 1: 07:11 (431 mins)
    // Log 2: 18:06 (1086 mins) -> Diff 655 mins (< 960), should pair
    // Log 3: 18:19 (1099 mins)
    // Log 4: Next Day 07:18 (438 + 1440 = 1878 mins) -> Diff 779 mins (< 960), should pair if standard
    // BUT what if we had a missed punch?
    
    // Let's test the specific issue case where we want to enforce reasonable shift lengths.
    // Case A: 8 hours gap -> Pair
    let logs_ok = vec![
        LogEvent { id: uuid::Uuid::new_v4(), minutes: 480, punch_type: None }, // 08:00
        LogEvent { id: uuid::Uuid::new_v4(), minutes: 960, punch_type: None }, // 16:00 (Diff 480)
    ];
    let sessions_ok = strategy.pair(&logs_ok, None);
    assert_eq!(sessions_ok.len(), 1);
    assert_eq!(sessions_ok[0].end_minutes, Some(960));

    // Case B: 17 hours gap -> Split
    let logs_gap = vec![
        LogEvent { id: uuid::Uuid::new_v4(), minutes: 480, punch_type: None }, // 08:00
        LogEvent { id: uuid::Uuid::new_v4(), minutes: 1560, punch_type: None }, // 02:00 next day (Diff 1080 > 960)
    ];
    let sessions_gap = strategy.pair(&logs_gap, None);
    assert_eq!(sessions_gap.len(), 2);
    
    // First session: 08:00 -> Missing Out
    assert_eq!(sessions_gap[0].start_minutes, 480);
    assert_eq!(sessions_gap[0].end_minutes, None);
    assert_eq!(sessions_gap[0].note, Some("Missing Punch Out".to_string()));

    // Second session: 02:00 -> Missing Out (since it's the last one and unmatched)
    assert_eq!(sessions_gap[1].start_minutes, 1560);
    assert_eq!(sessions_gap[1].end_minutes, None);
}
