use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Default, ToSchema)]
pub struct OvertimePolicy {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    description: Option<String>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl OvertimePolicy {
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            name,
            description,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OvertimeRule {
    id: Uuid,
    policy_id: Uuid,
    priority: i32,
    multiplier: rust_decimal::Decimal,
    time_start: Option<NaiveTime>,
    time_end: Option<NaiveTime>,
    days_of_week: i32,
    is_holiday: bool,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl OvertimeRule {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        policy_id: Uuid,
        priority: i32,
        multiplier: rust_decimal::Decimal,
        time_start: Option<NaiveTime>,
        time_end: Option<NaiveTime>,
        days_of_week: i32,
        is_holiday: bool,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            policy_id,
            priority,
            multiplier,
            time_start,
            time_end,
            days_of_week,
            is_holiday,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn policy_id(&self) -> Uuid {
        self.policy_id
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn multiplier(&self) -> rust_decimal::Decimal {
        self.multiplier
    }

    pub fn time_start(&self) -> Option<NaiveTime> {
        self.time_start
    }

    pub fn time_end(&self) -> Option<NaiveTime> {
        self.time_end
    }

    pub fn days_of_week(&self) -> i32 {
        self.days_of_week
    }

    pub fn is_holiday(&self) -> bool {
        self.is_holiday
    }

    pub fn set_is_holiday(&mut self, is_holiday: bool) {
        self.is_holiday = is_holiday;
    }

    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OvertimePolicyAssignment {
    id: Uuid,
    workspace_id: Uuid,
    assignee_type: String,
    assignee_id: Option<Uuid>,
    policy_id: Uuid,
    effective_from: NaiveDate,
    effective_to: Option<NaiveDate>,
    created_at: Option<DateTime<Utc>>,
}

impl OvertimePolicyAssignment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        assignee_type: String,
        assignee_id: Option<Uuid>,
        policy_id: Uuid,
        effective_from: NaiveDate,
        effective_to: Option<NaiveDate>,
        created_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            assignee_type,
            assignee_id,
            policy_id,
            effective_from,
            effective_to,
            created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn assignee_type(&self) -> &str {
        &self.assignee_type
    }

    pub fn assignee_id(&self) -> Option<Uuid> {
        self.assignee_id
    }

    pub fn policy_id(&self) -> Uuid {
        self.policy_id
    }

    pub fn effective_from(&self) -> NaiveDate {
        self.effective_from
    }

    pub fn effective_to(&self) -> Option<NaiveDate> {
        self.effective_to
    }

    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BehalfAssignment {
    id: Uuid,
    workspace_id: Uuid,
    assignee_user_id: Uuid,
    target_employee_id: Option<Uuid>,
    target_department_id: Option<Uuid>,
    assignment_type: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    // Optional joined details for client optimization
    assignee_name: Option<String>,
    assignee_employee_id_number: Option<String>,
    target_employee_name: Option<String>,
    target_employee_id_number: Option<String>,
    target_department_name: Option<String>,
}

impl BehalfAssignment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        assignee_user_id: Uuid,
        target_employee_id: Option<Uuid>,
        target_department_id: Option<Uuid>,
        assignment_type: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            assignee_user_id,
            target_employee_id,
            target_department_id,
            assignment_type,
            created_at,
            updated_at,
            assignee_name: None,
            assignee_employee_id_number: None,
            target_employee_name: None,
            target_employee_id_number: None,
            target_department_name: None,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn assignee_user_id(&self) -> Uuid {
        self.assignee_user_id
    }

    pub fn target_employee_id(&self) -> Option<Uuid> {
        self.target_employee_id
    }

    pub fn target_department_id(&self) -> Option<Uuid> {
        self.target_department_id
    }

    pub fn assignment_type(&self) -> &str {
        &self.assignment_type
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn assignee_name(&self) -> Option<&str> {
        self.assignee_name.as_deref()
    }

    pub fn set_assignee_name(&mut self, val: Option<String>) {
        self.assignee_name = val;
    }

    pub fn assignee_employee_id_number(&self) -> Option<&str> {
        self.assignee_employee_id_number.as_deref()
    }

    pub fn set_assignee_employee_id_number(&mut self, val: Option<String>) {
        self.assignee_employee_id_number = val;
    }

    pub fn target_employee_name(&self) -> Option<&str> {
        self.target_employee_name.as_deref()
    }

    pub fn set_target_employee_name(&mut self, val: Option<String>) {
        self.target_employee_name = val;
    }

    pub fn target_employee_id_number(&self) -> Option<&str> {
        self.target_employee_id_number.as_deref()
    }

    pub fn set_target_employee_id_number(&mut self, val: Option<String>) {
        self.target_employee_id_number = val;
    }

    pub fn target_department_name(&self) -> Option<&str> {
        self.target_department_name.as_deref()
    }

    pub fn set_target_department_name(&mut self, val: Option<String>) {
        self.target_department_name = val;
    }
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub struct CreateBehalfAssignmentRequest {
    pub assignee_user_id: Uuid,
    pub target_employee_id: Option<Uuid>,
    pub target_department_id: Option<Uuid>,
}
