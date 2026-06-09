use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{
    employee_salary_components, payroll_incentive_policies, payroll_incentive_tiers,
    payslip_incentives, payslip_rule_evaluations, payslip_segments, payslips,
    salary_component_definitions,
};
use crate::utils::bigdecimal_helpers::bd_to_dec;

// ---------------------------------------------------------------------------
// Payroll Incentive Policy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = payroll_incentive_policies)]
pub struct PayrollIncentivePolicyEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub base_amount: f64,
    pub payout_type: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Incentive Tier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = payroll_incentive_tiers)]
pub struct IncentiveTierEntity {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub priority: i32,
    pub payout_amount: f64,
    pub rule_config: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Payslip
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = payslips)]
pub struct PayslipEntity {
    pub id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub base_pay_amount: BigDecimal,
    pub overtime_pay_amount: BigDecimal,
    pub deductions_amount: BigDecimal,
    pub net_pay_amount: BigDecimal,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub standard_units: i32,
    pub overtime_units: i32,
    pub holiday_work_premium_amount: BigDecimal,
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
}

// ---------------------------------------------------------------------------
// Payslip Incentive
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = payslip_incentives)]
pub struct PayslipIncentiveEntity {
    pub payslip_id: Uuid,
    pub policy_name: String,
    pub amount: f64,
    pub tier_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub standard_amount: f64,
    pub id: Uuid,
}

// ---------------------------------------------------------------------------
// Payslip Rule Evaluation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = payslip_rule_evaluations)]
pub struct PayslipRuleEvaluationEntity {
    pub id: Uuid,
    pub payslip_id: Uuid,
    pub rule_code: String,
    pub amount: BigDecimal,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Salary Component Definition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = salary_component_definitions)]
pub struct SalaryComponentDefinitionEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub code: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub component_type: String,
    pub linked_rule_id: Option<Uuid>,
}

// ---------------------------------------------------------------------------
// Employee Salary Component
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = employee_salary_components)]
pub struct EmployeeSalaryComponentEntity {
    pub id: Uuid,
    pub component_definition_id: Uuid,
    pub amount: BigDecimal,
    pub effective_date: NaiveDate,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub employment_id: Uuid,
}

// ---------------------------------------------------------------------------
// Payslip Segment
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Identifiable, AsChangeset)]
#[diesel(table_name = payslip_segments)]
pub struct PayslipSegmentEntity {
    pub id: Uuid,
    pub payslip_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub department_id: Option<Uuid>,
    pub applied_base_salary: BigDecimal,
    pub applied_std_days: BigDecimal,
    pub actual_work_days: BigDecimal,
    pub paid_work_days: BigDecimal,
    pub amount: BigDecimal,
    pub created_at: Option<DateTime<Utc>>,
    pub applied_hours_per_day: BigDecimal,
    pub std_days_source: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

use crate::models::{
    EmployeeSalaryComponent, IncentiveTier, PayrollIncentivePolicy, Payslip, PayslipRuleEvaluation,
    PayslipSegment, SalaryComponentDefinition,
};

impl From<PayrollIncentivePolicyEntity> for PayrollIncentivePolicy {
    fn from(entity: PayrollIncentivePolicyEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            base_amount: entity.base_amount,
            payout_type: entity.payout_type,
            is_active: entity.is_active,
            created_at: entity.created_at,
            effective_from: entity.effective_from,
            effective_to: entity.effective_to,
        }
    }
}

impl From<IncentiveTierEntity> for IncentiveTier {
    fn from(entity: IncentiveTierEntity) -> Self {
        Self {
            id: entity.id,
            policy_id: entity.policy_id,
            priority: entity.priority,
            rule_config: entity.rule_config,
            payout_amount: entity.payout_amount,
        }
    }
}

impl From<PayslipEntity> for Payslip {
    fn from(entity: PayslipEntity) -> Self {
        Self {
            id: entity.id,
            employment_id: entity.employment_id,
            workspace_id: entity.workspace_id,
            period_start: entity.period_start,
            period_end: entity.period_end,
            base_pay_amount: bd_to_dec(&entity.base_pay_amount),
            overtime_pay_amount: bd_to_dec(&entity.overtime_pay_amount),
            deductions_amount: bd_to_dec(&entity.deductions_amount),
            net_pay_amount: bd_to_dec(&entity.net_pay_amount),
            status: entity.status,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            updated_at: entity.updated_at.unwrap_or_else(Utc::now),
            standard_units: entity.standard_units,
            overtime_units: entity.overtime_units,
            holiday_work_premium_amount: bd_to_dec(&entity.holiday_work_premium_amount),
        }
    }
}

impl From<PayslipRuleEvaluationEntity> for PayslipRuleEvaluation {
    fn from(entity: PayslipRuleEvaluationEntity) -> Self {
        Self {
            id: entity.id,
            payslip_id: entity.payslip_id,
            rule_code: entity.rule_code,
            amount: bd_to_dec(&entity.amount),
            created_at: entity.created_at,
        }
    }
}

impl From<SalaryComponentDefinitionEntity> for SalaryComponentDefinition {
    fn from(entity: SalaryComponentDefinitionEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            code: entity.code,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            updated_at: entity.updated_at.unwrap_or_else(Utc::now),
        }
    }
}

impl From<EmployeeSalaryComponentEntity> for EmployeeSalaryComponent {
    fn from(entity: EmployeeSalaryComponentEntity) -> Self {
        Self {
            id: entity.id,
            employment_id: entity.employment_id,
            component_definition_id: entity.component_definition_id,
            amount: bd_to_dec(&entity.amount),
            effective_date: entity.effective_date,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            updated_at: entity.updated_at.unwrap_or_else(Utc::now),
        }
    }
}

impl From<PayslipSegmentEntity> for PayslipSegment {
    fn from(entity: PayslipSegmentEntity) -> Self {
        Self {
            id: entity.id,
            payslip_id: entity.payslip_id,
            start_date: entity.start_date,
            end_date: entity.end_date,
            department_id: entity.department_id,
            applied_base_salary: bd_to_dec(&entity.applied_base_salary),
            applied_std_days: bd_to_dec(&entity.applied_std_days),
            actual_work_days: bd_to_dec(&entity.actual_work_days),
            paid_work_days: bd_to_dec(&entity.paid_work_days),
            amount: bd_to_dec(&entity.amount),
            applied_hours_per_day: bd_to_dec(&entity.applied_hours_per_day),
            std_days_source: entity.std_days_source,
            reason: None, // Reason is not in the DB entity, likely a transient field or from another table
        }
    }
}
