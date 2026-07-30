#[cfg(test)]
mod tests {
    use super::super::*;
    use chrono::{NaiveDate, NaiveTime, Utc};
    use uuid::Uuid;

    // ── WorkSegment serialization ───────────────────────────────────────

    #[test]
    fn test_work_segment_json_roundtrip() {
        let seg = WorkSegment {
            id: Uuid::from_u128(1),
            workspace_id: Uuid::from_u128(100),
            employment_id: Uuid::from_u128(200),
            date: NaiveDate::from_ymd_opt(2026, 6, 24).unwrap(),
            shift_id: Uuid::from_u128(300),
            interval_id: Some(Uuid::from_u128(400)),
            sort_order: 0,
            scheduled_start: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            scheduled_end: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            segment_type: "work".to_string(),
            segment_status: "scheduled".to_string(),
            compensation_rate: 1.0,
            gap_compensation_before: "unpaid".to_string(),
            gap_compensation_rate_before: 0.0,
            break_config_override: None,
            ot_config_override: None,
            source: "roster".to_string(),
            source_detail: serde_json::json!({"key": "value"}),
            notes: Some("Test segment".to_string()),
            generation_metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&seg).expect("Serialize");
        let parsed: WorkSegment = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(parsed.id, seg.id);
        assert_eq!(parsed.date, seg.date);
        assert_eq!(parsed.segment_type, "work");
        assert_eq!(parsed.segment_status, "scheduled");
        assert_eq!(parsed.compensation_rate, 1.0);
        assert_eq!(parsed.gap_compensation_before, "unpaid");
        assert_eq!(parsed.notes, Some("Test segment".to_string()));
    }

    #[test]
    fn test_work_segment_standby_type() {
        let seg = WorkSegment {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            date: NaiveDate::from_ymd_opt(2026, 6, 24).unwrap(),
            shift_id: Uuid::new_v4(),
            interval_id: None,
            sort_order: 1,
            scheduled_start: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            scheduled_end: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            segment_type: "standby".to_string(),
            segment_status: "scheduled".to_string(),
            compensation_rate: 0.5,
            gap_compensation_before: "standby".to_string(),
            gap_compensation_rate_before: 0.3,
            break_config_override: None,
            ot_config_override: None,
            source: "manual".to_string(),
            source_detail: serde_json::json!({}),
            notes: None,
            generation_metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(seg.segment_type, "standby");
        assert_eq!(seg.compensation_rate, 0.5);
        assert_eq!(seg.gap_compensation_before, "standby");
        assert_eq!(seg.gap_compensation_rate_before, 0.3);
    }

    #[test]
    fn test_work_segment_on_call_type() {
        let seg = WorkSegment {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            date: NaiveDate::from_ymd_opt(2026, 6, 24).unwrap(),
            shift_id: Uuid::new_v4(),
            interval_id: None,
            sort_order: 2,
            scheduled_start: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            scheduled_end: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            segment_type: "on_call".to_string(),
            segment_status: "scheduled".to_string(),
            compensation_rate: 0.0,
            gap_compensation_before: "unpaid".to_string(),
            gap_compensation_rate_before: 0.0,
            break_config_override: None,
            ot_config_override: None,
            source: "pattern".to_string(),
            source_detail: serde_json::json!({}),
            notes: None,
            generation_metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(seg.segment_type, "on_call");
        assert_eq!(seg.compensation_rate, 0.0);
    }

    // ── CreateWorkSegmentRequest defaults ──────────────────────────────

    #[test]
    fn test_create_request_defaults() {
        let json = serde_json::json!({
            "employment_id": "00000000-0000-0000-0000-000000000001",
            "date": "2026-06-24",
            "shift_id": "00000000-0000-0000-0000-000000000002",
            "scheduled_start": "07:00:00",
            "scheduled_end": "11:00:00"
        });

        let req: CreateWorkSegmentRequest =
            serde_json::from_value(json).expect("Deserialize with defaults");

        assert_eq!(req.segment_type, "work");
        assert_eq!(req.segment_status, "scheduled");
        assert_eq!(req.compensation_rate, 1.0);
        assert_eq!(req.gap_compensation_before, "unpaid");
        assert_eq!(req.source, "manual");
        assert_eq!(req.sort_order, 0);
    }

    #[test]
    fn test_create_request_full() {
        let json = serde_json::json!({
            "employment_id": "00000000-0000-0000-0000-000000000001",
            "date": "2026-06-24",
            "shift_id": "00000000-0000-0000-0000-000000000002",
            "scheduled_start": "07:00:00",
            "scheduled_end": "11:00:00",
            "segment_type": "training",
            "segment_status": "scheduled",
            "compensation_rate": 0.8,
            "gap_compensation_before": "standby",
            "gap_compensation_rate_before": 0.5,
            "source": "roster",
            "notes": "Training session"
        });

        let req: CreateWorkSegmentRequest =
            serde_json::from_value(json).expect("Deserialize full");

        assert_eq!(req.segment_type, "training");
        assert_eq!(req.compensation_rate, 0.8);
        assert_eq!(req.gap_compensation_before, "standby");
        assert_eq!(req.gap_compensation_rate_before, 0.5);
        assert_eq!(req.notes, Some("Training session".to_string()));
    }

    // ── ShiftDefaultInterval ────────────────────────────────────────────

    #[test]
    fn test_shift_interval_json_roundtrip() {
        let interval = ShiftDefaultInterval {
            id: Uuid::from_u128(10),
            shift_id: Uuid::from_u128(20),
            sort_order: 0,
            interval_name: Some("Ca sáng".to_string()),
            start_time: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            break_duration_minutes: Some(15),
            break_start: Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(9, 15, 0).unwrap()),
            is_break_auto_deduct: Some(true),
            break_is_overtime: Some(false),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&interval).expect("Serialize");
        let parsed: ShiftDefaultInterval = serde_json::from_str(&json).expect("Deserialize");

        assert_eq!(parsed.id, interval.id);
        assert_eq!(parsed.shift_id, interval.shift_id);
        assert_eq!(parsed.interval_name, Some("Ca sáng".to_string()));
        assert_eq!(parsed.break_duration_minutes, Some(15));
    }

    #[test]
    fn test_shift_interval_null_inherits_from_shift() {
        // When break fields are null, the interval inherits from parent shift
        let interval = ShiftDefaultInterval {
            id: Uuid::new_v4(),
            shift_id: Uuid::new_v4(),
            sort_order: 1,
            interval_name: None,
            start_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(21, 0, 0).unwrap(),
            break_duration_minutes: None,    // inherit from shift
            break_start: None,               // inherit from shift
            break_end: None,                 // inherit from shift
            is_break_auto_deduct: None,      // inherit from shift
            break_is_overtime: None,         // inherit from shift
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(interval.break_duration_minutes.is_none());
        assert!(interval.is_break_auto_deduct.is_none());
        assert!(interval.break_is_overtime.is_none());
    }

    // ── ResolvedSegment ──────────────────────────────────────────────────

    #[test]
    fn test_resolved_segment_defaults() {
        let seg = ResolvedSegment {
            shift_id: Uuid::from_u128(1),
            interval_id: None,
            sort_order: 0,
            scheduled_start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            scheduled_end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            source: "fixed".to_string(),
            break_duration_minutes: Some(60),
            break_start: None,
            break_end: None,
            is_break_auto_deduct: Some(true),
            break_is_overtime: Some(false),
            allow_early_ot: false,
            max_early_ot_minutes: None,
            early_punch_in_grace_period_minutes: 0,
            ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            is_night_shift: false,
        };

        assert_eq!(seg.shift_id, Uuid::from_u128(1));
        assert_eq!(seg.source, "fixed");
        assert_eq!(seg.ot_aggregation_mode, crate::services::attendance_calculator::OtAggregationMode::Aggregate);
        assert!(!seg.is_night_shift);
    }

    // ── BulkCreateWorkSegmentsRequest ────────────────────────────────────

    #[test]
    fn test_bulk_create_request_replace_default() {
        let json = serde_json::json!({
            "segments": [],
            "replace_existing": true
        });

        let req: BulkCreateWorkSegmentsRequest =
            serde_json::from_value(json).expect("Deserialize");

        assert!(req.replace_existing);
        assert!(req.segments.is_empty());
    }

    #[test]
    fn test_bulk_create_request_default_false() {
        let json = serde_json::json!({
            "segments": []
        });

        let req: BulkCreateWorkSegmentsRequest =
            serde_json::from_value(json).expect("Deserialize");

        assert!(!req.replace_existing); // default is false
    }

    // ── Edge cases ──────────────────────────────────────────────────────

    #[test]
    fn test_work_segment_all_types() {
        let types = vec!["work", "standby", "on_call", "training", "travel"];
        for t in &types {
            let seg = WorkSegment {
                id: Uuid::new_v4(),
                workspace_id: Uuid::new_v4(),
                employment_id: Uuid::new_v4(),
                date: NaiveDate::from_ymd_opt(2026, 6, 24).unwrap(),
                shift_id: Uuid::new_v4(),
                interval_id: None,
                sort_order: 0,
                scheduled_start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                scheduled_end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                segment_type: t.to_string(),
                segment_status: "scheduled".to_string(),
                compensation_rate: 1.0,
                gap_compensation_before: "unpaid".to_string(),
                gap_compensation_rate_before: 0.0,
                break_config_override: None,
                ot_config_override: None,
                source: "manual".to_string(),
                source_detail: serde_json::json!({}),
                notes: None,
                generation_metadata: serde_json::json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            assert_eq!(seg.segment_type, t.to_string());
        }
    }

    #[test]
    fn test_work_segment_all_sources() {
        let sources = vec!["roster", "pattern", "rule", "fixed", "manual", "auto"];
        for s in &sources {
            let seg = WorkSegment {
                id: Uuid::new_v4(),
                workspace_id: Uuid::new_v4(),
                employment_id: Uuid::new_v4(),
                date: NaiveDate::from_ymd_opt(2026, 6, 24).unwrap(),
                shift_id: Uuid::new_v4(),
                interval_id: None,
                sort_order: 0,
                scheduled_start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                scheduled_end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                segment_type: "work".to_string(),
                segment_status: "scheduled".to_string(),
                compensation_rate: 1.0,
                gap_compensation_before: "unpaid".to_string(),
                gap_compensation_rate_before: 0.0,
                break_config_override: None,
                ot_config_override: None,
                source: s.to_string(),
                source_detail: serde_json::json!({}),
                notes: None,
                generation_metadata: serde_json::json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            assert_eq!(seg.source, s.to_string());
        }
    }

    #[test]
    fn test_gap_compensation_rate_bounds() {
        // Valid rates: 0.0 to 1.0
        let valid_rates = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        for rate in valid_rates {
            let seg = WorkSegment {
                id: Uuid::new_v4(),
                workspace_id: Uuid::new_v4(),
                employment_id: Uuid::new_v4(),
                date: NaiveDate::from_ymd_opt(2026, 6, 24).unwrap(),
                shift_id: Uuid::new_v4(),
                interval_id: None,
                sort_order: 0,
                scheduled_start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                scheduled_end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                segment_type: "work".to_string(),
                segment_status: "scheduled".to_string(),
                compensation_rate: rate,
                gap_compensation_before: "unpaid".to_string(),
                gap_compensation_rate_before: rate,
                break_config_override: None,
                ot_config_override: None,
                source: "manual".to_string(),
                source_detail: serde_json::json!({}),
                notes: None,
                generation_metadata: serde_json::json!({}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            assert_eq!(seg.compensation_rate, rate);
        }
    }

    #[test]
    fn test_break_config_override_json() {
        let override_config = serde_json::json!({
            "break_duration_minutes": 30,
            "is_break_auto_deduct": false
        });

        let seg = WorkSegment {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            date: NaiveDate::from_ymd_opt(2026, 6, 24).unwrap(),
            shift_id: Uuid::new_v4(),
            interval_id: None,
            sort_order: 0,
            scheduled_start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            scheduled_end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            segment_type: "work".to_string(),
            segment_status: "scheduled".to_string(),
            compensation_rate: 1.0,
            gap_compensation_before: "unpaid".to_string(),
            gap_compensation_rate_before: 0.0,
            break_config_override: Some(override_config.clone()),
            ot_config_override: None,
            source: "manual".to_string(),
            source_detail: serde_json::json!({}),
            notes: None,
            generation_metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let override_val = seg.break_config_override.unwrap();
        assert_eq!(override_val["break_duration_minutes"], 30);
        assert_eq!(override_val["is_break_auto_deduct"], false);
    }
}
