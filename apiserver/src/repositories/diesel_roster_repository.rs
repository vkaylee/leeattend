use crate::errors::AppError;
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::models::diesel::roster::ShiftRosterEntity;
use crate::models::{AssignmentHistory, EmployeeSchedule, RosterItem, ScheduleTrace, ShiftRoster};
use crate::repositories::roster::RosterRepository;
use crate::schema::shift_rosters;

pub struct DieselRosterRepository {
    pub sqlx_pool: sqlx::PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselRosterRepository {
    pub fn new(sqlx_pool: sqlx::PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl RosterRepository for DieselRosterRepository {
    async fn get_workspace_schedules(
        &self,
        roster_cache: Option<&crate::cache::RosterCache>,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        employee_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<EmployeeSchedule>, AppError> {
        crate::repositories::roster::schedule_queries::get_workspace_schedules(
            &self.sqlx_pool,
            roster_cache,
            workspace_id,
            start_date,
            end_date,
            employee_ids,
        )
        .await
        .map_err(Into::into)
    }

    async fn get_assignment_history(
        &self,
        workspace_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AssignmentHistory>, AppError> {
        crate::repositories::roster::roster_crud::get_assignment_history(
            &self.sqlx_pool,
            workspace_id,
            limit,
        )
        .await
        .map_err(Into::into)
    }

    async fn get_employee_rosters(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ShiftRoster>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities: Vec<ShiftRosterEntity> = shift_rosters::table
            .filter(shift_rosters::employee_id.eq(employee_id))
            .filter(shift_rosters::date.ge(start_date))
            .filter(shift_rosters::date.le(end_date))
            .order(shift_rosters::date.asc())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn upsert_roster(&self, roster: &ShiftRoster) -> Result<ShiftRoster, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = ShiftRosterEntity {
            id: roster.id,
            employee_id: roster.employee_id,
            shift_id: roster.shift_id,
            date: roster.date,
            status: Some(roster.status.clone()),
            notes: roster.notes.clone(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let result: ShiftRosterEntity = diesel::insert_into(shift_rosters::table)
            .values(&entity)
            .on_conflict((shift_rosters::employee_id, shift_rosters::date))
            .do_update()
            .set((
                shift_rosters::shift_id.eq(entity.shift_id),
                shift_rosters::status.eq(&entity.status),
                shift_rosters::notes.eq(&entity.notes),
                shift_rosters::updated_at.eq(Utc::now()),
            ))
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn bulk_upsert_roster(
        &self,
        employee_id: Uuid,
        items: Vec<RosterItem>,
    ) -> Result<(), AppError> {
        crate::repositories::roster::roster_crud::bulk_upsert_roster(
            &self.sqlx_pool,
            employee_id,
            items,
        )
        .await
        .map_err(Into::into)
    }

    async fn get_effective_shift(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<(Option<Uuid>, Vec<ScheduleTrace>), AppError> {
        crate::repositories::roster::schedule_queries::get_effective_shift(
            &self.sqlx_pool,
            workspace_id,
            employee_id,
            date,
        )
        .await
        .map_err(Into::into)
    }

    async fn delete_roster(&self, employee_id: Uuid, date: NaiveDate) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::delete(shift_rosters::table)
            .filter(shift_rosters::employee_id.eq(employee_id))
            .filter(shift_rosters::date.eq(date))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}

impl crate::repositories::PoolBacked for DieselRosterRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: sqlx::PgPool) -> Self {
        panic!("DieselRosterRepository requires DieselPools. Use new() instead.");
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.sqlx_pool
    }
}
