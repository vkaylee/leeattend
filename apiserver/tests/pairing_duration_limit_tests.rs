use apiserver::services::pairing_algo::{
    AnchorFirstStrategy, DpPatternStrategy, FsmStrategy, LogEvent, PairingStrategy, SequentialStrategy,
};
use uuid::Uuid;

#[test]
fn test_anchor_first_16h_limit() {
    let strategy = AnchorFirstStrategy;
    
    // Scenario:
    // Log 1: 08:00 (480 min)
    // Log 2: 23:00 (1380 min) -> Diff 900 min (15h) - SHOULD PAIR
    let logs_pair = vec![
        LogEvent { id: Uuid::new_v4(), minutes: 480, punch_type: None },
        LogEvent { id: Uuid::new_v4(), minutes: 1380, punch_type: None },
    ];
    let sessions = strategy.pair(&logs_pair, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].end_minutes, Some(1380));

    // Scenario:
    // Log 1: 08:00 (480 min)
    // Log 2: 07:00 Next Day (1860 min) -> Diff 1380 min (23h) - SHOULD NOT PAIR
    let logs_steal = vec![
        LogEvent { id: Uuid::new_v4(), minutes: 480, punch_type: None },
        LogEvent { id: Uuid::new_v4(), minutes: 1860, punch_type: None },
    ];
    let sessions = strategy.pair(&logs_steal, None);
    assert_eq!(sessions.len(), 1);
    assert!(sessions[0].end_minutes.is_none(), "Should not steal log from next day (>16h)");
    assert_eq!(sessions[0].note, Some("Missing Punch Out".to_string()));
}

#[test]
fn test_sequential_16h_limit() {
    let strategy = SequentialStrategy;
    
    // Log 1: 08:00 (480)
    // Log 2: 07:00 Next Day (1860) -> 23h gap
    let logs = vec![
        LogEvent { id: Uuid::new_v4(), minutes: 480, punch_type: None },
        LogEvent { id: Uuid::new_v4(), minutes: 1860, punch_type: None },
    ];
    
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 2, "Should create 2 separate missing-out sessions due to 23h gap");
    assert!(sessions[0].end_minutes.is_none());
    assert!(sessions[1].end_minutes.is_none());
}

#[test]
fn test_fsm_16h_limit() {
    let strategy = FsmStrategy;
    
    // FSM Toggle mode (punch_type = None) should also respect the 16h gap
    let logs = vec![
        LogEvent { id: Uuid::new_v4(), minutes: 480, punch_type: None },
        LogEvent { id: Uuid::new_v4(), minutes: 1860, punch_type: None },
    ];
    
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 2, "FSM toggle should not pair logs with 23h gap");
}

#[test]
fn test_dp_pattern_16h_limit() {
    // DP Pattern is shift-aware. 
    // If we have a shift 08:00 - 17:00, and logs are 08:00 and 07:00 (Next Day).
    // It should definitely not pair them.
    let strategy = DpPatternStrategy;
    let shift = apiserver::services::pairing_algo::PairingShiftConfig {
        start_minutes: 480,
        end_minutes: 1020,
        break_start_minutes: None,
        break_end_minutes: None,
    };
    
    let logs = vec![
        LogEvent { id: Uuid::new_v4(), minutes: 480, punch_type: None },
        LogEvent { id: Uuid::new_v4(), minutes: 1860, punch_type: None },
    ];
    
    let sessions = strategy.pair(&logs, Some(&shift));
    // DP should see 08:00 as a good match for start, but 1860 is way outside shift.
    assert_eq!(sessions.len(), 1);
    assert!(sessions[0].end_minutes.is_none());
}
