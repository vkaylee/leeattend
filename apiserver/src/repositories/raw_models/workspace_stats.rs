use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::sql_types::*;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(QueryableByName)]
pub struct TzRow {
    #[diesel(sql_type = Text)]
    pub timezone: String,
}

#[derive(QueryableByName)]
pub struct BaseRow {
    #[diesel(sql_type = Date)]
    pub date: NaiveDate,
    #[diesel(sql_type = BigInt)]
    pub present: i64,
    #[diesel(sql_type = BigInt)]
    pub on_leave: i64,
    #[diesel(sql_type = BigInt)]
    pub active_employees: i64,
}

#[derive(QueryableByName)]
pub struct LateRow {
    #[diesel(sql_type = Date)]
    pub work_date: NaiveDate,
    #[diesel(sql_type = BigInt)]
    pub late_count: i64,
}

#[derive(QueryableByName)]
pub struct CountRow {
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(QueryableByName)]
pub struct DeptRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub department_id: Uuid,
    #[diesel(sql_type = Text)]
    pub department_name: String,
    #[diesel(sql_type = BigInt)]
    pub total_employees: i64,
}

#[derive(QueryableByName)]
pub struct PresentRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub department_id: Uuid,
    #[diesel(sql_type = BigInt)]
    pub present_count: i64,
}

#[derive(QueryableByName)]
pub struct LateDepartmentRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub department_id: Uuid,
    #[diesel(sql_type = BigInt)]
    pub late_count: i64,
}

#[derive(QueryableByName)]
pub struct LeaveRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub department_id: Uuid,
    #[diesel(sql_type = BigInt)]
    pub leave_count: i64,
}

#[derive(QueryableByName)]
pub struct OtRow {
    #[diesel(sql_type = Date)]
    pub date: NaiveDate,
    #[diesel(sql_type = BigInt)]
    pub system_ot_minutes: i64,
    #[diesel(sql_type = BigInt)]
    pub approved_ot_minutes: i64,
}

#[derive(QueryableByName)]
pub struct DecimalRow {
    #[diesel(sql_type = Numeric)]
    pub amount: Decimal,
}

#[derive(QueryableByName)]
pub struct FloatRow {
    #[diesel(sql_type = Nullable<Double>)]
    pub amount: Option<f64>,
}

#[derive(QueryableByName)]
pub struct SlotRow {
    #[diesel(sql_type = Text)]
    pub time_slot: String,
    #[diesel(sql_type = BigInt)]
    pub check_in: i64,
    #[diesel(sql_type = BigInt)]
    pub check_out: i64,
}

#[derive(QueryableByName)]
pub struct LtRow {
    #[diesel(sql_type = Text)]
    pub leave_type: String,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(QueryableByName)]
pub struct DayRow {
    #[diesel(sql_type = Integer)]
    pub dow: i32,
    #[diesel(sql_type = BigInt)]
    pub late_count: i64,
    #[diesel(sql_type = Double)]
    pub avg_late_minutes: f64,
}

#[derive(QueryableByName)]
pub struct EmployeeFirstPunchRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub employment_id: Uuid,
    #[diesel(sql_type = Date)]
    pub work_date: NaiveDate,
    #[diesel(sql_type = Time)]
    pub first_punch: chrono::NaiveTime,
}

#[derive(QueryableByName)]
pub struct EmployeeFirstPunchTodayRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub employment_id: Uuid,
    #[diesel(sql_type = Time)]
    pub first_punch: chrono::NaiveTime,
}

#[derive(QueryableByName)]
pub struct EmployeeLastPunchRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub employment_id: Uuid,
    #[diesel(sql_type = Time)]
    pub last_punch_out: chrono::NaiveTime,
}

#[derive(QueryableByName)]
pub struct EmployeeDepartmentRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub employment_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub department_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_raw_models_instantiation() {
        let tz = TzRow {
            timezone: "UTC".to_string(),
        };
        assert_eq!(tz.timezone, "UTC");

        let base = BaseRow {
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            present: 10,
            on_leave: 2,
            active_employees: 20,
        };
        assert_eq!(base.present, 10);

        let late = LateRow {
            work_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            late_count: 5,
        };
        assert_eq!(late.late_count, 5);

        let count = CountRow { count: 42 };
        assert_eq!(count.count, 42);

        let dept = DeptRow {
            department_id: Uuid::new_v4(),
            department_name: "Engineering".to_string(),
            total_employees: 50,
        };
        assert_eq!(dept.total_employees, 50);

        let present = PresentRow {
            department_id: Uuid::new_v4(),
            present_count: 45,
        };
        assert_eq!(present.present_count, 45);

        let late_dept = LateDepartmentRow {
            department_id: Uuid::new_v4(),
            late_count: 3,
        };
        assert_eq!(late_dept.late_count, 3);

        let leave = LeaveRow {
            department_id: Uuid::new_v4(),
            leave_count: 2,
        };
        assert_eq!(leave.leave_count, 2);

        let ot = OtRow {
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            system_ot_minutes: 120,
            approved_ot_minutes: 60,
        };
        assert_eq!(ot.system_ot_minutes, 120);

        let dec_row = DecimalRow {
            amount: Decimal::from_str("123.45").unwrap(),
        };
        assert_eq!(dec_row.amount, Decimal::from_str("123.45").unwrap());

        let float_row = FloatRow { amount: Some(12.3) };
        assert_eq!(float_row.amount, Some(12.3));

        let slot = SlotRow {
            time_slot: "08:00".to_string(),
            check_in: 20,
            check_out: 0,
        };
        assert_eq!(slot.check_in, 20);

        let lt = LtRow {
            leave_type: "annual".to_string(),
            count: 5,
        };
        assert_eq!(lt.count, 5);

        let day = DayRow {
            dow: 1,
            late_count: 10,
            avg_late_minutes: 15.5,
        };
        assert_eq!(day.avg_late_minutes, 15.5);
    }
}
