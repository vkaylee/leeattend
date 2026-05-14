use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::diesel::time_entry::{
    NewTimeEntryEntity, TimeEntryEntity, UpdateTimeEntryEntity,
};
use crate::models::{TimeEntry, TimeEntryLogFilter, TimeEntryWithEmployee};
use crate::repositories::time_entry_repository::TimeEntryRepository;
use crate::schema::{employees, time_entries};

pub struct DieselTimeEntryRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselTimeEntryRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl TimeEntryRepository for DieselTimeEntryRepository {
    async fn create_time_entry(&self, entry: &TimeEntry) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let new_entry = NewTimeEntryEntity {
            id: entry.id,
            employee_id: entry.employee_id,
            workspace_id: entry.workspace_id,
            punch_in_time: entry.punch_in_time,
            punch_out_time: entry.punch_out_time,
            entry_type: entry.entry_type.clone(),
            is_break_overtime: entry.is_break_overtime,
            note: entry.note.clone(),
            shift_id: entry.shift_id,
        };

        diesel::insert_into(time_entries::table)
            .values(&new_entry)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_time_entry_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entry: Option<TimeEntryEntity> = time_entries::table
            .filter(time_entries::workspace_id.eq(workspace_id))
            .filter(time_entries::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entry.map(Into::into))
    }

    async fn get_active_time_entry(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entry: Option<TimeEntryEntity> = time_entries::table
            .filter(time_entries::workspace_id.eq(workspace_id))
            .filter(time_entries::employee_id.eq(employee_id))
            .filter(time_entries::punch_out_time.is_null())
            .order(time_entries::punch_in_time.desc())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entry.map(Into::into))
    }

    async fn get_latest_active_entry(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError> {
        self.get_active_time_entry(workspace_id, employee_id).await
    }

    async fn get_latest_entry_for_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entry: Option<TimeEntryEntity> = time_entries::table
            .filter(time_entries::workspace_id.eq(workspace_id))
            .filter(time_entries::employee_id.eq(employee_id))
            .order(time_entries::punch_in_time.desc())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entry.map(Into::into))
    }

    async fn update_time_entry(
        &self,
        workspace_id: Uuid,
        entry: &TimeEntry,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let update_model = UpdateTimeEntryEntity {
            punch_in_time: Some(entry.punch_in_time),
            punch_out_time: Some(entry.punch_out_time),
            entry_type: Some(entry.entry_type.clone()),
            note: Some(entry.note.clone()),
            updated_at: Some(Utc::now()),
        };

        diesel::update(time_entries::table)
            .filter(time_entries::id.eq(entry.id))
            .filter(time_entries::workspace_id.eq(workspace_id))
            .set(&update_model)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn delete_time_entry(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::delete(time_entries::table)
            .filter(time_entries::id.eq(id))
            .filter(time_entries::workspace_id.eq(workspace_id))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_employee_time_entries(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entries: Vec<TimeEntryEntity> = time_entries::table
            .filter(time_entries::workspace_id.eq(workspace_id))
            .filter(time_entries::employee_id.eq(employee_id))
            .filter(time_entries::punch_in_time.ge(from))
            .filter(time_entries::punch_in_time.le(to))
            .order(time_entries::punch_in_time.desc())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entries.into_iter().map(Into::into).collect())
    }

    async fn get_paginated_logs(
        &self,
        workspace_id: Uuid,
        filter: TimeEntryLogFilter,
    ) -> Result<Vec<TimeEntryWithEmployee>, AppError> {
        // Dynamic SQL with Diesel boxed query
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut query = time_entries::table
            .inner_join(employees::table.on(time_entries::employee_id.eq(employees::id)))
            .filter(time_entries::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(emp_id) = filter.employee_id {
            query = query.filter(time_entries::employee_id.eq(emp_id));
        }
        if let Some(from_date) = filter.from {
            query = query.filter(time_entries::punch_in_time.ge(from_date));
        }
        if let Some(to_date) = filter.to {
            query = query.filter(time_entries::punch_in_time.le(to_date));
        }

        let results = query
            .select((
                time_entries::all_columns,
                employees::full_name,
                employees::employee_id_number,
            ))
            .order(time_entries::punch_in_time.desc())
            .limit(filter.limit)
            .offset(filter.offset)
            .load::<(TimeEntryEntity, String, String)>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results
            .into_iter()
            .map(|(entry, full_name, id_number)| {
                let domain: TimeEntry = entry.into();
                TimeEntryWithEmployee {
                    id: domain.id,
                    employee_id: domain.employee_id,
                    workspace_id: domain.workspace_id,
                    shift_id: domain.shift_id,
                    punch_in_time: domain.punch_in_time,
                    punch_out_time: domain.punch_out_time,
                    duration_minutes: domain.duration_minutes,
                    is_break_overtime: domain.is_break_overtime,
                    entry_type: domain.entry_type,
                    note: domain.note,
                    generation_metadata: domain.generation_metadata,
                    created_at: domain.created_at,
                    updated_at: domain.updated_at,
                    employee_full_name: full_name,
                    employee_id_number: id_number,
                }
            })
            .collect())
    }

    async fn count_logs(
        &self,
        workspace_id: Uuid,
        p_employee_id: Option<Uuid>,
        p_from: Option<DateTime<Utc>>,
        p_to: Option<DateTime<Utc>>,
    ) -> Result<i64, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut query = time_entries::table
            .inner_join(employees::table.on(time_entries::employee_id.eq(employees::id)))
            .filter(time_entries::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(emp_id) = p_employee_id {
            query = query.filter(time_entries::employee_id.eq(emp_id));
        }
        if let Some(from_date) = p_from {
            query = query.filter(time_entries::punch_in_time.ge(from_date));
        }
        if let Some(to_date) = p_to {
            query = query.filter(time_entries::punch_in_time.le(to_date));
        }

        let count: i64 = query
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(count)
    }

    async fn get_workspace_time_entries_batch(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<HashMap<Uuid, Vec<TimeEntry>>, AppError> {
        if employee_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let results: Vec<TimeEntryEntity> = time_entries::table
            .filter(time_entries::workspace_id.eq(workspace_id))
            .filter(time_entries::employee_id.eq_any(employee_ids))
            .filter(time_entries::punch_in_time.ge(from))
            .filter(time_entries::punch_in_time.le(to))
            .order((
                time_entries::employee_id.asc(),
                time_entries::punch_in_time.desc(),
            ))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut result_map: HashMap<Uuid, Vec<TimeEntry>> = HashMap::new();
        for entry in results {
            let domain: TimeEntry = entry.into();
            result_map
                .entry(domain.employee_id)
                .or_default()
                .push(domain);
        }

        Ok(result_map)
    }

    async fn replace_time_entries(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        entries: Vec<TimeEntry>,
    ) -> Result<(), AppError> {
        use diesel_async::AsyncConnection;

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                diesel::delete(
                    time_entries::table
                        .filter(time_entries::workspace_id.eq(workspace_id))
                        .filter(time_entries::employee_id.eq(employee_id))
                        .filter(time_entries::punch_in_time.ge(start_time))
                        .filter(time_entries::punch_in_time.lt(end_time)),
                )
                .execute(conn)
                .await?;

                if !entries.is_empty() {
                    let entities: Vec<TimeEntryEntity> = entries
                        .into_iter()
                        .map(|entry| TimeEntryEntity {
                            id: entry.id,
                            workspace_id: entry.workspace_id,
                            employee_id: entry.employee_id,
                            shift_id: entry.shift_id,
                            punch_in_time: entry.punch_in_time,
                            punch_out_time: entry.punch_out_time,
                            duration_minutes: entry.duration_minutes,
                            is_break_overtime: entry.is_break_overtime,
                            entry_type: entry.entry_type.clone(),
                            note: entry.note.clone(),
                            generation_metadata: entry.generation_metadata.clone(),
                            created_at: Some(entry.created_at),
                            updated_at: Some(entry.updated_at),
                        })
                        .collect();

                    diesel::insert_into(time_entries::table)
                        .values(&entities)
                        .execute(conn)
                        .await?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}
