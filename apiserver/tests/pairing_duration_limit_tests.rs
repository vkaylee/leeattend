use apiserver::services::pairing_algo::{
    AnchorFirstStrategy, LogEvent, PairingStrategy,
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

    // Log 2: 01:00 (next day, 1500 min) -> Diff 1020 min (17h) - SHOULD NOT PAIR (>16h)

    let logs_steal = vec![

        LogEvent { id: Uuid::new_v4(), minutes: 480, punch_type: None },

        LogEvent { id: Uuid::new_v4(), minutes: 1500, punch_type: None },

    ];

    let sessions = strategy.pair(&logs_steal, None);

    assert_eq!(sessions.len(), 1);

    assert!(sessions[0].end_minutes.is_none(), "Should not pair logs with >16h gap");

}
