
use apiserver::services::pairing_algo::{self, PairingShiftConfig, PairingStrategy, DpPatternStrategy, AnchorFirstStrategy};

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
    let logs = vec![475, 725, 775, 1025]; // 7:55, 12:05, 12:55, 17:05
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
    let logs = vec![436, 1204]; // 07:16, 20:04
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
    let logs = vec![480]; // 08:00
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
    let random = pairing_algo::get_strategy("something_else");

    let logs = vec![100, 200, 300, 400];
    
    // DP needs shift or it fallbacks to anchor behavior (1 session)
    assert_eq!(dp.pair(&logs, None).len(), 1);
    assert_eq!(anchor.pair(&logs, None).len(), 1);
    assert_eq!(random.pair(&logs, None).len(), 1); // defaults to DP -> fallback anchor
}
