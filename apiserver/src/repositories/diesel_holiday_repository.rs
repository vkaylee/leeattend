use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::holiday::HolidayEntity;
use crate::models::Holiday;
use crate::repositories::holiday_repository::HolidayRepositoryTrait;
use crate::schema::holidays;

pub struct DieselHolidayRepository {
    pub sqlx_pool: sqlx::PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselHolidayRepository {
    pub fn new(sqlx_pool: sqlx::PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl HolidayRepositoryTrait for DieselHolidayRepository {
    async fn list(&self, workspace_id: Uuid, year: Option<i32>) -> Result<Vec<Holiday>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut query = holidays::table
            .filter(holidays::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(y) = year {
            if let (Some(start), Some(end)) = (
                NaiveDate::from_ymd_opt(y, 1, 1),
                NaiveDate::from_ymd_opt(y, 12, 31),
            ) {
                query = query
                    .filter(holidays::date.ge(start))
                    .filter(holidays::date.le(end));
            }
        }

        let entities: Vec<HolidayEntity> = query
            .order(holidays::date.asc())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_by_id(&self, workspace_id: Uuid, id: Uuid) -> Result<Option<Holiday>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity: Option<HolidayEntity> = holidays::table
            .filter(holidays::id.eq(id))
            .filter(holidays::workspace_id.eq(workspace_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn create_holiday(
        &self,
        workspace_id: Uuid,
        req: &crate::models::CreateHolidayRequest,
    ) -> Result<Holiday, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let new_holiday = HolidayEntity {
            id: Uuid::new_v4(),
            workspace_id,
            name: req.name.clone(),
            date: req.date,
            holiday_type: req.holiday_type.clone(),
            is_paid: req.is_paid,
            leave_type_id: req.leave_type_id,
            ot_multiplier: req.ot_multiplier,
            description: req.description.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            work_threshold_percent: req.work_threshold_percent,
            is_credit_on_work: req.is_credit_on_work,
        };

        let result: HolidayEntity = diesel::insert_into(holidays::table)
            .values(&new_holiday)
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn update_holiday(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        req: &crate::models::UpdateHolidayRequest,
    ) -> Result<Option<Holiday>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result: Option<HolidayEntity> = diesel::update(holidays::table)
            .filter(holidays::id.eq(id))
            .filter(holidays::workspace_id.eq(workspace_id))
            .set((
                req.name.as_ref().map(|n| holidays::name.eq(n)),
                req.date.map(|d| holidays::date.eq(d)),
                req.holiday_type
                    .as_ref()
                    .map(|t| holidays::holiday_type.eq(t)),
                req.is_paid.map(|p| holidays::is_paid.eq(p)),
                req.leave_type_id.map(|l| holidays::leave_type_id.eq(l)),
                req.ot_multiplier.map(|m| holidays::ot_multiplier.eq(m)),
                req.description
                    .as_ref()
                    .map(|d| holidays::description.eq(d)),
                req.work_threshold_percent
                    .map(|w| holidays::work_threshold_percent.eq(w)),
                req.is_credit_on_work
                    .map(|c| holidays::is_credit_on_work.eq(c)),
                Some(holidays::updated_at.eq(Utc::now())),
            ))
            .get_result(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<bool, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let affected = diesel::delete(holidays::table)
            .filter(holidays::id.eq(id))
            .filter(holidays::workspace_id.eq(workspace_id))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected > 0)
    }

    async fn get_holidays_in_range(
        &self,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<NaiveDate, Holiday>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities: Vec<HolidayEntity> = holidays::table
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::date.ge(start_date))
            .filter(holidays::date.le(end_date))
            .order(holidays::date.asc())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(|e| (e.date, e.into())).collect())
    }

    async fn is_holiday(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Holiday>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity: Option<HolidayEntity> = holidays::table
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::date.eq(date))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }
}
