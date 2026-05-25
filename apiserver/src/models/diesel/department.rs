use crate::schema::departments;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, Clone, Debug)]
#[diesel(table_name = departments)]
pub struct DieselDepartment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub default_working_pattern_id: Option<Uuid>,
    pub std_working_days: Option<bigdecimal::BigDecimal>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<DieselDepartment> for crate::models::Department {
    fn from(val: DieselDepartment) -> Self {
        crate::models::Department {
            id: val.id,
            workspace_id: val.workspace_id,
            name: val.name,
            description: val.description,
            manager_id: val.manager_id,
            default_working_pattern_id: val.default_working_pattern_id,
            created_at: val.created_at.unwrap_or_else(Utc::now),
            updated_at: val.updated_at.unwrap_or_else(Utc::now),
            active_employee_count: 0, // PoC simplification
        }
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = departments)]
pub struct NewDieselDepartment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
    pub default_working_pattern_id: Option<Uuid>,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = departments)]
pub struct UpdateDieselDepartment {
    pub name: Option<String>,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
    pub default_working_pattern_id: Option<Option<Uuid>>,
}

#[derive(Queryable, Selectable, Identifiable, Clone, Debug)]
#[diesel(table_name = crate::schema::department_payroll_configs)]
pub struct DieselDepartmentPayrollConfig {
    pub id: Uuid,
    pub department_id: Uuid,
    pub std_working_days: bigdecimal::BigDecimal,
    pub effective_date: chrono::NaiveDate,
    pub created_at: Option<DateTime<Utc>>,
    pub hours_per_day: bigdecimal::BigDecimal,
}

impl From<DieselDepartmentPayrollConfig> for crate::models::DepartmentPayrollConfig {
    fn from(val: DieselDepartmentPayrollConfig) -> Self {
        use std::str::FromStr;
        crate::models::DepartmentPayrollConfig {
            id: val.id,
            department_id: val.department_id,
            std_working_days: rust_decimal::Decimal::from_str(&val.std_working_days.to_string())
                .unwrap_or_default(),
            effective_date: val.effective_date,
            created_at: val.created_at.unwrap_or_else(Utc::now),
            hours_per_day: rust_decimal::Decimal::from_str(&val.hours_per_day.to_string())
                .unwrap_or_default(),
        }
    }
}

#[derive(Queryable, Selectable, Identifiable, Clone, Debug)]
#[diesel(table_name = crate::schema::employee_department_history)]
pub struct DieselEmployeeDepartmentHistory {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub department_id: Uuid,
    pub effective_date: chrono::NaiveDate,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<DieselEmployeeDepartmentHistory> for crate::models::EmployeeDepartmentHistory {
    fn from(val: DieselEmployeeDepartmentHistory) -> Self {
        crate::models::EmployeeDepartmentHistory {
            id: val.id,
            employee_id: val.employee_id,
            department_id: val.department_id,
            effective_date: val.effective_date,
            notes: val.notes,
            created_at: val.created_at.unwrap_or_else(Utc::now),
        }
    }
}
