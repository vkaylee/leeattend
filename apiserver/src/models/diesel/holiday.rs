use crate::models::Holiday;
use crate::schema::holidays;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = holidays)]
pub struct HolidayEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub date: NaiveDate,
    pub holiday_type: String,
    pub is_paid: bool,
    pub ot_multiplier: Option<Decimal>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub leave_type_id: Option<Uuid>,
    pub work_threshold_percent: Option<i32>,
    pub is_credit_on_work: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<HolidayEntity> for Holiday {
    fn from(entity: HolidayEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            date: entity.date,
            holiday_type: entity.holiday_type,
            is_paid: entity.is_paid,
            leave_type_id: entity.leave_type_id,
            ot_multiplier: entity.ot_multiplier,
            description: entity.description,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            work_threshold_percent: entity.work_threshold_percent,
            is_credit_on_work: entity.is_credit_on_work,
        }
    }
}
