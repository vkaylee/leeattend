use apiserver::services::pairing_algo::{
    self, AnchorFirstStrategy, LogEvent, PairingShiftConfig, PairingStrategy,
};

fn standard_shift() -> PairingShiftConfig {
    PairingShiftConfig {
        start_minutes: 8 * 60,              // 480 (08:00)
        end_minutes: 17 * 60,               // 1020 (17:00)
        break_start_minutes: Some(12 * 60), // 720 (12:00)
        break_end_minutes: Some(13 * 60),   // 780 (13:00)
    }
}

#[test]
fn test_anchor_perfect_day() {
    let logs = vec![
        LogEvent { id: uuid::Uuid::new_v4(),
            minutes: 475,
            punch_type: None,
        },
        LogEvent { id: uuid::Uuid::new_v4(),
            minutes: 725,
            punch_type: None,
        },
        LogEvent { id: uuid::Uuid::new_v4(),
            minutes: 775,
            punch_type: None,
        },
        LogEvent { id: uuid::Uuid::new_v4(),
            minutes: 1025,
            punch_type: None,
        },
    ];
    let shift = standard_shift();

    // Anchor Strategy should find 1 session (First and Last)
    let anchor = AnchorFirstStrategy;
    let anchor_sessions = anchor.pair(&logs, Some(&shift));
    assert_eq!(anchor_sessions.len(), 1, "Anchor should find 1 session");
    assert_eq!(anchor_sessions[0].start_minutes, 475);
    assert_eq!(anchor_sessions[0].end_minutes, Some(1025));
}

#[test]
fn test_anchor_overtime() {
    let logs = vec![
        LogEvent { id: uuid::Uuid::new_v4(),
            minutes: 436, // 07:16
            punch_type: None,
        },
        LogEvent { id: uuid::Uuid::new_v4(),
            minutes: 1150, // 19:10 (Duration 714m < 720m limit)
            punch_type: None,
        },
    ];
    let shift = standard_shift();

    // Anchor Strategy should pair these as they are within 12h
    let anchor = AnchorFirstStrategy;
    let anchor_sessions = anchor.pair(&logs, Some(&shift));
    assert_eq!(anchor_sessions.len(), 1);
    assert_eq!(anchor_sessions[0].start_minutes, 436);
    assert_eq!(anchor_sessions[0].end_minutes, Some(1150));
}

#[test]
fn test_anchor_missing_out() {
    let logs = vec![LogEvent { id: uuid::Uuid::new_v4(),
        minutes: 480,
        punch_type: None,
    }];
    let shift = standard_shift();

    let anchor = AnchorFirstStrategy;
    let anchor_sessions = anchor.pair(&logs, Some(&shift));
    assert_eq!(anchor_sessions.len(), 1);
    assert!(anchor_sessions[0].end_minutes.is_none());
    assert_eq!(
        anchor_sessions[0].note,
        Some("Missing Punch Out".to_string())
    );
}

#[test]
fn test_factory_dispatch_defaults_to_anchor() {
    // All inputs should now return AnchorFirstStrategy behavior
    let anchor = pairing_algo::get_strategy("anchor_first");
    let other = pairing_algo::get_strategy("something_else");

    let logs = vec![
        LogEvent { id: uuid::Uuid::new_v4(),
            minutes: 100,
            punch_type: None,
        },
        LogEvent { id: uuid::Uuid::new_v4(),
            minutes: 200, // 100 min gap
            punch_type: None,
        },
        LogEvent { id: uuid::Uuid::new_v4(),
            minutes: 300,
            punch_type: None,
        },
    ];

        // All should find exactly 1 session (Anchor behavior: First=100, Last=300)

        assert_eq!(anchor.pair(&logs, None).len(), 1);

        assert_eq!(other.pair(&logs, None).len(), 1);

        

        assert_eq!(anchor.pair(&logs, None)[0].end_minutes, Some(300));

    }

    