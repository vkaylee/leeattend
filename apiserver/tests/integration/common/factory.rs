use apiserver::diesel_pool::DieselPools;
use apiserver::models::LeaveStatus;
use chrono::{DateTime, Datelike, Utc};
use uuid::Uuid;

pub struct TestFactory {
    pub diesel_pools: apiserver::diesel_pool::DieselPools,
    pub redis_pool: Option<apiserver::queue::RedisPool>,
}

impl TestFactory {
    #[allow(dead_code)]
    #[must_use]
    pub const fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        Self {
            diesel_pools,
            redis_pool: None,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_redis(mut self, redis_pool: Option<apiserver::queue::RedisPool>) -> Self {
        self.redis_pool = redis_pool;
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn user(&self) -> UserBuilder {
        UserBuilder::new(self.diesel_pools.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn workspace(&self) -> WorkspaceBuilder {
        WorkspaceBuilder::new(self.diesel_pools.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn agent(&self) -> AgentBuilder {
        AgentBuilder::new(self.diesel_pools.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn employee(&self) -> EmployeeBuilder {
        EmployeeBuilder::new(self.diesel_pools.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn shift(&self) -> ShiftBuilder {
        ShiftBuilder::new(self.diesel_pools.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn time_entry(&self) -> TimeEntryBuilder {
        TimeEntryBuilder::new(self.diesel_pools.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn leave_request(&self) -> LeaveRequestBuilder {
        LeaveRequestBuilder::new(self.diesel_pools.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn leave_balance(&self) -> LeaveBalanceBuilder {
        LeaveBalanceBuilder::new(self.diesel_pools.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn leave_policy(&self) -> LeavePolicyBuilder {
        LeavePolicyBuilder::new(self.diesel_pools.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn department(&self) -> DepartmentBuilder {
        DepartmentBuilder::new(self.diesel_pools.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn assignment_rule(&self) -> AssignmentRuleBuilder {
        AssignmentRuleBuilder::new(self.diesel_pools.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn auth(&self) -> AuthHelper {
        AuthHelper::new()
    }
}

#[derive(Default)]
pub struct AuthHelper;

impl AuthHelper {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn create_token(&self, user_id: Uuid) -> String {
        let now = chrono::Utc::now();
        let claims = apiserver::services::auth_service::Claims {
            sub: user_id.to_string(),
            workspace_id: None,
            exp: (now + chrono::Duration::hours(1)).timestamp(),
            iat: now.timestamp(),
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret("test_secret_tower".as_ref()),
        )
        .unwrap()
    }
}

#[allow(dead_code)]
pub struct UserBuilder {
    diesel_pools: apiserver::diesel_pool::DieselPools,
    email: String,
    full_name: Option<String>,
    password_hash: String,
}

impl UserBuilder {
    #[allow(dead_code)]
    #[must_use]
    pub fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            diesel_pools,
            email: format!("user_{}@example.com", &id[..8]),
            full_name: None,
            password_hash: "test_hash".to_string(),
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_email(mut self, email: &str) -> Self {
        self.email = email.to_string();
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_full_name(mut self, name: &str) -> Self {
        self.full_name = Some(name.to_string());
        self
    }

    #[allow(dead_code)]
    pub async fn create(self) -> Uuid {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            crate::test_query_scalar_uuid!(
        &self.diesel_pools.api,
        "INSERT INTO users (email, full_name, password_hash) VALUES ($1, $2, $3) RETURNING id",
        diesel::sql_types::Text, self.email,
        diesel::sql_types::Nullable<diesel::sql_types::Text>, self.full_name,
        diesel::sql_types::Text, self.password_hash
    )
        })
        .await
        .expect("User creation timed out")
    }
}

#[allow(dead_code)]
pub struct WorkspaceBuilder {
    diesel_pools: apiserver::diesel_pool::DieselPools,
    name: String,
    timezone: String,
}

impl WorkspaceBuilder {
    #[allow(dead_code)]
    #[must_use]
    pub fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            diesel_pools,
            name: format!("Workspace {}", &id[..8]),
            timezone: "Asia/Ho_Chi_Minh".to_string(),
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_timezone(mut self, tz: &str) -> Self {
        self.timezone = tz.to_string();
        self
    }

    #[allow(dead_code)]
    pub async fn create(self) -> Uuid {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let ws_id: Uuid = crate::test_query_scalar_uuid!(
        &self.diesel_pools.api,
        "INSERT INTO workspaces (name, timezone) VALUES ($1, $2) RETURNING id",
        diesel::sql_types::Text, self.name,
        diesel::sql_types::Text, self.timezone.clone()
    );

            // Seed default leave types for tests
            crate::test_execute!(
        &self.diesel_pools.api,
        "INSERT INTO leave_types (workspace_id, code, name, is_paid) VALUES
                ($1, 'vacation', 'Vacation', true),
                ($1, 'sick', 'Sick Leave', true),
                ($1, 'personal', 'Personal Leave', true),
                ($1, 'unpaid', 'Unpaid Leave', false)",
        diesel::sql_types::Uuid, ws_id
    );

            // Seed default attendance policy for tests
            let policy_id: Uuid = crate::test_query_scalar_uuid!(
        &self.diesel_pools.api,
        "INSERT INTO attendance_policies (workspace_id, name, is_default) VALUES ($1, 'Default Test Policy', true) RETURNING id",
        diesel::sql_types::Uuid, ws_id
    );

            let rules_json = serde_json::json!({
                "work_unit_minutes": 30,
                "rounding_policy": "floor"
            });
            crate::test_execute!(
        &self.diesel_pools.api,
        "INSERT INTO attendance_policy_versions (policy_id, rules, valid_from) VALUES ($1, $2, '2000-01-01 00:00:00Z')",
        diesel::sql_types::Uuid, policy_id,
        diesel::sql_types::Jsonb, rules_json
    );

            // Seed default work location
            crate::test_execute!(
        &self.diesel_pools.api,
        "INSERT INTO work_locations (id, workspace_id, name, timezone, is_headquarters, status) VALUES ($1, $2, 'Headquarters', $3, true, 'active')",
        diesel::sql_types::Uuid, Uuid::new_v4(),
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Text, self.timezone.clone()
    );

            // Seed default employer
            crate::test_execute!(
        &self.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Default Test Employer', 'DEFAULT-TAX-1', 'Default Address', NOW(), NOW())",
        diesel::sql_types::Uuid, Uuid::new_v4(),
        diesel::sql_types::Uuid, ws_id
    );

            ws_id
        })
        .await
        .expect("Workspace creation timed out")
    }

    #[allow(dead_code)]
    pub async fn create_with_owner(self, user_id: Uuid) -> Uuid {
        let pool = self.diesel_pools.clone();
        let ws_id = self.create().await;
        crate::test_execute!(
            &pool.api,
            "INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ($1, $2, $3)",
            diesel::sql_types::Uuid,
            ws_id,
            diesel::sql_types::Uuid,
            user_id,
            diesel::sql_types::Uuid,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000001").unwrap()
        );
        ws_id
    }
}

impl TestFactory {
    #[allow(dead_code)]
    pub async fn add_member(&self, ws_id: Uuid, user_id: Uuid, role_id: Uuid) {
        crate::test_execute!(
            &self.diesel_pools.api,
            "INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ($1, $2, $3)",
            diesel::sql_types::Uuid,
            ws_id,
            diesel::sql_types::Uuid,
            user_id,
            diesel::sql_types::Uuid,
            role_id
        );
    }

    #[allow(dead_code)]
    pub async fn add_extra_role(
        &self,
        ws_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
        expires_at: DateTime<Utc>,
    ) {
        crate::test_execute!(
        &self.diesel_pools.api,
        "INSERT INTO member_extra_roles (workspace_id, user_id, role_id, expires_at) VALUES ($1, $2, $3, $4) RETURNING *",
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, user_id,
        diesel::sql_types::Uuid, role_id,
        diesel::sql_types::Timestamptz, expires_at
    );
    }
}

// --- Employee Builder ---
#[allow(dead_code)]
pub struct EmployeeBuilder {
    diesel_pools: apiserver::diesel_pool::DieselPools,
    workspace_id: Option<Uuid>,
    user_id: Option<Uuid>,
    full_name: String,
    employee_id_number: String,
    hire_date: chrono::NaiveDate,
}

#[allow(dead_code)]
impl EmployeeBuilder {
    #[must_use]
    pub fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            diesel_pools,
            workspace_id: None,
            user_id: None,
            full_name: "Test Employee".to_string(),
            employee_id_number: format!("EMP-{}", Uuid::new_v4().as_u128() % 1_000_000_000),
            hire_date: chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_workspace(mut self, ws_id: Uuid) -> Self {
        self.workspace_id = Some(ws_id);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_employee_id_number(mut self, id_num: &str) -> Self {
        self.employee_id_number = id_num.to_string();
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_full_name(mut self, name: &str) -> Self {
        self.full_name = name.to_string();
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_hire_date(mut self, date: chrono::NaiveDate) -> Self {
        self.hire_date = date;
        self
    }

    pub async fn create(self) -> Uuid {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let normalized_id = apiserver::utils::normalize_employee_id(&self.employee_id_number);
            let workspace_id = self.workspace_id.expect("Workspace ID is required for Employee");
            let location_id: Uuid = crate::test_query_scalar_uuid!(
                &self.diesel_pools.api,
                "SELECT id FROM work_locations WHERE workspace_id = $1 AND is_headquarters = true LIMIT 1",
                diesel::sql_types::Uuid, workspace_id
            );

            let employer_id: Uuid = match crate::test_query_scalar_uuid_optional!(
                &self.diesel_pools.api,
                "SELECT id FROM employers WHERE workspace_id = $1 LIMIT 1",
                diesel::sql_types::Uuid, workspace_id
            ) {
                Some(id) => id,
                None => {
                    let new_id = Uuid::new_v4();
                    crate::test_execute!(
                        &self.diesel_pools.api,
                        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Default Test Employer', 'DEFAULT-TAX-1', 'Default Address', NOW(), NOW())",
                        diesel::sql_types::Uuid, new_id,
                        diesel::sql_types::Uuid, workspace_id
                    );
                    new_id
                }
            };

            let employee_id = crate::test_query_scalar_uuid!(
                &self.diesel_pools.api,
                "INSERT INTO employees (workspace_id, user_id, full_name, employee_id_number, hire_date, original_hire_date) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
                diesel::sql_types::Uuid, workspace_id,
                diesel::sql_types::Nullable<diesel::sql_types::Uuid>, self.user_id,
                diesel::sql_types::Text, self.full_name,
                diesel::sql_types::Text, normalized_id,
                diesel::sql_types::Date, self.hire_date,
                diesel::sql_types::Date, self.hire_date
            );

            crate::test_execute!(
                &self.diesel_pools.api,
                "INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'active', $6, NOW(), NOW())",
                diesel::sql_types::Uuid, Uuid::new_v4(),
                diesel::sql_types::Uuid, workspace_id,
                diesel::sql_types::Uuid, employee_id,
                diesel::sql_types::Uuid, employer_id,
                diesel::sql_types::Nullable<diesel::sql_types::Uuid>, Some(location_id),
                diesel::sql_types::Date, self.hire_date
            );

            employee_id
        }).await.expect("Employee creation timed out")
    }
}

// --- Shift Builder ---
#[allow(dead_code)]
pub struct ShiftBuilder {
    diesel_pools: apiserver::diesel_pool::DieselPools,
    workspace_id: Option<Uuid>,
    name: String,
    start_time: chrono::NaiveTime,
    end_time: chrono::NaiveTime,
    break_duration_minutes: i32,
    break_is_overtime: bool,
    early_punch_in_grace_period_minutes: i32,
    allow_early_ot: bool,
}

#[allow(dead_code)]
impl ShiftBuilder {
    #[must_use]
    pub fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            diesel_pools,
            workspace_id: None,
            name: format!("Shift {}", &id[..8]),
            start_time: chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_time: chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            break_duration_minutes: 60,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_workspace(mut self, ws_id: Uuid) -> Self {
        self.workspace_id = Some(ws_id);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_times(mut self, start: chrono::NaiveTime, end: chrono::NaiveTime) -> Self {
        self.start_time = start;
        self.end_time = end;
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_break_duration(mut self, minutes: i32) -> Self {
        self.break_duration_minutes = minutes;
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_grace_period(mut self, minutes: i32) -> Self {
        self.early_punch_in_grace_period_minutes = minutes;
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_allow_early_ot(mut self, allow: bool) -> Self {
        self.allow_early_ot = allow;
        self
    }

    pub async fn create(self) -> Uuid {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            crate::test_query_scalar_uuid!(
                &self.diesel_pools.api,
                "INSERT INTO shifts (workspace_id, name, start_time, end_time, break_duration_minutes, break_is_overtime, early_punch_in_grace_period_minutes, allow_early_ot) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
                diesel::sql_types::Uuid, self.workspace_id.expect("Workspace ID is required for Shift"),
                diesel::sql_types::Text, self.name,
                diesel::sql_types::Time, self.start_time,
                diesel::sql_types::Time, self.end_time,
                diesel::sql_types::Integer, self.break_duration_minutes,
                diesel::sql_types::Bool, self.break_is_overtime,
                diesel::sql_types::Integer, self.early_punch_in_grace_period_minutes,
                diesel::sql_types::Bool, self.allow_early_ot
            )
        }).await.expect("Shift creation timed out")
    }

    pub async fn assign_to_employee(
        self,
        employee_id: Uuid,
        start_date: chrono::NaiveDate,
    ) -> Uuid {
        let pool = self.diesel_pools.clone();
        let shift_id = self.create().await;
        crate::test_execute!(
            &pool.api,
            "INSERT INTO employee_shifts (employee_id, shift_id, start_date) VALUES ($1, $2, $3)",
            diesel::sql_types::Uuid,
            employee_id,
            diesel::sql_types::Uuid,
            shift_id,
            diesel::sql_types::Date,
            start_date
        );
        shift_id
    }
}

// --- TimeEntry Builder ---
#[allow(dead_code)]
pub struct TimeEntryBuilder {
    diesel_pools: apiserver::diesel_pool::DieselPools,
    employee_id: Option<Uuid>,
    punch_in_time: DateTime<Utc>,
    punch_out_time: Option<DateTime<Utc>>,
    is_break_overtime: bool,
    entry_type: String,
}

#[allow(dead_code)]
impl TimeEntryBuilder {
    #[must_use]
    pub fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        Self {
            diesel_pools,
            employee_id: None,
            punch_in_time: Utc::now(),
            punch_out_time: None,
            is_break_overtime: false,
            entry_type: "work".to_string(),
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_employee(mut self, employee_id: Uuid) -> Self {
        self.employee_id = Some(employee_id);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_punch_in(mut self, time: DateTime<Utc>) -> Self {
        self.punch_in_time = time;
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_punch_out(mut self, time: DateTime<Utc>) -> Self {
        self.punch_out_time = Some(time);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_is_break_overtime(mut self, val: bool) -> Self {
        self.is_break_overtime = val;
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_entry_type(mut self, entry_type: &str) -> Self {
        self.entry_type = entry_type.to_string();
        self
    }

    pub async fn create(self) -> Uuid {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let duration_minutes = self.punch_out_time.map(|po| {
                po.signed_duration_since(self.punch_in_time).num_minutes() as i32
            });

            let employee_id = self.employee_id.expect("Employee ID is required for TimeEntry");
            let workspace_id: Uuid = crate::test_query_scalar_uuid!(
        &self.diesel_pools.api,
        "SELECT workspace_id AS id FROM employees WHERE id = $1",
        diesel::sql_types::Uuid, employee_id
    );

            crate::test_query_scalar_uuid!(
        &self.diesel_pools.api,
        "INSERT INTO time_entries (employee_id, workspace_id, punch_in_time, punch_out_time, is_break_overtime, entry_type, duration_minutes) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        diesel::sql_types::Uuid, employee_id,
        diesel::sql_types::Uuid, workspace_id,
        diesel::sql_types::Timestamptz, self.punch_in_time,
        diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, self.punch_out_time,
        diesel::sql_types::Bool, self.is_break_overtime,
        diesel::sql_types::Text, self.entry_type,
        diesel::sql_types::Nullable<diesel::sql_types::Integer>, duration_minutes
    )}).await.expect("TimeEntry creation timed out")
    }
}

#[allow(dead_code)]
pub struct AgentBuilder {
    diesel_pools: apiserver::diesel_pool::DieselPools,
    id: String,
    name: String,
    token: String,
    workspace_id: Option<Uuid>,
    is_trusted: bool,
}

#[allow(dead_code)]
impl AgentBuilder {
    #[must_use]
    pub fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            diesel_pools,
            id: format!("AGENT_{}", &id[..8]),
            name: "Test Agent".to_string(),
            token: "test_token".to_string(),
            workspace_id: None,
            is_trusted: false,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_token(mut self, token: &str) -> Self {
        self.token = token.to_string();
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_workspace(mut self, ws_id: Uuid) -> Self {
        self.workspace_id = Some(ws_id);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn with_trust(mut self, trusted: bool) -> Self {
        self.is_trusted = trusted;
        self
    }

    pub async fn create(self) -> String {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let ws_id = self.workspace_id.expect("Workspace ID is required for Agent");
            crate::test_execute!(
        &self.diesel_pools.api,
        "INSERT INTO authorized_agents (id, name, token, workspace_id, is_trusted, is_active) VALUES ($1, $2, $3, $4, $5, true)",
        diesel::sql_types::Text, &self.id,
        diesel::sql_types::Text, &self.name,
        diesel::sql_types::Text, &self.token,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Bool, self.is_trusted
    );
            self.id
        }).await.expect("Agent creation timed out")
    }
}

impl TestFactory {
    #[allow(dead_code)]
    pub async fn cleanup(&self) {
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            crate::test_execute!(&self.diesel_pools.api, "DELETE FROM time_entries CASCADE");
            crate::test_execute!(
                &self.diesel_pools.api,
                "DELETE FROM employee_shifts CASCADE"
            );
            crate::test_execute!(&self.diesel_pools.api, "DELETE FROM shifts CASCADE");
            crate::test_execute!(&self.diesel_pools.api, "DELETE FROM employees CASCADE");
            crate::test_execute!(
                &self.diesel_pools.api,
                "DELETE FROM member_extra_roles CASCADE"
            );
            crate::test_execute!(
                &self.diesel_pools.api,
                "DELETE FROM workspace_members CASCADE"
            );
            crate::test_execute!(&self.diesel_pools.api, "DELETE FROM devices CASCADE");
            crate::test_execute!(
                &self.diesel_pools.api,
                "DELETE FROM authorized_agents CASCADE"
            );
            crate::test_execute!(
                &self.diesel_pools.api,
                "DELETE FROM assignment_rules CASCADE"
            );
            crate::test_execute!(
                &self.diesel_pools.api,
                "DELETE FROM leave_accrual_logs CASCADE"
            );
            crate::test_execute!(
                &self.diesel_pools.api,
                "DELETE FROM leave_policy_assignments CASCADE"
            );
            crate::test_execute!(
                &self.diesel_pools.api,
                "DELETE FROM leave_policy_definitions CASCADE"
            );
            crate::test_execute!(&self.diesel_pools.api, "DELETE FROM leave_requests CASCADE");
            crate::test_execute!(&self.diesel_pools.api, "DELETE FROM leave_balances CASCADE");
            crate::test_execute!(&self.diesel_pools.api, "DELETE FROM users CASCADE");
            crate::test_execute!(
                &self.diesel_pools.api,
                "DELETE FROM attendance_policy_versions CASCADE"
            );
            crate::test_execute!(
                &self.diesel_pools.api,
                "DELETE FROM attendance_policies CASCADE"
            );
            crate::test_execute!(&self.diesel_pools.api, "DELETE FROM workspaces CASCADE");
        })
        .await
        .expect("Cleanup timed out");
    }
}

// --- Leave Request Builder ---
#[allow(dead_code)]
pub struct LeaveRequestBuilder {
    diesel_pools: apiserver::diesel_pool::DieselPools,
    employee_id: Option<Uuid>,
    leave_type_id: Option<Uuid>,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    status: LeaveStatus,
}

#[allow(dead_code)]
impl LeaveRequestBuilder {
    #[must_use]
    pub fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        Self {
            diesel_pools,
            employee_id: None,
            leave_type_id: None,
            start_date: chrono::Utc::now().date_naive(),
            end_date: chrono::Utc::now().date_naive(),
            status: LeaveStatus::Pending,
        }
    }

    #[must_use]
    pub const fn with_employee(mut self, employee_id: Uuid) -> Self {
        self.employee_id = Some(employee_id);
        self
    }

    #[must_use]
    pub const fn with_dates(mut self, start: chrono::NaiveDate, end: chrono::NaiveDate) -> Self {
        self.start_date = start;
        self.end_date = end;
        self
    }

    #[must_use]
    pub const fn with_status(mut self, status: LeaveStatus) -> Self {
        self.status = status;
        self
    }

    #[must_use]
    pub const fn with_leave_type(mut self, leave_type_id: Uuid) -> Self {
        self.leave_type_id = Some(leave_type_id);
        self
    }

    pub async fn create(self) -> Uuid {
        let id = Uuid::new_v4();
        let emp_id = self
            .employee_id
            .expect("Employee ID is required for LeaveRequest");
        let leave_type_id = if let Some(id) = self.leave_type_id {
            id
        } else {
            let ws_id: Uuid = crate::test_query_scalar_uuid!(
                &self.diesel_pools.api,
                "SELECT workspace_id AS id FROM employees WHERE id = $1",
                diesel::sql_types::Uuid,
                emp_id
            );
            crate::test_query_scalar_uuid!(
                &self.diesel_pools.api,
                "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
                diesel::sql_types::Uuid,
                ws_id
            )
        };

        let status_str = match self.status {
            LeaveStatus::Pending => "pending",
            LeaveStatus::Approved => "approved",
            LeaveStatus::Rejected => "rejected",
            LeaveStatus::Cancelled => "cancelled",
            LeaveStatus::PreApproved => "pre_approved",
        };

        let employment_id: Uuid = crate::test_query_scalar_uuid!(
            &self.diesel_pools.api,
            "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
            diesel::sql_types::Uuid,
            emp_id
        );

        crate::test_execute!(
            &self.diesel_pools.api,
            "INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, status) VALUES ($1, $2, $3, $4, $5, $6::leave_status_type)",
            diesel::sql_types::Uuid, id,
            diesel::sql_types::Uuid, employment_id,
            diesel::sql_types::Uuid, leave_type_id,
            diesel::sql_types::Date, self.start_date,
            diesel::sql_types::Date, self.end_date,
            diesel::sql_types::Text, status_str
        );
        id
    }
}

// --- Leave Balance Builder ---
#[allow(dead_code)]
pub struct LeaveBalanceBuilder {
    diesel_pools: apiserver::diesel_pool::DieselPools,
    employee_id: Option<Uuid>,
    leave_type_id: Option<Uuid>,
    year: i32,
    entitlement: i32,
    carried_over: i32,
}

#[allow(dead_code)]
impl LeaveBalanceBuilder {
    #[must_use]
    pub fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        Self {
            diesel_pools,
            employee_id: None,
            leave_type_id: None,
            year: chrono::Utc::now().date_naive().year(),
            entitlement: 12,
            carried_over: 0,
        }
    }

    #[must_use]
    pub const fn with_employee(mut self, employee_id: Uuid) -> Self {
        self.employee_id = Some(employee_id);
        self
    }

    #[must_use]
    pub const fn with_entitlement(mut self, entitlement: i32) -> Self {
        self.entitlement = entitlement;
        self
    }

    #[must_use]
    pub const fn with_leave_type(mut self, leave_type_id: Uuid) -> Self {
        self.leave_type_id = Some(leave_type_id);
        self
    }

    pub async fn create(self) -> Uuid {
        let id = Uuid::new_v4();
        let emp_id = self
            .employee_id
            .expect("Employee ID is required for LeaveBalance");
        let leave_type_id = if let Some(id) = self.leave_type_id {
            id
        } else {
            let ws_id: Uuid = crate::test_query_scalar_uuid!(
                &self.diesel_pools.api,
                "SELECT workspace_id AS id FROM employees WHERE id = $1",
                diesel::sql_types::Uuid,
                emp_id
            );
            crate::test_query_scalar_uuid!(
                &self.diesel_pools.api,
                "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
                diesel::sql_types::Uuid,
                ws_id
            )
        };

        let employment_id: Uuid = crate::test_query_scalar_uuid!(
            &self.diesel_pools.api,
            "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
            diesel::sql_types::Uuid,
            emp_id
        );

        crate::test_execute!(
            &self.diesel_pools.api,
            "INSERT INTO leave_balances (id, employment_id, leave_type_id, year, entitlement, carried_over) VALUES ($1, $2, $3, $4, $5, $6)",
            diesel::sql_types::Uuid, id,
            diesel::sql_types::Uuid, employment_id,
            diesel::sql_types::Uuid, leave_type_id,
            diesel::sql_types::Integer, self.year,
            diesel::sql_types::Numeric, rust_decimal::Decimal::from(self.entitlement),
            diesel::sql_types::Numeric, rust_decimal::Decimal::from(self.carried_over)
        );
        id
    }
}

// --- Leave Policy Builder ---
#[allow(dead_code)]
pub struct LeavePolicyBuilder {
    diesel_pools: apiserver::diesel_pool::DieselPools,
    workspace_id: Option<Uuid>,
    name: String,
    rule_config: serde_json::Value,
}

#[allow(dead_code)]
impl LeavePolicyBuilder {
    #[must_use]
    pub fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        Self {
            diesel_pools,
            workspace_id: None,
            name: "Test Policy".to_string(),
            rule_config: serde_json::json!({
                "frequency": "monthly",
                "accrual_amount": 1.0,
                "accrual_timing": "start_of_period"
            }),
        }
    }

    #[must_use]
    pub const fn with_workspace(mut self, ws_id: Uuid) -> Self {
        self.workspace_id = Some(ws_id);
        self
    }

    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    #[must_use]
    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.rule_config = config;
        self
    }

    pub async fn create(self) -> Uuid {
        let ws_id = self.workspace_id.expect("Workspace ID required");
        let pool = self.diesel_pools.clone();

        // Fetch vacation ID as default
        let leave_type_id: Uuid = crate::test_query_scalar_uuid!(
            &pool.api,
            "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
            diesel::sql_types::Uuid,
            ws_id
        );

        crate::test_query_scalar_uuid!(&pool.api,
            "INSERT INTO leave_policy_definitions (workspace_id, name, rule_config, leave_type_id) VALUES ($1, $2, $3, $4) RETURNING id",
            diesel::sql_types::Uuid, ws_id,
            diesel::sql_types::Text, self.name,
            diesel::sql_types::Jsonb, self.rule_config,
            diesel::sql_types::Uuid, leave_type_id
        )
    }

    pub async fn assign_to(
        &self,
        workspace_id: Uuid,
        policy_id: Uuid,
        assignee_type: &str,
        assignee_id: Option<Uuid>,
        from: chrono::NaiveDate,
        to: Option<chrono::NaiveDate>,
    ) -> Uuid {
        crate::test_query_scalar_uuid!(
        &self.diesel_pools.api,
        "INSERT INTO leave_policy_assignments (workspace_id, policy_id, assignee_type, assignee_id, effective_from, effective_to) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        diesel::sql_types::Uuid, workspace_id,
        diesel::sql_types::Uuid, policy_id,
        diesel::sql_types::Text, assignee_type,
        diesel::sql_types::Nullable<diesel::sql_types::Uuid>, assignee_id,
        diesel::sql_types::Date, from,
        diesel::sql_types::Nullable<diesel::sql_types::Date>, to
    )
    }
}

// --- Department Builder ---
#[allow(dead_code)]
pub struct DepartmentBuilder {
    diesel_pools: apiserver::diesel_pool::DieselPools,
    workspace_id: Option<Uuid>,
    name: String,
    description: Option<String>,
    manager_id: Option<Uuid>,
}

#[allow(dead_code)]
impl DepartmentBuilder {
    #[must_use]
    pub fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            diesel_pools,
            workspace_id: None,
            name: format!("Dept {}", &id[..8]),
            description: None,
            manager_id: None,
        }
    }

    #[must_use]
    pub const fn with_workspace(mut self, ws_id: Uuid) -> Self {
        self.workspace_id = Some(ws_id);
        self
    }

    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub async fn create(self) -> Uuid {
        crate::test_query_scalar_uuid!(
            &self.diesel_pools.api,
            "INSERT INTO departments (workspace_id, name, description, manager_id) VALUES ($1, $2, $3, $4) RETURNING id",
            diesel::sql_types::Uuid, self.workspace_id.expect("Workspace ID required"),
            diesel::sql_types::Text, self.name,
            diesel::sql_types::Nullable<diesel::sql_types::Text>, self.description,
            diesel::sql_types::Nullable<diesel::sql_types::Uuid>, self.manager_id
        )
    }
}

// --- Assignment Rule Builder ---
#[allow(dead_code)]
pub struct AssignmentRuleBuilder {
    diesel_pools: apiserver::diesel_pool::DieselPools,
    workspace_id: Option<Uuid>,
    pattern_id: Option<Uuid>,
    name: String,
    priority: i32,
    conditions: apiserver::models::AssignmentRuleConditions,
    start_date: Option<chrono::NaiveDate>,
    end_date: Option<chrono::NaiveDate>,
}

#[allow(dead_code)]
impl AssignmentRuleBuilder {
    #[must_use]
    pub fn new(diesel_pools: apiserver::diesel_pool::DieselPools) -> Self {
        Self {
            diesel_pools,
            workspace_id: None,
            pattern_id: None,
            name: "Test Rule".to_string(),
            priority: 0,
            conditions: apiserver::models::AssignmentRuleConditions::new(
                None, None, None, None, None,
            ),
            start_date: None,
            end_date: None,
        }
    }

    #[must_use]
    pub const fn with_workspace(mut self, ws_id: Uuid) -> Self {
        self.workspace_id = Some(ws_id);
        self
    }

    #[must_use]
    pub const fn with_pattern(mut self, pattern_id: Uuid) -> Self {
        self.pattern_id = Some(pattern_id);
        self
    }

    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    #[must_use]
    pub const fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    #[must_use]
    pub fn with_conditions(
        mut self,
        conditions: apiserver::models::AssignmentRuleConditions,
    ) -> Self {
        self.conditions = conditions;
        self
    }

    #[must_use]
    pub const fn with_start_date(mut self, date: chrono::NaiveDate) -> Self {
        self.start_date = Some(date);
        self
    }

    #[must_use]
    pub const fn with_end_date(mut self, date: chrono::NaiveDate) -> Self {
        self.end_date = Some(date);
        self
    }

    pub async fn create(self) -> Uuid {
        crate::test_query_scalar_uuid!(
            &self.diesel_pools.api,
            "INSERT INTO assignment_rules (workspace_id, pattern_id, name, priority, conditions, start_date, end_date) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            diesel::sql_types::Uuid, self.workspace_id.expect("WS ID required"),
            diesel::sql_types::Uuid, self.pattern_id.expect("Pattern ID required"),
            diesel::sql_types::Text, self.name,
            diesel::sql_types::Integer, self.priority,
            diesel::sql_types::Jsonb, serde_json::to_value(&self.conditions).unwrap(),
            diesel::sql_types::Nullable<diesel::sql_types::Date>, self.start_date,
            diesel::sql_types::Nullable<diesel::sql_types::Date>, self.end_date
        )
    }
}
