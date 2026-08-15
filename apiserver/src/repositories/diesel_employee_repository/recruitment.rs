use crate::diesel_pool::DieselPoolsTenantTxExt;
use crate::errors::AppError;
use async_trait::async_trait;
use chrono::Datelike;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::dashboard_metrics::DashboardComparisonBasis;
use crate::models::diesel::employee::{
    AvgHeadcountRow, ClassificationRow, CohortRow, CountRow, DeptRow, EffectivenessHiresRow,
    EffectivenessRetentionRow, EffectivenessTrendRow, EffectivenessTurnoverRow, EmploymentTypeRow,
    LocationRow, ManagerRow, ReasonRow, RecruitmentSourceBreakdownRow, RecruitmentSourceEntity,
    TenureRow, TrendRow,
};
use crate::models::{
    CohortRetentionRow, RecruitmentEffectivenessAnalytics, RecruitmentEffectivenessOverview,
    RecruitmentEffectivenessQuery, RecruitmentEffectivenessSource, RecruitmentFunnelStage,
    RecruitmentSource, RecruitmentSourceBreakdown, RecruitmentTrendPoint, TurnoverAnalytics,
    TurnoverAnalyticsQuery, TurnoverByDepartment, TurnoverByEmploymentType, TurnoverByLocation,
    TurnoverByManager, TurnoverBySource, TurnoverClassification, TurnoverComparison, TurnoverRate,
    TurnoverReasonRow, TurnoverTenureBucket, TurnoverTrendPoint, TurnoverWarning,
};
use crate::repositories::employee::*;
use crate::schema::{employees, recruitment_sources};
use diesel::sql_types::{Array, Date, Integer, Text, Uuid as UuidType};

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
        query: &TurnoverAnalyticsQuery,
    ) -> Result<TurnoverAnalytics, AppError> {
        let workspace_id = auth_token.workspace_id();
        let start_date = query.start_date;
        let end_date = query.end_date;
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

                    // 5. Voluntary / involuntary / unknown classification
                    let class_rows: Vec<ClassificationRow> = diesel::sql_query(include_str!(
                        "../sql/recruitment/turnover_classification.sql"
                    ))
                    .bind::<UuidType, _>(workspace_id)
                    .bind::<Date, _>(start_date)
                    .bind::<Date, _>(end_date)
                    .load(conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    let classification = class_rows
                        .iter()
                        .map(|r| TurnoverClassification {
                            classification: r.classification.clone(),
                            count: r.count,
                        })
                        .collect::<Vec<_>>();
                    let voluntary_count = classification
                        .iter()
                        .find(|c| c.classification == "voluntary")
                        .map_or(0, |c| c.count);
                    let involuntary_count = classification
                        .iter()
                        .find(|c| c.classification == "involuntary")
                        .map_or(0, |c| c.count);
                    let unknown_classification_count = classification
                        .iter()
                        .find(|c| c.classification == "unknown")
                        .map_or(0, |c| c.count);

                    // 6. Termination reason breakdown
                    let reason_rows: Vec<ReasonRow> =
                        diesel::sql_query(include_str!("../sql/recruitment/turnover_reasons.sql"))
                            .bind::<UuidType, _>(workspace_id)
                            .bind::<Date, _>(start_date)
                            .bind::<Date, _>(end_date)
                            .load(conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    // 7. By employment type / location / manager
                    let emp_type_rows: Vec<EmploymentTypeRow> = diesel::sql_query(include_str!(
                        "../sql/recruitment/turnover_employment_type.sql"
                    ))
                    .bind::<UuidType, _>(workspace_id)
                    .bind::<Date, _>(start_date)
                    .bind::<Date, _>(end_date)
                    .load(conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    let location_rows: Vec<LocationRow> =
                        diesel::sql_query(include_str!("../sql/recruitment/turnover_location.sql"))
                            .bind::<UuidType, _>(workspace_id)
                            .bind::<Date, _>(start_date)
                            .bind::<Date, _>(end_date)
                            .load(conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    let manager_rows: Vec<ManagerRow> =
                        diesel::sql_query(include_str!("../sql/recruitment/turnover_manager.sql"))
                            .bind::<UuidType, _>(workspace_id)
                            .bind::<Date, _>(start_date)
                            .bind::<Date, _>(end_date)
                            .load(conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    // 8. Average headcount + headcount_start (retention denominator)
                    let avg_row: AvgHeadcountRow = diesel::sql_query(include_str!(
                        "../sql/recruitment/turnover_average_headcount.sql"
                    ))
                    .bind::<UuidType, _>(workspace_id)
                    .bind::<Date, _>(start_date)
                    .bind::<Date, _>(end_date)
                    .get_result::<AvgHeadcountRow>(conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    let average_headcount = avg_row.average_headcount;

                    // 9. Early attrition (tenure < 90 days)
                    let early: i64 = diesel::sql_query(include_str!(
                        "../sql/recruitment/turnover_early_attrition.sql"
                    ))
                    .bind::<UuidType, _>(workspace_id)
                    .bind::<Date, _>(start_date)
                    .bind::<Date, _>(end_date)
                    .get_result::<CountRow>(conn)
                    .await
                    .map(|r| r.count)
                    .map_err(AppError::from)?;

                    // 10. Monthly trend (period must span >= 2 months)
                    let period_days = end_date.signed_duration_since(start_date).num_days() + 1;
                    let monthly_trend = if period_days >= 60 {
                        let trend_rows: Vec<TrendRow> = diesel::sql_query(include_str!(
                            "../sql/recruitment/turnover_monthly_trend.sql"
                        ))
                        .bind::<UuidType, _>(workspace_id)
                        .bind::<Date, _>(start_date)
                        .bind::<Date, _>(end_date)
                        .load(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                        trend_rows
                            .into_iter()
                            .map(|r| TurnoverTrendPoint {
                                month: r.month,
                                headcount: r.headcount,
                                leavers: r.leavers,
                                turnover_rate: Some(TurnoverRate::percent(r.leavers, r.headcount)),
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };

                    // 11. Cohort retention matrix
                    let period_months = {
                        let start_first = chrono::NaiveDate::from_ymd_opt(
                            start_date.year(),
                            start_date.month(),
                            1,
                        )
                        .unwrap_or(start_date);
                        let end_first =
                            chrono::NaiveDate::from_ymd_opt(end_date.year(), end_date.month(), 1)
                                .unwrap_or(end_date);
                        ((end_first.year() - start_first.year()) * 12
                            + end_first.month().cast_signed()
                            - start_first.month().cast_signed()
                            + 1)
                        .max(1)
                    };
                    let cohort_rows: Vec<CohortRow> = diesel::sql_query(include_str!(
                        "../sql/recruitment/turnover_cohort_retention.sql"
                    ))
                    .bind::<UuidType, _>(workspace_id)
                    .bind::<Date, _>(start_date)
                    .bind::<Date, _>(end_date)
                    .bind::<Integer, _>(3) // MIN_COHORT
                    .bind::<Integer, _>(period_months)
                    .load(conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    let cohort_retention = cohort_rows
                        .into_iter()
                        .map(|r| CohortRetentionRow {
                            cohort_month: r.cohort_month,
                            months_since_hire: r.months_since_hire,
                            cohort_size: r.cohort_size,
                            active_count: r.active_count,
                            retention_rate: if r.cohort_size > 0 {
                                Some(
                                    rust_decimal::Decimal::from(r.active_count)
                                        / rust_decimal::Decimal::from(r.cohort_size)
                                        * rust_decimal::Decimal::from(100),
                                )
                            } else {
                                None
                            },
                        })
                        .collect();

                    // 12. Build the rate + comparison + warnings
                    let turnover_rate = TurnoverRate::percent(total, average_headcount);
                    let early_attrition = TurnoverRate::percent(early, average_headcount);

                    let retention_rate = if avg_row.headcount_start > 0 {
                        // Leavers active at period start are excluded from the retained set.
                        let retained = (avg_row.headcount_start - total).max(0);
                        Some(
                            rust_decimal::Decimal::from(retained)
                                / rust_decimal::Decimal::from(avg_row.headcount_start)
                                * rust_decimal::Decimal::from(100),
                        )
                    } else {
                        None
                    };

                    let mut warnings = Vec::new();
                    if average_headcount <= 0 {
                        warnings.push(TurnoverWarning {
                            code: "LOW_DENOMINATOR".to_string(),
                            message: "dashboard.hr.turnover_warning_low_denominator".to_string(),
                        });
                    }
                    warnings.push(TurnoverWarning {
                        code: "NO_DATA_SOURCE".to_string(),
                        message: "dashboard.hr.turnover_warning_regrettable".to_string(),
                    });

                    let comparison = match query.comparison_basis {
                        DashboardComparisonBasis::PreviousPeriod => {
                            let len = end_date.signed_duration_since(start_date).num_days();
                            (
                                start_date - chrono::Duration::days(len + 1),
                                start_date - chrono::Duration::days(1),
                            )
                        }
                        DashboardComparisonBasis::SamePeriodLastYear => (
                            start_date - chrono::Duration::days(365),
                            end_date - chrono::Duration::days(365),
                        ),
                        _ => (start_date, end_date),
                    };
                    let comparison = if query.comparison_basis
                        == DashboardComparisonBasis::PreviousPeriod
                        || query.comparison_basis == DashboardComparisonBasis::SamePeriodLastYear
                    {
                        let (c_start, c_end) = comparison;
                        let comp_total: i64 = diesel::sql_query(include_str!(
                            "../sql/recruitment/turnover_total.sql"
                        ))
                        .bind::<UuidType, _>(workspace_id)
                        .bind::<Date, _>(c_start)
                        .bind::<Date, _>(c_end)
                        .get_result::<CountRow>(conn)
                        .await
                        .map(|r| r.count)
                        .map_err(AppError::from)?;
                        let comp_avg: AvgHeadcountRow = diesel::sql_query(include_str!(
                            "../sql/recruitment/turnover_average_headcount.sql"
                        ))
                        .bind::<UuidType, _>(workspace_id)
                        .bind::<Date, _>(c_start)
                        .bind::<Date, _>(c_end)
                        .get_result::<AvgHeadcountRow>(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                        let comp_rate =
                            TurnoverRate::percent(comp_total, comp_avg.average_headcount);
                        let delta = match (turnover_rate.value, comp_rate.value) {
                            (Some(cur), Some(prev)) => Some(cur - prev),
                            _ => None,
                        };
                        Some(TurnoverComparison {
                            basis: query.comparison_basis.clone(),
                            period_start: c_start,
                            period_end: c_end,
                            rate: Some(comp_rate),
                            delta_percentage_points: delta,
                        })
                    } else {
                        None
                    };

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
                        turnover_rate,
                        average_headcount,
                        voluntary_count,
                        involuntary_count,
                        unknown_classification_count,
                        retention_rate,
                        early_attrition,
                        regrettable_attrition: None,
                        comparison,
                        target: query.target.clone(),
                        monthly_trend,
                        cohort_retention,
                        termination_reasons: reason_rows
                            .into_iter()
                            .map(|r| TurnoverReasonRow {
                                reason: r.reason,
                                count: r.count,
                            })
                            .collect(),
                        classification,
                        by_employment_type: emp_type_rows
                            .into_iter()
                            .map(|r| TurnoverByEmploymentType {
                                employment_type: r.employment_type,
                                count: r.count,
                            })
                            .collect(),
                        by_location: location_rows
                            .into_iter()
                            .map(|r| TurnoverByLocation {
                                location_name: r.location_name,
                                count: r.count,
                            })
                            .collect(),
                        by_manager: manager_rows
                            .into_iter()
                            .map(|r| TurnoverByManager {
                                manager_name: r.manager_name,
                                count: r.count,
                            })
                            .collect(),
                        warnings,
                    })
                })
            })
            .await
    }

    async fn get_recruitment_effectiveness(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        query: &RecruitmentEffectivenessQuery,
    ) -> Result<RecruitmentEffectivenessAnalytics, AppError> {
        let workspace_id = auth_token.workspace_id();
        let start_date = query.start_date;
        let end_date = query.end_date;

        // Empty filter vecs bind as NULL → guarded by `($x::... IS NULL OR ...)`.
        let department_ids = if query.department_ids.is_empty() {
            None
        } else {
            Some(query.department_ids.as_slice())
        };
        let location_ids = if query.location_ids.is_empty() {
            None
        } else {
            Some(query.location_ids.as_slice())
        };
        let job_roles = if query.job_roles.is_empty() {
            None
        } else {
            Some(query.job_roles.as_slice())
        };
        let source_types = if query.source_types.is_empty() {
            None
        } else {
            Some(query.source_types.as_slice())
        };
        let source_ids = if query.source_ids.is_empty() {
            None
        } else {
            Some(query.source_ids.as_slice())
        };

        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    // 1. Per-source hires + cost.
                    let hire_rows: Vec<EffectivenessHiresRow> = diesel::sql_query(include_str!(
                        "../sql/recruitment/recruitment_effectiveness_hires.sql"
                    ))
                    .bind::<UuidType, _>(workspace_id)
                    .bind::<Date, _>(start_date)
                    .bind::<Date, _>(end_date)
                    .bind::<NullableArrayUuid, _>(department_ids)
                    .bind::<NullableArrayUuid, _>(location_ids)
                    .bind::<NullableArrayText, _>(job_roles)
                    .bind::<NullableArrayText, _>(source_types)
                    .bind::<NullableArrayUuid, _>(source_ids)
                    .load(conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    let total_hires: i64 = hire_rows.iter().map(|r| r.hires).sum();

                    // 2. Per-source 90-day retention.
                    let retention_rows: Vec<EffectivenessRetentionRow> = diesel::sql_query(
                        include_str!("../sql/recruitment/recruitment_effectiveness_retention.sql"),
                    )
                    .bind::<UuidType, _>(workspace_id)
                    .bind::<Date, _>(start_date)
                    .bind::<Date, _>(end_date)
                    .bind::<NullableArrayUuid, _>(department_ids)
                    .bind::<NullableArrayUuid, _>(location_ids)
                    .bind::<NullableArrayText, _>(job_roles)
                    .bind::<NullableArrayText, _>(source_types)
                    .bind::<NullableArrayUuid, _>(source_ids)
                    .load(conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    // 3. Per-source terminations in range.
                    let turnover_rows: Vec<EffectivenessTurnoverRow> = diesel::sql_query(
                        include_str!("../sql/recruitment/recruitment_effectiveness_turnover.sql"),
                    )
                    .bind::<UuidType, _>(workspace_id)
                    .bind::<Date, _>(start_date)
                    .bind::<Date, _>(end_date)
                    .bind::<NullableArrayUuid, _>(department_ids)
                    .bind::<NullableArrayUuid, _>(location_ids)
                    .bind::<NullableArrayText, _>(job_roles)
                    .bind::<NullableArrayText, _>(source_types)
                    .bind::<NullableArrayUuid, _>(source_ids)
                    .load(conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    // 4. Monthly hires trend (period must span >= 2 months).
                    let period_days = end_date.signed_duration_since(start_date).num_days() + 1;
                    let hires_trend = if period_days >= 60 {
                        let trend_rows: Vec<EffectivenessTrendRow> = diesel::sql_query(
                            include_str!("../sql/recruitment/recruitment_effectiveness_trend.sql"),
                        )
                        .bind::<UuidType, _>(workspace_id)
                        .bind::<Date, _>(start_date)
                        .bind::<Date, _>(end_date)
                        .bind::<NullableArrayUuid, _>(department_ids)
                        .bind::<NullableArrayUuid, _>(location_ids)
                        .bind::<NullableArrayText, _>(job_roles)
                        .bind::<NullableArrayText, _>(source_types)
                        .bind::<NullableArrayUuid, _>(source_ids)
                        .load(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                        trend_rows
                            .into_iter()
                            .map(|r| RecruitmentTrendPoint {
                                month: r.month,
                                hires: r.hires,
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };

                    // 5. Aggregate per-source rows.
                    let retained_total: i64 = retention_rows.iter().map(|r| r.retained_90d).sum();
                    let turnover_in_range: i64 = turnover_rows.iter().map(|r| r.count).sum();
                    let cost_total: rust_decimal::Decimal = hire_rows
                        .iter()
                        .filter(|r| r.cost_available)
                        .filter_map(|r| r.total_cost)
                        .sum();
                    let cost_available =
                        hire_rows.iter().any(|r| r.cost_available) && total_hires > 0;

                    let by_source = hire_rows
                        .into_iter()
                        .map(|r| {
                            let retention = retention_rows
                                .iter()
                                .find(|ret| ret.source_id == r.source_id)
                                .map_or(0, |ret| ret.retained_90d);
                            let turnover = turnover_rows
                                .iter()
                                .find(|tr| tr.source_id == r.source_id)
                                .map_or(0, |tr| tr.count);
                            let cost_per_hire = if r.cost_available && r.hires > 0 {
                                r.total_cost
                                    .map(|c| c / rust_decimal::Decimal::from(r.hires))
                            } else {
                                None
                            };
                            RecruitmentEffectivenessSource {
                                source_id: r.source_id,
                                source_type: r.source_type,
                                source_name: r.source_name,
                                hires: r.hires,
                                hires_ratio: if total_hires > 0 {
                                    Some(
                                        rust_decimal::Decimal::from(r.hires)
                                            / rust_decimal::Decimal::from(total_hires)
                                            * rust_decimal::Decimal::from(100),
                                    )
                                } else {
                                    None
                                },
                                retained_90d: retention,
                                retention_90d: TurnoverRate::percent(retention, r.hires).value,
                                cost_per_hire,
                                cost_per_hire_available: r.cost_available,
                                turnover_in_range: turnover,
                            }
                        })
                        .collect::<Vec<_>>();

                    // 6. Comparison window — hires in the previous period.
                    let comparison_hires = match query.comparison_basis {
                        DashboardComparisonBasis::PreviousPeriod => {
                            let len = end_date.signed_duration_since(start_date).num_days();
                            Some((
                                start_date - chrono::Duration::days(len + 1),
                                start_date - chrono::Duration::days(1),
                            ))
                        }
                        DashboardComparisonBasis::SamePeriodLastYear => Some((
                            start_date - chrono::Duration::days(365),
                            end_date - chrono::Duration::days(365),
                        )),
                        _ => None,
                    };
                    let comparison_hires = if let Some((c_start, c_end)) = comparison_hires {
                        let comp_rows: Vec<EffectivenessHiresRow> = diesel::sql_query(
                            include_str!("../sql/recruitment/recruitment_effectiveness_hires.sql"),
                        )
                        .bind::<UuidType, _>(workspace_id)
                        .bind::<Date, _>(c_start)
                        .bind::<Date, _>(c_end)
                        .bind::<NullableArrayUuid, _>(department_ids)
                        .bind::<NullableArrayUuid, _>(location_ids)
                        .bind::<NullableArrayText, _>(job_roles)
                        .bind::<NullableArrayText, _>(source_types)
                        .bind::<NullableArrayUuid, _>(source_ids)
                        .load(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                        Some(comp_rows.iter().map(|r| r.hires).sum())
                    } else {
                        None
                    };

                    // 7. Honest funnel.
                    let funnel = vec![
                        RecruitmentFunnelStage {
                            stage: "applicants".into(),
                            count: None,
                            data_source: None,
                            conversion_rate: None,
                        },
                        RecruitmentFunnelStage {
                            stage: "screening".into(),
                            count: None,
                            data_source: None,
                            conversion_rate: None,
                        },
                        RecruitmentFunnelStage {
                            stage: "interview".into(),
                            count: None,
                            data_source: None,
                            conversion_rate: None,
                        },
                        RecruitmentFunnelStage {
                            stage: "offer".into(),
                            count: None,
                            data_source: None,
                            conversion_rate: None,
                        },
                        RecruitmentFunnelStage {
                            stage: "hired".into(),
                            count: Some(total_hires),
                            data_source: Some("hires".into()),
                            conversion_rate: None,
                        },
                        RecruitmentFunnelStage {
                            stage: "retained_90d".into(),
                            count: Some(retained_total),
                            data_source: Some("hires".into()),
                            conversion_rate: if total_hires > 0 {
                                Some(
                                    rust_decimal::Decimal::from(retained_total)
                                        / rust_decimal::Decimal::from(total_hires)
                                        * rust_decimal::Decimal::from(100),
                                )
                            } else {
                                None
                            },
                        },
                    ];

                    // 8. Data gaps + warnings.
                    let data_gaps = RecruitmentEffectivenessAnalytics::data_gaps();
                    let mut warnings = Vec::new();
                    if total_hires <= 0 {
                        warnings.push(TurnoverWarning {
                            code: "LOW_DENOMINATOR".to_string(),
                            message: "dashboard.hr.effective_warning_low_sample".to_string(),
                        });
                    }
                    warnings.push(TurnoverWarning {
                        code: "NO_DATA_SOURCE".to_string(),
                        message: "dashboard.hr.effective_warning_no_data_source".to_string(),
                    });

                    Ok(RecruitmentEffectivenessAnalytics {
                        period_start: start_date.to_string(),
                        period_end: end_date.to_string(),
                        total_hires,
                        overview: RecruitmentEffectivenessOverview {
                            hires: total_hires,
                            average_cost_per_hire: if cost_available && total_hires > 0 {
                                Some(cost_total / rust_decimal::Decimal::from(total_hires))
                            } else {
                                None
                            },
                            cost_available,
                            retained_90d: retained_total,
                            retention_90d: TurnoverRate::percent(retained_total, total_hires).value,
                            turnover_in_range,
                            comparison_hires,
                            comparison_hires_delta: comparison_hires.map(|prev| total_hires - prev),
                        },
                        funnel,
                        by_source,
                        hires_trend,
                        data_gaps,
                        warnings,
                    })
                })
            })
            .await
    }
}

/// `diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>` — the
/// binding type for the `$N::uuid[] IS NULL OR ... = ANY($N)` filter guards.
type NullableArrayUuid = diesel::sql_types::Nullable<Array<UuidType>>;
/// `diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Text>>` — binding
/// type for the job-role / source-type string filters.
type NullableArrayText = diesel::sql_types::Nullable<Array<Text>>;
