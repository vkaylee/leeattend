
use apiserver::services::pairing_algo::{self, PairingShiftConfig, PairingStrategy, DpPatternStrategy, AnchorFirstStrategy, SequentialStrategy, FsmStrategy, LogEvent};

fn standard_shift() -> PairingShiftConfig {
    PairingShiftConfig {
        start_minutes: 8 * 60, // 480 (08:00)
        end_minutes: 17 * 60, // 1020 (17:00)
        break_start_minutes: Some(12 * 60), // 720 (12:00)
        break_end_minutes: Some(13 * 60), // 780 (13:00)
    }
}

#[test]
fn test_strategies_perfect_day() {
    let logs = vec![
        LogEvent { minutes: 475, punch_type: None },
        LogEvent { minutes: 725, punch_type: None },
        LogEvent { minutes: 775, punch_type: None },
        LogEvent { minutes: 1025, punch_type: None },
    ];
    let shift = standard_shift();

    // DP Strategy should find 2 sessions
    let dp = DpPatternStrategy;
    let dp_sessions = dp.pair(&logs, Some(&shift));
    assert_eq!(dp_sessions.len(), 2, "DP should find 2 sessions");
    assert_eq!(dp_sessions[0].start_minutes, 475);
    assert_eq!(dp_sessions[0].end_minutes, Some(725));
    assert_eq!(dp_sessions[1].start_minutes, 775);
    assert_eq!(dp_sessions[1].end_minutes, Some(1025));

    // Anchor Strategy should find 1 session
    let anchor = AnchorFirstStrategy;
    let anchor_sessions = anchor.pair(&logs, Some(&shift));
    assert_eq!(anchor_sessions.len(), 1, "Anchor should find 1 session");
    assert_eq!(anchor_sessions[0].start_minutes, 475);
    assert_eq!(anchor_sessions[0].end_minutes, Some(1025));
}

#[test]
fn test_strategies_overtime() {
    let logs = vec![
        LogEvent { minutes: 436, punch_type: None },
        LogEvent { minutes: 1204, punch_type: None },
    ];
    let shift = standard_shift();

    // DP Strategy (with increased penalty 300) should now pair these
    let dp = DpPatternStrategy;
    let dp_sessions = dp.pair(&logs, Some(&shift));
    assert_eq!(dp_sessions.len(), 1, "DP should pair overtime logs");
    assert_eq!(dp_sessions[0].start_minutes, 436);
    assert_eq!(dp_sessions[0].end_minutes, Some(1204));

    // Anchor Strategy should also pair these
    let anchor = AnchorFirstStrategy;
    let anchor_sessions = anchor.pair(&logs, Some(&shift));
    assert_eq!(anchor_sessions.len(), 1);
    assert_eq!(anchor_sessions[0].start_minutes, 436);
    assert_eq!(anchor_sessions[0].end_minutes, Some(1204));
}

#[test]
fn test_strategies_missing_out() {
    let logs = vec![
        LogEvent { minutes: 480, punch_type: None },
    ];
    let shift = standard_shift();

    let dp = DpPatternStrategy;
    let dp_sessions = dp.pair(&logs, Some(&shift));
    assert_eq!(dp_sessions.len(), 1);
    assert!(dp_sessions[0].end_minutes.is_none());
    assert_eq!(dp_sessions[0].note, Some("Missing Punch Out".to_string()));

    let anchor = AnchorFirstStrategy;
    let anchor_sessions = anchor.pair(&logs, Some(&shift));
    assert_eq!(anchor_sessions.len(), 1);
    assert!(anchor_sessions[0].end_minutes.is_none());
    assert_eq!(anchor_sessions[0].note, Some("Missing Punch Out".to_string()));
}

#[test]
fn test_factory_dispatch() {
    let dp = pairing_algo::get_strategy("dp_pattern");
    let anchor = pairing_algo::get_strategy("anchor_first");
    let sequential = pairing_algo::get_strategy("sequential");
    let fsm = pairing_algo::get_strategy("fsm");
    let random = pairing_algo::get_strategy("something_else");

    let logs = vec![
        LogEvent { minutes: 100, punch_type: None },
        LogEvent { minutes: 200, punch_type: None },
        LogEvent { minutes: 300, punch_type: None },
        LogEvent { minutes: 400, punch_type: None },
    ];
    
    // DP needs shift or it fallbacks to anchor behavior (1 session)
    assert_eq!(dp.pair(&logs, None).len(), 1);
    assert_eq!(anchor.pair(&logs, None).len(), 1);
    assert_eq!(sequential.pair(&logs, None).len(), 2);
    assert_eq!(fsm.pair(&logs, None).len(), 2);
    assert_eq!(random.pair(&logs, None).len(), 1);
}

#[test]
fn test_strategies_sequential() {
    let strategy = SequentialStrategy;
    
    // Case 1: 4 logs -> 2 sessions
    let logs = vec![
        LogEvent { minutes: 480, punch_type: None },
        LogEvent { minutes: 720, punch_type: None },
        LogEvent { minutes: 780, punch_type: None },
        LogEvent { minutes: 1020, punch_type: None },
    ];
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].start_minutes, 480);
    assert_eq!(sessions[0].end_minutes, Some(720));
    assert_eq!(sessions[1].start_minutes, 780);
    assert_eq!(sessions[1].end_minutes, Some(1020));

    // Case 2: 3 logs -> 1 session + 1 missing
    let logs = vec![
        LogEvent { minutes: 480, punch_type: None },
        LogEvent { minutes: 720, punch_type: None },
        LogEvent { minutes: 780, punch_type: None },
    ];
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[1].start_minutes, 780);
    assert!(sessions[1].end_minutes.is_none());
}

#[test]
fn test_strategies_fsm() {
    let strategy = FsmStrategy;

    // Case 1: Explicit In/Out
    let logs = vec![
        LogEvent { minutes: 480, punch_type: Some(0) }, // In
        LogEvent { minutes: 1020, punch_type: Some(1) }, // Out
    ];
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].start_minutes, 480);
    assert_eq!(sessions[0].end_minutes, Some(1020));

    // Case 2: Double Check-In (Error Recovery)
    let logs = vec![
        LogEvent { minutes: 480, punch_type: Some(0) }, // In
        LogEvent { minutes: 500, punch_type: Some(0) }, // In again!
        LogEvent { minutes: 1020, punch_type: Some(1) }, // Out
    ];
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 2);
    assert!(sessions[0].end_minutes.is_none()); // 1st closed as missing
    assert_eq!(sessions[1].start_minutes, 500);
    assert_eq!(sessions[1].end_minutes, Some(1020));

    // Case 3: Mixed metadata (Fallback to toggle)
    let logs = vec![
        LogEvent { minutes: 480, punch_type: Some(0) }, // In
        LogEvent { minutes: 720, punch_type: None },    // Unspecified (acts as Out)
        LogEvent { minutes: 780, punch_type: None },    // Unspecified (acts as In)
        LogEvent { minutes: 1020, punch_type: Some(1) }, // Out
    ];
    let sessions = strategy.pair(&logs, None);
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].end_minutes, Some(720));
    assert_eq!(sessions[1].start_minutes, 780);
}
