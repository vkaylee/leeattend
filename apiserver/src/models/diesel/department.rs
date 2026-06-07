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
    pub std_working_days: Option<bigdecimal::BigDecimal>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<DieselDepartment> for crate::models::Department {
    fn from(val: DieselDepartment) -> Self {
        crate::models::Department::new(
            val.id,
            val.workspace_id,
            val.name,
            val.description,
            val.manager_id,
            val.created_at.unwrap_or_else(Utc::now),
            val.updated_at.unwrap_or_else(Utc::now),
            0, // PoC simplification
        )
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
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = departments)]
pub struct UpdateDieselDepartment {
    pub name: Option<String>,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
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
        crate::models::EmployeeDepartmentHistory::new(
            val.id,
            val.employee_id,
            val.department_id,
            val.effective_date,
            val.notes,
            val.created_at.unwrap_or_else(Utc::now),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::str::FromStr;

    #[test]
    fn test_diesel_department_from() {
        let id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();
        let now = Utc::now();

        let diesel_dept = DieselDepartment {
            id,
            workspace_id,
            name: "Engineering".to_string(),
            description: Some("Tech team".to_string()),
            manager_id: Some(manager_id),
            created_at: Some(now),
            updated_at: Some(now),
            std_working_days: Some(bigdecimal::BigDecimal::from_str("22.5").unwrap()),
            deleted_at: None,
        };

        let dept: crate::models::Department = diesel_dept.into();
        assert_eq!(dept.id(), id);
        assert_eq!(dept.workspace_id(), workspace_id);
        assert_eq!(dept.name(), "Engineering");
        assert_eq!(dept.description(), Some("Tech team"));
        assert_eq!(dept.manager_id(), Some(manager_id));
        assert_eq!(dept.created_at(), now);
        assert_eq!(dept.updated_at(), now);
        assert_eq!(dept.active_employee_count(), 0);
    }

    #[test]
    fn test_diesel_department_from_defaults() {
        let id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        let diesel_dept = DieselDepartment {
            id,
            workspace_id,
            name: "HR".to_string(),
            description: None,
            manager_id: None,
            created_at: None,
            updated_at: None,
            std_working_days: None,
            deleted_at: None,
        };

        let dept: crate::models::Department = diesel_dept.into();
        assert_eq!(dept.id(), id);
        assert_eq!(dept.description(), None);
        assert!(dept.created_at() <= Utc::now());
    }

    #[test]
    fn test_diesel_employee_department_history_from() {
        let id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let dept_id = Uuid::new_v4();
        let effective = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let now = Utc::now();

        let diesel_history = DieselEmployeeDepartmentHistory {
            id,
            employee_id: emp_id,
            department_id: dept_id,
            effective_date: effective,
            notes: Some("Transfer".to_string()),
            created_at: Some(now),
        };

        let history: crate::models::EmployeeDepartmentHistory = diesel_history.into();
        assert_eq!(history.id(), id);
        assert_eq!(history.employee_id(), emp_id);
        assert_eq!(history.department_id(), dept_id);
        assert_eq!(history.effective_date(), effective);
        assert_eq!(history.notes(), Some("Transfer"));
        assert_eq!(history.created_at(), now);
    }
}
