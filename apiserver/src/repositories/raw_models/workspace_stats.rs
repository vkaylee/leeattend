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
