use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::working_pattern::{
    EmployeeWorkingPatternEntity, WorkingPatternDetailEntity, WorkingPatternEntity,
};
use crate::models::{
    BulkAssignResponse, EmployeeWorkingPattern, WorkingPattern, WorkingPatternDetail,
};
use crate::schema::{employee_working_patterns, working_pattern_details, working_patterns};

pub struct DieselWorkingPatternRepository {
    diesel_pools: DieselPools,
}

impl DieselWorkingPatternRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

macro_rules! map_db_err {
    ($e:expr) => {
        $e.map_err(|e| AppError::Internal(format!("db: {e}")))
    };
}

use crate::repositories::working_pattern_repository::WorkingPatternRepositoryTrait;
use async_trait::async_trait;

#[async_trait]
impl WorkingPatternRepositoryTrait for DieselWorkingPatternRepository {
    async fn create_pattern_with_details(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
        pattern_type: &str,
        cycle_length_days: Option<i32>,
        details: Vec<(i32, Option<Uuid>)>,
    ) -> Result<WorkingPattern, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let name_owned = name.to_string();
        let description_owned = description.map(ToString::to_string);
        let pattern_type_owned = pattern_type.to_string();

        let result = conn
            .transaction(|conn| {
                Box::pin(async move {
                    let pattern_entity = diesel::insert_into(working_patterns::table)
                        .values((
                            working_patterns::workspace_id.eq(workspace_id),
                            working_patterns::name.eq(&name_owned),
                            working_patterns::description.eq(&description_owned),
                            working_patterns::pattern_type.eq(&pattern_type_owned),
                            working_patterns::cycle_length_days.eq(cycle_length_days),
                        ))
                        .returning(WorkingPatternEntity::as_returning())
                        .get_result::<WorkingPatternEntity>(&mut *conn)
                        .await?;

                    let detail_records: Vec<_> = details
                        .into_iter()
                        .map(|(schedule_day, shift_id)| {
                            (
                                working_pattern_details::pattern_id.eq(pattern_entity.id),
                                working_pattern_details::schedule_day.eq(schedule_day),
                                working_pattern_details::shift_id.eq(shift_id),
                            )
                        })
                        .collect();

                    if !detail_records.is_empty() {
                        diesel::insert_into(working_pattern_details::table)
                            .values(&detail_records)
                            .execute(&mut *conn)
                            .await?;
                    }

                    Ok::<_, diesel::result::Error>(pattern_entity)
                })
            })
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to create pattern with details: {e}"))
            })?;

        Ok(result.into())
    }

    #[allow(clippy::too_many_arguments)]
    async fn update_pattern_with_details(
        &self,
        workspace_id: Uuid,
        pattern_id: Uuid,
        name: &str,
        description: Option<&str>,
        pattern_type: &str,
        cycle_length_days: Option<i32>,
        details: Vec<(i32, Option<Uuid>)>,
    ) -> Result<WorkingPattern, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let name_owned = name.to_string();
        let description_owned = description.map(ToString::to_string);
        let pattern_type_owned = pattern_type.to_string();

        let result = conn
            .transaction(|conn| {
                Box::pin(async move {
                    let pattern_entity = diesel::update(
                        working_patterns::table
                            .filter(working_patterns::id.eq(pattern_id))
                            .filter(working_patterns::workspace_id.eq(workspace_id)),
                    )
                    .set((
                        working_patterns::name.eq(&name_owned),
                        working_patterns::description.eq(&description_owned),
                        working_patterns::pattern_type.eq(&pattern_type_owned),
                        working_patterns::cycle_length_days.eq(cycle_length_days),
                    ))
                    .returning(WorkingPatternEntity::as_returning())
                    .get_result::<WorkingPatternEntity>(&mut *conn)
                    .await?;

                    diesel::delete(
                        working_pattern_details::table
                            .filter(working_pattern_details::pattern_id.eq(pattern_id)),
                    )
                    .execute(&mut *conn)
                    .await?;

                    let detail_records: Vec<_> = details
                        .into_iter()
                        .map(|(schedule_day, shift_id)| {
                            (
                                working_pattern_details::pattern_id.eq(pattern_id),
                                working_pattern_details::schedule_day.eq(schedule_day),
                                working_pattern_details::shift_id.eq(shift_id),
                            )
                        })
                        .collect();

                    if !detail_records.is_empty() {
                        diesel::insert_into(working_pattern_details::table)
                            .values(&detail_records)
                            .execute(&mut *conn)
                            .await?;
                    }

                    Ok::<_, diesel::result::Error>(pattern_entity)
                })
            })
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to update pattern with details: {e}"))
            })?;

        Ok(result.into())
    }

    async fn get_patterns(&self, workspace_id: Uuid) -> Result<Vec<WorkingPattern>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        map_db_err!(
            working_patterns::table
                .filter(working_patterns::workspace_id.eq(workspace_id))
                .filter(working_patterns::is_active.eq(true))
                .filter(working_patterns::deleted_at.is_null())
                .order(working_patterns::name.asc())
                .select(WorkingPatternEntity::as_select())
                .load(&mut *conn)
                .await
        )
        .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    async fn get_pattern_details(
        &self,
        workspace_id: Uuid,
        pattern_id: Uuid,
    ) -> Result<Vec<WorkingPatternDetail>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        map_db_err!(
            working_pattern_details::table
                .filter(working_pattern_details::pattern_id.eq(pattern_id))
                .order(working_pattern_details::schedule_day.asc())
                .select(WorkingPatternDetailEntity::as_select())
                .load(&mut *conn)
                .await
        )
        .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    async fn assign_pattern(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        pattern_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        anchor_date: Option<NaiveDate>,
    ) -> Result<EmployeeWorkingPattern, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Check if employment exists and belongs to the workspace of this tenant
        let employment_exists = crate::schema::employments::table
            .filter(crate::schema::employments::id.eq(employment_id))
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .filter(crate::schema::employments::deleted_at.is_null())
            .select(crate::schema::employments::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

        if employment_exists.is_none() {
            return Err(AppError::NotFound("error.employment_not_found".to_string()));
        }

        map_db_err!(
            diesel::insert_into(employee_working_patterns::table)
                .values((
                    employee_working_patterns::workspace_id.eq(workspace_id),
                    employee_working_patterns::employment_id.eq(employment_id),
                    employee_working_patterns::pattern_id.eq(pattern_id),
                    employee_working_patterns::start_date.eq(start_date),
                    employee_working_patterns::end_date.eq(end_date),
                    employee_working_patterns::anchor_date.eq(anchor_date),
                ))
                .returning(EmployeeWorkingPatternEntity::as_returning())
                .get_result(&mut *conn)
                .await
        )
        .map(Into::into)
    }

    async fn get_employment_pattern_history(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
    ) -> Result<Vec<EmployeeWorkingPattern>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        map_db_err!(
            employee_working_patterns::table
                .filter(employee_working_patterns::workspace_id.eq(workspace_id))
                .filter(employee_working_patterns::employment_id.eq(employment_id))
                .order(employee_working_patterns::start_date.desc())
                .select(EmployeeWorkingPatternEntity::as_select())
                .load(&mut *conn)
                .await
        )
        .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    async fn bulk_assign_pattern(
        &self,
        workspace_id: Uuid,
        employment_ids: &[Uuid],
        pattern_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        anchor_date: Option<NaiveDate>,
    ) -> Result<BulkAssignResponse, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Verify all employment_ids belong to this workspace
        let valid_count = crate::schema::employments::table
            .filter(crate::schema::employments::id.eq_any(employment_ids))
            .filter(crate::schema::employments::workspace_id.eq(workspace_id))
            .filter(crate::schema::employments::deleted_at.is_null())
            .count()
            .get_result::<i64>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db check employments: {e}")))?;

        if valid_count != employment_ids.len() as i64 {
            return Err(AppError::NotFound("error.employment_not_found".to_string()));
        }

        let conflicts: Vec<Uuid> = employee_working_patterns::table
            .filter(employee_working_patterns::workspace_id.eq(workspace_id))
            .filter(employee_working_patterns::start_date.eq(start_date))
            .filter(employee_working_patterns::employment_id.eq_any(employment_ids))
            .select(employee_working_patterns::employment_id)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if !conflicts.is_empty() {
            return Ok(BulkAssignResponse {
                success_count: 0,
                failed_employees: conflicts,
            });
        }

        let assignment_records: Vec<_> = employment_ids
            .iter()
            .map(|&employment_id| {
                (
                    employee_working_patterns::workspace_id.eq(workspace_id),
                    employee_working_patterns::employment_id.eq(employment_id),
                    employee_working_patterns::pattern_id.eq(pattern_id),
                    employee_working_patterns::start_date.eq(start_date),
                    employee_working_patterns::end_date.eq(end_date),
                    employee_working_patterns::anchor_date.eq(anchor_date),
                )
            })
            .collect();

        if !assignment_records.is_empty() {
            diesel::insert_into(employee_working_patterns::table)
                .values(&assignment_records)
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;
        }

        Ok(BulkAssignResponse {
            success_count: employment_ids.len(),
            failed_employees: vec![],
        })
    }

    async fn delete_pattern(&self, workspace_id: Uuid, pattern_id: Uuid) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result = conn
            .transaction(|conn| {
                Box::pin(async move {
                    diesel::update(
                        working_patterns::table
                            .filter(working_patterns::id.eq(pattern_id))
                            .filter(working_patterns::workspace_id.eq(workspace_id)),
                    )
                    .set(working_patterns::deleted_at.eq(chrono::Utc::now()))
                    .execute(&mut *conn)
                    .await?;

                    Ok::<_, diesel::result::Error>(())
                })
            })
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete pattern: {e}")))?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselWorkingPatternRepository::new(pools);
        // DI connection test passes if it compiles
    }

    #[tokio::test]
    async fn test_repo_methods_error_handling() {
        let bad_url = "postgres://invalid:invalid@localhost:12345/invalid_db";
        let pools = DieselPools::for_test(bad_url);
        let repo = DieselWorkingPatternRepository::new(pools);
        let ws_id = Uuid::new_v4();

        let err = repo
            .create_pattern_with_details(ws_id, "Name", None, "weekly", None, vec![])
            .await;
        assert!(err.is_err());

        let err = repo
            .update_pattern_with_details(
                ws_id,
                Uuid::new_v4(),
                "Name",
                None,
                "weekly",
                None,
                vec![],
            )
            .await;
        assert!(err.is_err());

        let err = repo.get_patterns(ws_id).await;
        assert!(err.is_err());

        let err = repo.get_pattern_details(ws_id, Uuid::new_v4()).await;
        assert!(err.is_err());

        let err = repo
            .assign_pattern(
                ws_id,
                Uuid::new_v4(),
                Uuid::new_v4(),
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                None,
                None,
            )
            .await;
        assert!(err.is_err());

        let err = repo
            .get_employment_pattern_history(ws_id, Uuid::new_v4())
            .await;
        assert!(err.is_err());

        let err = repo
            .bulk_assign_pattern(
                ws_id,
                &[Uuid::new_v4()],
                Uuid::new_v4(),
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                None,
                None,
            )
            .await;
        assert!(err.is_err());

        let err = repo.delete_pattern(ws_id, Uuid::new_v4()).await;
        assert!(err.is_err());
    }
}
