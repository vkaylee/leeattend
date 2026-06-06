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
        Self::new(
            entity.id,
            entity.workspace_id,
            entity.name,
            entity.date,
            entity.holiday_type,
            entity.is_paid,
            entity.leave_type_id,
            entity.ot_multiplier,
            entity.description,
            entity.work_threshold_percent,
            entity.is_credit_on_work,
            entity.created_at,
            entity.updated_at,
        )
    }
}
