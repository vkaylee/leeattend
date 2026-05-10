use apiserver::models::LeaveStatus;
use chrono::{DateTime, Datelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct TestFactory {
    pub pool: PgPool,
}

impl TestFactory {
    #[allow(dead_code)]
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn user(&self) -> UserBuilder {
        UserBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn workspace(&self) -> WorkspaceBuilder {
        WorkspaceBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn agent(&self) -> AgentBuilder {
        AgentBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn employee(&self) -> EmployeeBuilder {
        EmployeeBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn shift(&self) -> ShiftBuilder {
        ShiftBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn time_entry(&self) -> TimeEntryBuilder {
        TimeEntryBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn leave_request(&self) -> LeaveRequestBuilder {
        LeaveRequestBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn leave_balance(&self) -> LeaveBalanceBuilder {
        LeaveBalanceBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn leave_policy(&self) -> LeavePolicyBuilder {
        LeavePolicyBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn department(&self) -> DepartmentBuilder {
        DepartmentBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn assignment_rule(&self) -> AssignmentRuleBuilder {
        AssignmentRuleBuilder::new(self.pool.clone())
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
    pool: PgPool,
    email: String,
    full_name: Option<String>,
    password_hash: String,
}

impl UserBuilder {
    #[allow(dead_code)]
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            pool,
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
            sqlx::query_scalar(
                "INSERT INTO users (email, full_name, password_hash) VALUES ($1, $2, $3) RETURNING id",
            )
            .bind(self.email)
            .bind(self.full_name)
            .bind(self.password_hash)
            .fetch_one(&self.pool)
            .await
            .unwrap()
        }).await.expect("User creation timed out")
    }
}

#[allow(dead_code)]
pub struct WorkspaceBuilder {
    pool: PgPool,
    name: String,
    timezone: String,
}

impl WorkspaceBuilder {
    #[allow(dead_code)]
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            pool,
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
            let ws_id: Uuid = sqlx::query_scalar(
                "INSERT INTO workspaces (name, timezone) VALUES ($1, $2) RETURNING id",
            )
            .bind(self.name)
            .bind(self.timezone)
            .fetch_one(&self.pool)
            .await
            .unwrap();

            // Seed default leave types for tests
            sqlx::query(
                "INSERT INTO leave_types (workspace_id, code, name, is_paid) VALUES 
                ($1, 'vacation', 'Vacation', true),
                ($1, 'sick', 'Sick Leave', true),
                ($1, 'personal', 'Personal Leave', true),
                ($1, 'unpaid', 'Unpaid Leave', false)",
            )
            .bind(ws_id)
            .execute(&self.pool)
            .await
            .unwrap();

            // Seed default attendance policy for tests
            let policy_id: Uuid = sqlx::query_scalar(
                "INSERT INTO attendance_policies (workspace_id, name, is_default) VALUES ($1, 'Default Test Policy', true) RETURNING id"
            )
            .bind(ws_id)
            .fetch_one(&self.pool)
            .await
            .unwrap();

            let rules_json = serde_json::json!({
                "work_unit_minutes": 30,
                "rounding_policy": "floor"
            });
            sqlx::query(
                "INSERT INTO attendance_policy_versions (policy_id, rules, valid_from) VALUES ($1, $2, '2000-01-01 00:00:00Z')"
            )
            .bind(policy_id)
            .bind(rules_json)
            .execute(&self.pool)
            .await
            .unwrap();

            ws_id
        })
        .await
        .expect("Workspace creation timed out")
    }

    #[allow(dead_code)]
    pub async fn create_with_owner(self, user_id: Uuid) -> Uuid {
        let pool = self.pool.clone();
        let ws_id = self.create().await;
        sqlx::query(
            "INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ($1, $2, $3)",
        )
        .bind(ws_id)
        .bind(user_id)
        .bind(Uuid::parse_str("a0000000-0000-0000-0000-000000000001").unwrap())
        .execute(&pool)
        .await
        .unwrap();
        ws_id
    }
}

impl TestFactory {
    #[allow(dead_code)]
    pub async fn add_member(&self, ws_id: Uuid, user_id: Uuid, role_id: Uuid) {
        sqlx::query(
            "INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ($1, $2, $3)",
        )
        .bind(ws_id)
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .unwrap();
    }

    #[allow(dead_code)]
    pub async fn add_extra_role(
        &self,
        ws_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
        expires_at: DateTime<Utc>,
    ) {
        sqlx::query("INSERT INTO member_extra_roles (workspace_id, user_id, role_id, expires_at) VALUES ($1, $2, $3, $4) RETURNING *")
            .bind(ws_id)
            .bind(user_id)
            .bind(role_id)
            .bind(expires_at)
            .execute(&self.pool)
            .await
            .unwrap();
    }
}

// --- Employee Builder ---
#[allow(dead_code)]
pub struct EmployeeBuilder {
    pool: PgPool,
    workspace_id: Option<Uuid>,
    user_id: Option<Uuid>,
    full_name: String,
    employee_id_number: String,
    hire_date: chrono::NaiveDate,
}

#[allow(dead_code)]
impl EmployeeBuilder {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            pool,
            workspace_id: None,
            user_id: None,
            full_name: "Test Employee".to_string(),
            employee_id_number: format!("EMP-{}", Uuid::new_v4().as_u128() % 1_000_000_000),
            hire_date: chrono::Utc::now().date_naive(),
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
            let mut tx = self.pool.begin().await.unwrap();

            let normalized_id = apiserver::utils::normalize_employee_id(&self.employee_id_number);
            let id: Uuid = sqlx::query_scalar(
                "INSERT INTO employees (workspace_id, user_id, full_name, employee_id_number, original_hire_date, hire_date) VALUES ($1, $2, $3, $4, $5, $5) RETURNING id",
            )
            .bind(self.workspace_id.expect("Workspace ID is required for Employee"))
            .bind(self.user_id)
            .bind(self.full_name)
            .bind(normalized_id)
            .bind(self.hire_date)
            .fetch_one(&mut *tx)
            .await
            .unwrap();

            tx.commit().await.unwrap();
            id
        }).await.expect("Employee creation timed out")
    }
}

// --- Shift Builder ---
#[allow(dead_code)]
pub struct ShiftBuilder {
    pool: PgPool,
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
    pub fn new(pool: PgPool) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            pool,
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
            sqlx::query_scalar(
                "INSERT INTO shifts (workspace_id, name, start_time, end_time, break_duration_minutes, break_is_overtime, early_punch_in_grace_period_minutes, allow_early_ot) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
            )
            .bind(self.workspace_id.expect("Workspace ID is required for Shift"))
            .bind(self.name)
            .bind(self.start_time)
            .bind(self.end_time)
            .bind(self.break_duration_minutes)
            .bind(self.break_is_overtime)
            .bind(self.early_punch_in_grace_period_minutes)
            .bind(self.allow_early_ot)
            .fetch_one(&self.pool)
            .await
            .unwrap()
        }).await.expect("Shift creation timed out")
    }

    pub async fn assign_to_employee(
        self,
        employee_id: Uuid,
        start_date: chrono::NaiveDate,
    ) -> Uuid {
        let pool = self.pool.clone();
        let shift_id = self.create().await;
        sqlx::query(
            "INSERT INTO employee_shifts (employee_id, shift_id, start_date) VALUES ($1, $2, $3)",
        )
        .bind(employee_id)
        .bind(shift_id)
        .bind(start_date)
        .execute(&pool)
        .await
        .unwrap();
        shift_id
    }
}

// --- TimeEntry Builder ---
#[allow(dead_code)]
pub struct TimeEntryBuilder {
    pool: PgPool,
    employee_id: Option<Uuid>,
    punch_in_time: DateTime<Utc>,
    punch_out_time: Option<DateTime<Utc>>,
    is_break_overtime: bool,
    entry_type: String,
}

#[allow(dead_code)]
impl TimeEntryBuilder {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
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
            let workspace_id: Uuid = sqlx::query_scalar("SELECT workspace_id FROM employees WHERE id = $1")
                .bind(employee_id)
                .fetch_one(&self.pool)
                .await
                .unwrap();

            sqlx::query_scalar(
                "INSERT INTO time_entries (employee_id, workspace_id, punch_in_time, punch_out_time, is_break_overtime, entry_type, duration_minutes) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            )
            .bind(employee_id)
            .bind(workspace_id)
            .bind(self.punch_in_time)
            .bind(self.punch_out_time)
            .bind(self.is_break_overtime)
            .bind(self.entry_type)
            .bind(duration_minutes)
            .fetch_one(&self.pool)
            .await
            .unwrap()
        }).await.expect("TimeEntry creation timed out")
    }
}

#[allow(dead_code)]
pub struct AgentBuilder {
    pool: PgPool,
    id: String,
    name: String,
    token: String,
    workspace_id: Option<Uuid>,
    is_trusted: bool,
}

#[allow(dead_code)]
impl AgentBuilder {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            pool,
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
            sqlx::query(
                "INSERT INTO authorized_agents (id, name, token, workspace_id, is_trusted) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(&self.id)
            .bind(&self.name)
            .bind(&self.token)
            .bind(ws_id)
            .bind(self.is_trusted)
            .execute(&self.pool)
            .await
            .unwrap();
            self.id
        }).await.expect("Agent creation timed out")
    }
}

impl TestFactory {
    #[allow(dead_code)]
    pub async fn cleanup(&self) {
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            sqlx::query("DELETE FROM time_entries CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM employee_shifts CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM shifts CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM employees CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM member_extra_roles CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM workspace_members CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM devices CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM authorized_agents CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM assignment_rules CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM leave_accrual_logs CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM leave_policy_assignments CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM leave_policy_definitions CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM leave_requests CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM leave_balances CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM users CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM attendance_policy_versions CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM attendance_policies CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM workspaces CASCADE")
                .execute(&self.pool)
                .await
                .unwrap();
        })
        .await
        .expect("Cleanup timed out");
    }
}

// --- Leave Request Builder ---
#[allow(dead_code)]
pub struct LeaveRequestBuilder {
    pool: PgPool,
    employee_id: Option<Uuid>,
    leave_type_id: Option<Uuid>,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    status: LeaveStatus,
}

#[allow(dead_code)]
impl LeaveRequestBuilder {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
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
        let leave_type_id = if let Some(id) = self.leave_type_id {
            id
        } else {
            let emp_id = self
                .employee_id
                .expect("Employee ID is required for LeaveRequest");
            sqlx::query_scalar!(
                "SELECT id FROM leave_types WHERE workspace_id = (SELECT workspace_id FROM employees WHERE id = $1) AND code = 'vacation'",
                emp_id
            )
            .fetch_one(&self.pool)
            .await
            .expect("Default leave type 'vacation' not found")
        };
        sqlx::query(
            "INSERT INTO leave_requests (id, employee_id, leave_type_id, start_date, end_date, status) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(self.employee_id.expect("Employee ID is required for LeaveRequest"))
        .bind(leave_type_id)
        .bind(self.start_date)
        .bind(self.end_date)
        .bind(self.status)
        .execute(&self.pool)
        .await
        .unwrap();
        id
    }
}

// --- Leave Balance Builder ---
#[allow(dead_code)]
pub struct LeaveBalanceBuilder {
    pool: PgPool,
    employee_id: Option<Uuid>,
    leave_type_id: Option<Uuid>,
    year: i32,
    entitlement: i32,
    carried_over: i32,
}

#[allow(dead_code)]
impl LeaveBalanceBuilder {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
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
        let leave_type_id = if let Some(id) = self.leave_type_id {
            id
        } else {
            let emp_id = self
                .employee_id
                .expect("Employee ID is required for LeaveBalance");
            sqlx::query_scalar!(
                "SELECT id FROM leave_types WHERE workspace_id = (SELECT workspace_id FROM employees WHERE id = $1) AND code = 'vacation'",
                emp_id
            )
            .fetch_one(&self.pool)
            .await
            .expect("Default leave type 'vacation' not found")
        };
        sqlx::query(
            "INSERT INTO leave_balances (id, employee_id, leave_type_id, year, entitlement, carried_over) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(self.employee_id.expect("Employee ID is required for LeaveBalance"))
        .bind(leave_type_id)
        .bind(self.year)
        .bind(self.entitlement)
        .bind(self.carried_over)
        .execute(&self.pool)
        .await
        .unwrap();
        id
    }
}

// --- Leave Policy Builder ---
#[allow(dead_code)]
pub struct LeavePolicyBuilder {
    pool: PgPool,
    workspace_id: Option<Uuid>,
    name: String,
    rule_config: serde_json::Value,
}

#[allow(dead_code)]
impl LeavePolicyBuilder {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
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
        let pool = self.pool.clone();

        // Fetch vacation ID as default
        let leave_type_id = sqlx::query_scalar!(
            "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
            ws_id
        )
        .fetch_one(&pool)
        .await
        .expect("Default leave type 'vacation' not found - check WorkspaceBuilder seeding");

        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            sqlx::query_scalar(
                "INSERT INTO leave_policy_definitions (workspace_id, name, rule_config, leave_type_id) VALUES ($1, $2, $3, $4) RETURNING id",
            )
            .bind(ws_id)
            .bind(self.name)
            .bind(self.rule_config)
            .bind(leave_type_id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
        }).await.expect("Policy creation timed out")
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
        sqlx::query_scalar(
            "INSERT INTO leave_policy_assignments (workspace_id, policy_id, assignee_type, assignee_id, effective_from, effective_to) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(workspace_id)
        .bind(policy_id)
        .bind(assignee_type)
        .bind(assignee_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }
}

// --- Department Builder ---
#[allow(dead_code)]
pub struct DepartmentBuilder {
    pool: PgPool,
    workspace_id: Option<Uuid>,
    name: String,
    description: Option<String>,
    manager_id: Option<Uuid>,
}

#[allow(dead_code)]
impl DepartmentBuilder {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            pool,
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
        sqlx::query_scalar(
            "INSERT INTO departments (workspace_id, name, description, manager_id) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(self.workspace_id.expect("Workspace ID required"))
        .bind(self.name)
        .bind(self.description)
        .bind(self.manager_id)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }
}

// --- Assignment Rule Builder ---
#[allow(dead_code)]
pub struct AssignmentRuleBuilder {
    pool: PgPool,
    workspace_id: Option<Uuid>,
    pattern_id: Option<Uuid>,
    name: String,
    priority: i32,
    conditions: apiserver::models::AssignmentRuleConditions,
}

#[allow(dead_code)]
impl AssignmentRuleBuilder {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            workspace_id: None,
            pattern_id: None,
            name: "Test Rule".to_string(),
            priority: 0,
            conditions: apiserver::models::AssignmentRuleConditions {
                department_id: None,
                employee_id_numbers: None,
                tags: None,
            },
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

    pub async fn create(self) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO assignment_rules (workspace_id, pattern_id, name, priority, conditions) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            self.workspace_id.expect("WS ID required"),
            self.pattern_id.expect("Pattern ID required"),
            self.name,
            self.priority,
            serde_json::to_value(&self.conditions).unwrap()
        )
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }
}
