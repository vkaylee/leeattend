use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct TestFactory {
    pub pool: PgPool,
}

impl TestFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn user(&self) -> UserBuilder {
        UserBuilder::new(self.pool.clone())
    }

    pub fn workspace(&self) -> WorkspaceBuilder {
        WorkspaceBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    pub fn agent(&self) -> AgentBuilder {
        AgentBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    pub fn employee(&self) -> EmployeeBuilder {
        EmployeeBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    pub fn shift(&self) -> ShiftBuilder {
        ShiftBuilder::new(self.pool.clone())
    }

    #[allow(dead_code)]
    pub fn time_entry(&self) -> TimeEntryBuilder {
        TimeEntryBuilder::new(self.pool.clone())
    }
}

pub struct UserBuilder {
    pool: PgPool,
    email: String,
    full_name: Option<String>,
    password_hash: String,
}

impl UserBuilder {
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
    pub fn with_email(mut self, email: &str) -> Self {
        self.email = email.to_string();
        self
    }

    #[allow(dead_code)]
    pub fn with_full_name(mut self, name: &str) -> Self {
        self.full_name = Some(name.to_string());
        self
    }

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

pub struct WorkspaceBuilder {
    pool: PgPool,
    name: String,
}

impl WorkspaceBuilder {
    pub fn new(pool: PgPool) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            pool,
            name: format!("Workspace {}", &id[..8]),
        }
    }

    #[allow(dead_code)]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub async fn create(self) -> Uuid {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            sqlx::query_scalar("INSERT INTO workspaces (name) VALUES ($1) RETURNING id")
                .bind(self.name)
                .fetch_one(&self.pool)
                .await
                .unwrap()
        })
        .await
        .expect("Workspace creation timed out")
    }

    pub async fn create_with_owner(self, user_id: Uuid) -> Uuid {
        let pool = self.pool.clone();
        let ws_id = self.create().await;
        sqlx::query(
            "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, $3)",
        )
        .bind(ws_id)
        .bind(user_id)
        .bind("owner")
        .execute(&pool)
        .await
        .unwrap();
        ws_id
    }
}

impl TestFactory {
    #[allow(dead_code)]
    pub async fn add_member(&self, ws_id: Uuid, user_id: Uuid, role: &str) {
        sqlx::query(
            "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, $3)",
        )
        .bind(ws_id)
        .bind(user_id)
        .bind(role)
        .execute(&self.pool)
        .await
        .unwrap();
    }

    #[allow(dead_code)]
    pub async fn add_extra_role(
        &self,
        ws_id: Uuid,
        user_id: Uuid,
        role: &str,
        expires_at: DateTime<Utc>,
    ) {
        sqlx::query("INSERT INTO member_extra_roles (workspace_id, user_id, role, expires_at) VALUES ($1, $2, $3, $4) RETURNING *")
            .bind(ws_id)
            .bind(user_id)
            .bind(role)
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
    first_name: String,
    last_name: String,
    employee_id_number: String,
    hire_date: chrono::NaiveDate,
}

#[allow(dead_code)]
impl EmployeeBuilder {
    pub fn new(pool: PgPool) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            pool,
            workspace_id: None,
            user_id: None,
            first_name: "Test".to_string(),
            last_name: "Employee".to_string(),
            employee_id_number: format!("EMP-{}", &id[..8]),
            hire_date: chrono::Utc::now().date_naive(),
        }
    }

    #[allow(dead_code)]
    pub fn with_workspace(mut self, ws_id: Uuid) -> Self {
        self.workspace_id = Some(ws_id);
        self
    }

    #[allow(dead_code)]
    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    #[allow(dead_code)]
    pub fn with_employee_id_number(mut self, id_num: &str) -> Self {
        self.employee_id_number = id_num.to_string();
        self
    }

    pub async fn create(self) -> Uuid {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            sqlx::query_scalar(
                "INSERT INTO employees (workspace_id, user_id, first_name, last_name, employee_id_number, hire_date) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
            )
            .bind(self.workspace_id.expect("Workspace ID is required for Employee"))
            .bind(self.user_id)
            .bind(self.first_name)
            .bind(self.last_name)
            .bind(self.employee_id_number)
            .bind(self.hire_date)
            .fetch_one(&self.pool)
            .await
            .unwrap()
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
}

#[allow(dead_code)]
impl ShiftBuilder {
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
        }
    }

    #[allow(dead_code)]
    pub fn with_workspace(mut self, ws_id: Uuid) -> Self {
        self.workspace_id = Some(ws_id);
        self
    }

    #[allow(dead_code)]
    pub fn with_times(mut self, start: chrono::NaiveTime, end: chrono::NaiveTime) -> Self {
        self.start_time = start;
        self.end_time = end;
        self
    }

    #[allow(dead_code)]
    pub fn with_break_duration(mut self, minutes: i32) -> Self {
        self.break_duration_minutes = minutes;
        self
    }

    #[allow(dead_code)]
    pub fn with_grace_period(mut self, minutes: i32) -> Self {
        self.early_punch_in_grace_period_minutes = minutes;
        self
    }

    pub async fn create(self) -> Uuid {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            sqlx::query_scalar(
                "INSERT INTO shifts (workspace_id, name, start_time, end_time, break_duration_minutes, break_is_overtime, early_punch_in_grace_period_minutes) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            )
            .bind(self.workspace_id.expect("Workspace ID is required for Shift"))
            .bind(self.name)
            .bind(self.start_time)
            .bind(self.end_time)
            .bind(self.break_duration_minutes)
            .bind(self.break_is_overtime)
            .bind(self.early_punch_in_grace_period_minutes)
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
    pub fn with_employee(mut self, employee_id: Uuid) -> Self {
        self.employee_id = Some(employee_id);
        self
    }

    #[allow(dead_code)]
    pub fn with_punch_in(mut self, time: DateTime<Utc>) -> Self {
        self.punch_in_time = time;
        self
    }

    #[allow(dead_code)]
    pub fn with_punch_out(mut self, time: DateTime<Utc>) -> Self {
        self.punch_out_time = Some(time);
        self
    }

    #[allow(dead_code)]
    pub fn with_is_break_overtime(mut self, val: bool) -> Self {
        self.is_break_overtime = val;
        self
    }

    pub async fn create(self) -> Uuid {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            sqlx::query_scalar(
                "INSERT INTO time_entries (employee_id, punch_in_time, punch_out_time, is_break_overtime, entry_type) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            )
            .bind(self.employee_id.expect("Employee ID is required for TimeEntry"))
            .bind(self.punch_in_time)
            .bind(self.punch_out_time)
            .bind(self.is_break_overtime)
            .bind(self.entry_type)
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
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    #[allow(dead_code)]
    pub fn with_token(mut self, token: &str) -> Self {
        self.token = token.to_string();
        self
    }

    #[allow(dead_code)]
    pub fn with_workspace(mut self, ws_id: Uuid) -> Self {
        self.workspace_id = Some(ws_id);
        self
    }

    #[allow(dead_code)]
    pub fn with_trust(mut self, trusted: bool) -> Self {
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
            sqlx::query("DELETE FROM users CASCADE")
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
