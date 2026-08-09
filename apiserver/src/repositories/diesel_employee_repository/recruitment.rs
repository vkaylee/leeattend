use crate::diesel_pool::DieselPoolsTenantTxExt;
use crate::errors::AppError;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::diesel::employee::{
    CountRow, DeptRow, RecruitmentSourceBreakdownRow, RecruitmentSourceEntity, TenureRow,
};
use crate::models::{
    RecruitmentSource, RecruitmentSourceBreakdown, TurnoverAnalytics, TurnoverByDepartment,
    TurnoverBySource, TurnoverTenureBucket,
};
use crate::repositories::employee::*;
use crate::schema::{employees, recruitment_sources};
use diesel::sql_types::{Date, Uuid as UuidType};

use super::DieselEmployeeRepository;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl EmployeeRecruitmentRepository for DieselEmployeeRepository {
    async fn set_employee_recruitment_source(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    diesel::update(employees::table)
                        .filter(employees::id.eq(employee_id))
                        .filter(employees::workspace_id.eq(workspace_id))
                        .set((
                            employees::recruitment_source_id.eq(recruitment_source_id),
                            employees::updated_at.eq(Some(chrono::Utc::now())),
                        ))
                        .execute(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(())
                })
            })
            .await
    }

    async fn create_recruitment_source(&self, source: &RecruitmentSource) -> Result<(), AppError> {
        let auth_token =
            crate::diesel_pool::TenantAuthToken::system_override(source.workspace_id());
        let entity: RecruitmentSourceEntity = source.clone().into();

        self.diesel_pools
            .with_tenant_tx(&auth_token, |conn| {
                Box::pin(async move {
                    diesel::insert_into(recruitment_sources::table)
                        .values(&entity)
                        .execute(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(())
                })
            })
            .await
    }

    async fn get_recruitment_sources(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<RecruitmentSource>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let total: i64 = recruitment_sources::table
                        .filter(recruitment_sources::workspace_id.eq(workspace_id))
                        .count()
                        .get_result(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

                    let results: Vec<RecruitmentSourceEntity> = recruitment_sources::table
                        .filter(recruitment_sources::workspace_id.eq(workspace_id))
                        .order(recruitment_sources::created_at.desc())
                        .limit(limit)
                        .offset((page - 1) * limit)
                        .load(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    Ok((results.into_iter().map(Into::into).collect(), total))
                })
            })
            .await
    }

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let result: Option<RecruitmentSourceEntity> = recruitment_sources::table
                        .filter(recruitment_sources::id.eq(id))
                        .filter(recruitment_sources::workspace_id.eq(workspace_id))
                        .first(conn)
                        .await
                        .optional()
                        .map_err(AppError::from)?;

                    Ok(result.map(Into::into))
                })
            })
            .await
    }

    async fn update_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let result: Option<RecruitmentSourceEntity> =
                        diesel::update(recruitment_sources::table)
                            .filter(recruitment_sources::id.eq(id))
                            .filter(recruitment_sources::workspace_id.eq(workspace_id))
                            .set((
                                req.name.as_ref().map(|n| recruitment_sources::name.eq(n)),
                                req.referrer_employee_id
                                    .map(|id| recruitment_sources::referrer_employee_id.eq(id)),
                                req.referral_bonus_amount
                                    .map(|a| recruitment_sources::referral_bonus_amount.eq(a)),
                                req.referral_bonus_paid
                                    .map(|p| recruitment_sources::referral_bonus_paid.eq(p)),
                                req.agency_contact_name
                                    .as_ref()
                                    .map(|n| recruitment_sources::agency_contact_name.eq(n)),
                                req.agency_contact_email
                                    .as_ref()
                                    .map(|e| recruitment_sources::agency_contact_email.eq(e)),
                                req.agency_fee_amount
                                    .map(|a| recruitment_sources::agency_fee_amount.eq(a)),
                                req.agency_fee_paid
                                    .map(|p| recruitment_sources::agency_fee_paid.eq(p)),
                                req.notes.as_ref().map(|n| recruitment_sources::notes.eq(n)),
                                recruitment_sources::updated_at.eq(chrono::Utc::now()),
                            ))
                            .get_result(conn)
                            .await
                            .optional()
                            .map_err(AppError::from)?;

                    Ok(result.map(Into::into))
                })
            })
            .await
    }

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let affected = diesel::delete(recruitment_sources::table)
                        .filter(recruitment_sources::id.eq(id))
                        .filter(recruitment_sources::workspace_id.eq(workspace_id))
                        .execute(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(affected as u64)
                })
            })
            .await
    }

    async fn get_recruitment_source_breakdown(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let rows: Vec<RecruitmentSourceBreakdownRow> = diesel::sql_query(include_str!(
                        "../sql/recruitment/recruitment_source_breakdown.sql"
                    ))
                    .bind::<UuidType, _>(workspace_id)
                    .load(conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    Ok(rows
                        .into_iter()
                        .map(|r| RecruitmentSourceBreakdown {
                            source_type: r.source_type,
                            source_name: r.source_name,
                            count: r.count,
                        })
                        .collect())
                })
            })
            .await
    }

    async fn get_turnover_analytics(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    // 1. Total terminated count
                    let total: i64 =
                        diesel::sql_query(include_str!("../sql/recruitment/turnover_total.sql"))
                            .bind::<UuidType, _>(workspace_id)
                            .bind::<Date, _>(start_date)
                            .bind::<Date, _>(end_date)
                            .get_result::<CountRow>(conn)
                            .await
                            .map(|r| r.count)
                            .map_err(AppError::from)?;

                    // 2. Tenure distribution
                    let tenure_rows: Vec<TenureRow> =
                        diesel::sql_query(include_str!("../sql/recruitment/turnover_tenure.sql"))
                            .bind::<UuidType, _>(workspace_id)
                            .bind::<Date, _>(start_date)
                            .bind::<Date, _>(end_date)
                            .load(conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    // 3. By recruitment source
                    let source_rows: Vec<RecruitmentSourceBreakdownRow> =
                        diesel::sql_query(include_str!("../sql/recruitment/turnover_source.sql"))
                            .bind::<UuidType, _>(workspace_id)
                            .bind::<Date, _>(start_date)
                            .bind::<Date, _>(end_date)
                            .load(conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    // 4. By department (latest department assignment before termination)
                    let dept_rows: Vec<DeptRow> = diesel::sql_query(include_str!(
                        "../sql/recruitment/turnover_department.sql"
                    ))
                    .bind::<UuidType, _>(workspace_id)
                    .bind::<Date, _>(start_date)
                    .bind::<Date, _>(end_date)
                    .load(conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    Ok(TurnoverAnalytics {
                        tenure_distribution: tenure_rows
                            .into_iter()
                            .map(|r| TurnoverTenureBucket {
                                bucket: r.bucket,
                                count: r.count,
                            })
                            .collect(),
                        by_source: source_rows
                            .into_iter()
                            .map(|r| TurnoverBySource {
                                source_type: r.source_type,
                                source_name: r.source_name,
                                count: r.count,
                            })
                            .collect(),
                        by_department: dept_rows
                            .into_iter()
                            .map(|r| TurnoverByDepartment {
                                department_name: r.department_name,
                                count: r.count,
                            })
                            .collect(),
                        total_terminated: total,
                        period_start: start_date.to_string(),
                        period_end: end_date.to_string(),
                    })
                })
            })
            .await
    }
}
