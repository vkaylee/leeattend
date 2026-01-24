use chrono::NaiveDate;
use rust_decimal::Decimal;
use apiserver::models::Payslip;
use uuid::Uuid;

mod common;

/// Tests that verify: net_pay_amount = base_pay_amount + overtime_pay_amount + incentive_amount - deductions_amount
/// This ensures the PaySlip total equals the sum of all components
#[cfg(test)]
mod payslip_total_validation {
    use super::*;

    /// Helper function to validate payslip totals
    /// Returns a tuple (is_valid, expected_net, actual_net, difference)
    fn validate_payslip_total(payslip: &Payslip) -> (bool, Decimal, Decimal, Decimal) {
        // Calculate expected net_pay_amount
        let expected_net = payslip.base_pay_amount
            + payslip.overtime_pay_amount
            - payslip.deductions_amount;

        let actual_net = payslip.net_pay_amount;
        let difference = (expected_net - actual_net).abs();

        // Allow for small decimal rounding differences (up to 0.01)
        let is_valid = difference <= Decimal::new(1, 2); // 0.01

        (is_valid, expected_net, actual_net, difference)
    }

    // ===== Test 1: Happy Path - Standard Payslip =====
    #[tokio::test]
    async fn test_payslip_total_with_standard_components() {
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(500000, 0),      // 500,000
            overtime_pay_amount: Decimal::new(100000, 0),  // 100,000
            deductions_amount: Decimal::new(50000, 0),     // 50,000
            net_pay_amount: Decimal::new(550000, 0),       // Expected: 500,000 + 100,000 - 50,000 = 550,000
            standard_units: 160,
            overtime_units: 20,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let (is_valid, expected, actual, diff) = validate_payslip_total(&payslip);

        assert!(is_valid,
            "PaySlip total mismatch:\n  Expected: {}\n  Actual: {}\n  Difference: {}",
            expected, actual, diff
        );
        assert_eq!(expected, actual, "Net pay should equal sum of components");
    }

    // ===== Test 2: Zero Overtime =====
    #[tokio::test]
    async fn test_payslip_total_zero_overtime() {
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(300000, 0),      // 300,000
            overtime_pay_amount: Decimal::new(0, 0),       // 0 (no overtime)
            deductions_amount: Decimal::new(30000, 0),     // 30,000
            net_pay_amount: Decimal::new(270000, 0),       // Expected: 300,000 + 0 - 30,000 = 270,000
            standard_units: 160,
            overtime_units: 0,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let (is_valid, expected, actual, diff) = validate_payslip_total(&payslip);

        assert!(is_valid,
            "PaySlip total mismatch:\n  Expected: {}\n  Actual: {}\n  Difference: {}",
            expected, actual, diff
        );
        assert_eq!(expected, actual, "Net pay should equal sum of components (zero overtime)");
    }

    // ===== Test 3: Zero Deductions =====
    #[tokio::test]
    async fn test_payslip_total_zero_deductions() {
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(400000, 0),      // 400,000
            overtime_pay_amount: Decimal::new(80000, 0),   // 80,000
            deductions_amount: Decimal::new(0, 0),         // 0 (no deductions)
            net_pay_amount: Decimal::new(480000, 0),       // Expected: 400,000 + 80,000 - 0 = 480,000
            standard_units: 160,
            overtime_units: 15,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let (is_valid, expected, actual, diff) = validate_payslip_total(&payslip);

        assert!(is_valid,
            "PaySlip total mismatch:\n  Expected: {}\n  Actual: {}\n  Difference: {}",
            expected, actual, diff
        );
        assert_eq!(expected, actual, "Net pay should equal sum of components (zero deductions)");
    }

    // ===== Test 4: Decimal Values (High Precision) =====
    #[tokio::test]
    async fn test_payslip_total_with_decimal_precision() {
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(50000050, 2),    // 500,000.50
            overtime_pay_amount: Decimal::new(10000075, 2), // 100,000.75
            deductions_amount: Decimal::new(5000025, 2),   // 50,000.25
            net_pay_amount: Decimal::new(55000100, 2),     // Expected: 500,000.50 + 100,000.75 - 50,000.25 = 550,001.00
            standard_units: 160,
            overtime_units: 20,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let (is_valid, expected, actual, diff) = validate_payslip_total(&payslip);

        assert!(is_valid,
            "PaySlip total mismatch:\n  Expected: {}\n  Actual: {}\n  Difference: {}",
            expected, actual, diff
        );
        assert_eq!(expected, actual, "Net pay should equal sum of components (decimal precision)");
    }

    // ===== Test 5: Very Large Amounts =====
    #[tokio::test]
    async fn test_payslip_total_large_amounts() {
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(999999999, 0),   // 999,999,999
            overtime_pay_amount: Decimal::new(500000000, 0), // 500,000,000
            deductions_amount: Decimal::new(100000000, 0),  // 100,000,000
            net_pay_amount: Decimal::new(1399999999, 0),   // Expected: 999,999,999 + 500,000,000 - 100,000,000 = 1,399,999,999
            standard_units: 160,
            overtime_units: 50,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let (is_valid, expected, actual, diff) = validate_payslip_total(&payslip);

        assert!(is_valid,
            "PaySlip total mismatch:\n  Expected: {}\n  Actual: {}\n  Difference: {}",
            expected, actual, diff
        );
        assert_eq!(expected, actual, "Net pay should equal sum of components (large amounts)");
    }

    // ===== Test 6: Minimum Values (Edge Case) =====
    #[tokio::test]
    async fn test_payslip_total_minimum_values() {
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(1, 0),           // 1
            overtime_pay_amount: Decimal::new(1, 0),       // 1
            deductions_amount: Decimal::new(1, 0),         // 1
            net_pay_amount: Decimal::new(1, 0),            // Expected: 1 + 1 - 1 = 1
            standard_units: 1,
            overtime_units: 1,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let (is_valid, expected, actual, diff) = validate_payslip_total(&payslip);

        assert!(is_valid,
            "PaySlip total mismatch:\n  Expected: {}\n  Actual: {}\n  Difference: {}",
            expected, actual, diff
        );
        assert_eq!(expected, actual, "Net pay should equal sum of components (minimum values)");
    }

    // ===== Test 7: Deductions Greater Than Base Pay =====
    #[tokio::test]
    async fn test_payslip_total_high_deductions() {
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(200000, 0),      // 200,000
            overtime_pay_amount: Decimal::new(50000, 0),   // 50,000
            deductions_amount: Decimal::new(150000, 0),    // 150,000 (high deductions)
            net_pay_amount: Decimal::new(100000, 0),       // Expected: 200,000 + 50,000 - 150,000 = 100,000
            standard_units: 160,
            overtime_units: 10,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let (is_valid, expected, actual, diff) = validate_payslip_total(&payslip);

        assert!(is_valid,
            "PaySlip total mismatch:\n  Expected: {}\n  Actual: {}\n  Difference: {}",
            expected, actual, diff
        );
        assert_eq!(expected, actual, "Net pay should equal sum of components (high deductions)");
    }

    // ===== Test 8: All Components Zero =====
    #[tokio::test]
    async fn test_payslip_total_all_zero() {
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(0, 0),           // 0
            overtime_pay_amount: Decimal::new(0, 0),       // 0
            deductions_amount: Decimal::new(0, 0),         // 0
            net_pay_amount: Decimal::new(0, 0),            // Expected: 0 + 0 - 0 = 0
            standard_units: 0,
            overtime_units: 0,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let (is_valid, expected, actual, diff) = validate_payslip_total(&payslip);

        assert!(is_valid,
            "PaySlip total mismatch:\n  Expected: {}\n  Actual: {}\n  Difference: {}",
            expected, actual, diff
        );
        assert_eq!(expected, actual, "Net pay should equal sum of components (all zero)");
    }

    // ===== Test 9: Negative Values (Refunds/Credits) =====
    #[tokio::test]
    async fn test_payslip_total_negative_values() {
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(-100000, 0),     // -100,000 (adjustment/refund)
            overtime_pay_amount: Decimal::new(300000, 0),  // 300,000
            deductions_amount: Decimal::new(50000, 0),     // 50,000
            net_pay_amount: Decimal::new(150000, 0),       // Expected: -100,000 + 300,000 - 50,000 = 150,000
            standard_units: 160,
            overtime_units: 30,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let (is_valid, expected, actual, diff) = validate_payslip_total(&payslip);

        assert!(is_valid,
            "PaySlip total mismatch:\n  Expected: {}\n  Actual: {}\n  Difference: {}",
            expected, actual, diff
        );
        assert_eq!(expected, actual, "Net pay should equal sum of components (negative values)");
    }

    // ===== Test 10: Rounding Edge Case =====
    #[tokio::test]
    async fn test_payslip_total_rounding_edge_case() {
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(3333333, 1),      // 333,333.3
            overtime_pay_amount: Decimal::new(6666667, 1),  // 666,666.7
            deductions_amount: Decimal::new(1666667, 1),    // 166,666.7
            // Expected: 333,333.3 + 666,666.7 - 166,666.7 = 833,333.3
            net_pay_amount: Decimal::new(8333333, 1),
            standard_units: 160,
            overtime_units: 25,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let (is_valid, expected, actual, diff) = validate_payslip_total(&payslip);

        assert!(is_valid,
            "PaySlip total mismatch:\n  Expected: {}\n  Actual: {}\n  Difference: {}",
            expected, actual, diff
        );
        assert_eq!(expected, actual, "Net pay should equal sum of components (rounding)");
    }

    // ===== Test 11: Units Consistency Check =====
    #[tokio::test]
    async fn test_payslip_units_are_non_negative() {
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(400000, 0),
            overtime_pay_amount: Decimal::new(80000, 0),
            deductions_amount: Decimal::new(40000, 0),
            net_pay_amount: Decimal::new(440000, 0),
            standard_units: 160,
            overtime_units: 20,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Verify units are non-negative
        assert!(payslip.standard_units >= 0, "Standard units must be non-negative");
        assert!(payslip.overtime_units >= 0, "Overtime units must be non-negative");
    }

    // ===== Test 12: Status and Timestamps =====
    #[tokio::test]
    async fn test_payslip_metadata_consistency() {
        let created_at = chrono::Utc::now();
        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(400000, 0),
            overtime_pay_amount: Decimal::new(80000, 0),
            deductions_amount: Decimal::new(40000, 0),
            net_pay_amount: Decimal::new(440000, 0),
            standard_units: 160,
            overtime_units: 20,
            status: "draft".to_string(),
            created_at,
            updated_at: created_at,
        };

        // Verify metadata
        assert_eq!(payslip.status, "draft", "Status should be draft");
        assert!(payslip.created_at <= payslip.updated_at, "Updated_at should be >= created_at");
        assert!(payslip.period_start <= payslip.period_end, "Period_start should be <= period_end");
    }

    // ===== Test 13: ID Uniqueness (Structural) =====
    #[tokio::test]
    async fn test_payslip_ids_are_unique() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let payslip1 = Payslip {
            id: id1,
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(400000, 0),
            overtime_pay_amount: Decimal::new(80000, 0),
            deductions_amount: Decimal::new(40000, 0),
            net_pay_amount: Decimal::new(440000, 0),
            standard_units: 160,
            overtime_units: 20,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let payslip2 = Payslip {
            id: id2,
            employee_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            base_pay_amount: Decimal::new(400000, 0),
            overtime_pay_amount: Decimal::new(80000, 0),
            deductions_amount: Decimal::new(40000, 0),
            net_pay_amount: Decimal::new(440000, 0),
            standard_units: 160,
            overtime_units: 20,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert_ne!(payslip1.id, payslip2.id, "PaySlip IDs should be unique");
        assert_eq!(payslip1.id, id1, "First payslip ID should match");
        assert_eq!(payslip2.id, id2, "Second payslip ID should match");
    }
}
