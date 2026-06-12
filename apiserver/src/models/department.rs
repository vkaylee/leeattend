use crate::models::custom_field::CustomFieldValueResponse;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Department {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    description: Option<String>,
    manager_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    active_employee_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_fields: Option<Vec<CustomFieldValueResponse>>,
}

impl Department {
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        manager_id: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        active_employee_count: i64,
    ) -> Self {
        Self {
            id,
            workspace_id,
            name,
            description,
            manager_id,
            created_at,
            updated_at,
            active_employee_count,
            custom_fields: None,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn custom_fields(&self) -> Option<&[CustomFieldValueResponse]> {
        self.custom_fields.as_deref()
    }

    #[must_use]
    pub fn with_custom_fields(mut self, custom_fields: Vec<CustomFieldValueResponse>) -> Self {
        self.custom_fields = Some(custom_fields);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn manager_id(&self) -> Option<Uuid> {
        self.manager_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn active_employee_count(&self) -> i64 {
        self.active_employee_count
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub manager_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeeDepartmentHistory {
    id: Uuid,
    employee_id: Uuid,
    department_id: Uuid,
    effective_date: NaiveDate,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

impl EmployeeDepartmentHistory {
    pub fn new(
        id: Uuid,
        employee_id: Uuid,
        department_id: Uuid,
        effective_date: NaiveDate,
        notes: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            employee_id,
            department_id,
            effective_date,
            notes,
            created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn employee_id(&self) -> Uuid {
        self.employee_id
    }

    pub fn department_id(&self) -> Uuid {
        self.department_id
    }

    pub fn effective_date(&self) -> NaiveDate {
        self.effective_date
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TransferEmployeeRequest {
    pub department_id: Uuid,
    pub effective_date: NaiveDate,
    pub notes: Option<String>,
}
