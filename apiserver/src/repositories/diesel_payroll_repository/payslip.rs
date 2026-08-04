use super::*;
use crate::errors::AppError;
use crate::models::diesel::payroll::*;
use crate::models::{IncentiveEvaluationResult, Payslip, PayslipSegment};
use chrono::{NaiveDate, Utc};
use diesel_async::{AsyncConnection, RunQueryDsl};
use rust_decimal::prelude::*;
use uuid::Uuid;

#[allow(clippy::too_many_lines)]
impl DieselPayrollRepository {
    pub(super) async fn create_payslip(&self, payslip: &Payslip) -> Result<Payslip, AppError> {
        use std::str::FromStr;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&payslip.workspace_id())
                .await?,
        );

        let entity = PayslipEntity {
            id: payslip.id(),
            employment_id: payslip.employment_id(),
            period_start: payslip.period_start(),
            period_end: payslip.period_end(),
            base_pay_amount: bigdecimal::BigDecimal::from_str(
                &payslip.base_pay_amount().to_string(),
            )
            .unwrap_or_default(),
            overtime_pay_amount: bigdecimal::BigDecimal::from_str(
                &payslip.overtime_pay_amount().to_string(),
            )
            .unwrap_or_default(),
            deductions_amount: bigdecimal::BigDecimal::from_str(
                &payslip.deductions_amount().to_string(),
            )
            .unwrap_or_default(),
            net_pay_amount: bigdecimal::BigDecimal::from_str(&payslip.net_pay_amount().to_string())
                .unwrap_or_default(),
            status: payslip.status().to_string(),
            created_at: Some(payslip.created_at()),
            updated_at: Some(payslip.updated_at()),
            standard_units: payslip.standard_units(),
            overtime_units: payslip.overtime_units(),
            holiday_work_premium_amount: bigdecimal::BigDecimal::from_str(
                &payslip.holiday_work_premium_amount().to_string(),
            )
            .unwrap_or_default(),
            workspace_id: payslip.workspace_id(),
        };

        let result = diesel::insert_into(payslips::table)
            .values(&entity)
            .get_result::<PayslipEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(result.into())
    }

    pub(super) async fn get_payslips_by_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<Payslip>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payslips::table
            .inner_join(employments::table)
            .filter(payslips::workspace_id.eq(workspace_id))
            .filter(employments::employee_id.eq(employee_id))
            .select(PayslipEntity::as_select())
            .order(payslips::period_start.desc())
            .load::<PayslipEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn get_payslips_by_workspace_period(
        &self,
        workspace_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<Payslip>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payslips::table
            .filter(payslips::workspace_id.eq(workspace_id))
            .filter(payslips::period_start.eq(period_start))
            .filter(payslips::period_end.eq(period_end))
            .order(payslips::created_at.desc())
            .load::<PayslipEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn create_payslip_segment(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
        args: &crate::models::CreatePayslipSegmentArgs,
    ) -> Result<(), AppError> {
        use std::str::FromStr;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entity = PayslipSegmentEntity {
            id: Uuid::new_v4(),
            payslip_id,
            start_date: args.start_date,
            end_date: args.end_date,
            department_id: args.department_id,
            applied_base_salary: bigdecimal::BigDecimal::from_str(
                &args.applied_base_salary.to_string(),
            )
            .unwrap_or_default(),
            applied_std_days: bigdecimal::BigDecimal::from_str(&args.applied_std_days.to_string())
                .unwrap_or_default(),
            actual_work_days: bigdecimal::BigDecimal::from_str(&args.actual_work_days.to_string())
                .unwrap_or_default(),
            paid_work_days: bigdecimal::BigDecimal::from_str(&args.paid_work_days.to_string())
                .unwrap_or_default(),
            amount: bigdecimal::BigDecimal::from_str(&args.amount.to_string()).unwrap_or_default(),
            created_at: Some(Utc::now()),
            applied_hours_per_day: bigdecimal::BigDecimal::from_str(
                &args.hours_per_day.to_string(),
            )
            .unwrap_or_default(),
            std_days_source: args.std_days_source.clone(),
        };

        diesel::insert_into(payslip_segments::table)
            .values(&entity)
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn get_payslip_segments(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
    ) -> Result<Vec<PayslipSegment>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payslip_segments::table
            .filter(payslip_segments::payslip_id.eq(payslip_id))
            .load::<PayslipSegmentEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn persist_calculated_payslip_data(
        &self,
        workspace_id: Uuid,
        payslip: &Payslip,
        segments: &[crate::models::CreatePayslipSegmentArgs],
        incentives: &[IncentiveEvaluationResult],
        rule_evaluations: &[(String, Decimal)],
    ) -> Result<Payslip, AppError> {
        use std::str::FromStr;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let payslip_entity = PayslipEntity {
            id: payslip.id(),
            employment_id: payslip.employment_id(),
            period_start: payslip.period_start(),
            period_end: payslip.period_end(),
            base_pay_amount: bigdecimal::BigDecimal::from_str(
                &payslip.base_pay_amount().to_string(),
            )
            .unwrap_or_default(),
            overtime_pay_amount: bigdecimal::BigDecimal::from_str(
                &payslip.overtime_pay_amount().to_string(),
            )
            .unwrap_or_default(),
            deductions_amount: bigdecimal::BigDecimal::from_str(
                &payslip.deductions_amount().to_string(),
            )
            .unwrap_or_default(),
            net_pay_amount: bigdecimal::BigDecimal::from_str(&payslip.net_pay_amount().to_string())
                .unwrap_or_default(),
            status: payslip.status().to_string(),
            created_at: Some(payslip.created_at()),
            updated_at: Some(payslip.updated_at()),
            standard_units: payslip.standard_units(),
            overtime_units: payslip.overtime_units(),
            holiday_work_premium_amount: bigdecimal::BigDecimal::from_str(
                &payslip.holiday_work_premium_amount().to_string(),
            )
            .unwrap_or_default(),
            workspace_id: payslip.workspace_id(),
        };

        let segments_entities: Vec<PayslipSegmentEntity> = segments
            .iter()
            .map(|args| PayslipSegmentEntity {
                id: Uuid::new_v4(),
                payslip_id: payslip.id(),
                start_date: args.start_date,
                end_date: args.end_date,
                department_id: args.department_id,
                applied_base_salary: bigdecimal::BigDecimal::from_str(
                    &args.applied_base_salary.to_string(),
                )
                .unwrap_or_default(),
                applied_std_days: bigdecimal::BigDecimal::from_str(
                    &args.applied_std_days.to_string(),
                )
                .unwrap_or_default(),
                actual_work_days: bigdecimal::BigDecimal::from_str(
                    &args.actual_work_days.to_string(),
                )
                .unwrap_or_default(),
                paid_work_days: bigdecimal::BigDecimal::from_str(&args.paid_work_days.to_string())
                    .unwrap_or_default(),
                amount: bigdecimal::BigDecimal::from_str(&args.amount.to_string())
                    .unwrap_or_default(),
                created_at: Some(Utc::now()),
                applied_hours_per_day: bigdecimal::BigDecimal::from_str(
                    &args.hours_per_day.to_string(),
                )
                .unwrap_or_default(),
                std_days_source: args.std_days_source.clone(),
            })
            .collect();

        let incentives_entities: Vec<PayslipIncentiveEntity> = incentives
            .iter()
            .map(|result| PayslipIncentiveEntity {
                id: Uuid::new_v4(),
                payslip_id: payslip.id(),
                policy_name: result.policy_name.clone(),
                amount: result.amount.to_f64().unwrap_or_default(),
                tier_id: result.tier_id,
                standard_amount: result.standard_amount.to_f64().unwrap_or_default(),
                created_at: Utc::now(),
            })
            .collect();

        let rules_entities: Vec<PayslipRuleEvaluationEntity> = rule_evaluations
            .iter()
            .map(|(rule_code, amount)| PayslipRuleEvaluationEntity {
                id: Uuid::new_v4(),
                payslip_id: payslip.id(),
                rule_code: rule_code.clone(),
                amount: bigdecimal::BigDecimal::from_str(&amount.to_string()).unwrap_or_default(),
                created_at: Utc::now(),
            })
            .collect();

        let result = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    let created_payslip = diesel::insert_into(payslips::table)
                        .values(&payslip_entity)
                        .get_result::<PayslipEntity>(&mut *conn)
                        .await?;

                    if !segments_entities.is_empty() {
                        diesel::insert_into(payslip_segments::table)
                            .values(&segments_entities)
                            .execute(&mut *conn)
                            .await?;
                    }

                    if !incentives_entities.is_empty() {
                        diesel::insert_into(payslip_incentives::table)
                            .values(&incentives_entities)
                            .execute(&mut *conn)
                            .await?;
                    }

                    if !rules_entities.is_empty() {
                        diesel::insert_into(payslip_rule_evaluations::table)
                            .values(&rules_entities)
                            .execute(&mut *conn)
                            .await?;
                    }

                    Ok(created_payslip)
                })
            })
            .await
            .map_err(|e| AppError::Internal(format!("db transaction error: {e}")))?;

        Ok(result.into())
    }
}
