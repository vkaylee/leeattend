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

    pub async fn cleanup(&self) {
        sqlx::query("DELETE FROM member_extra_roles")
            .execute(&self.pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM workspace_members")
            .execute(&self.pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM devices")
            .execute(&self.pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM authorized_agents")
            .execute(&self.pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM users")
            .execute(&self.pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM workspaces")
            .execute(&self.pool)
            .await
            .unwrap();
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
        sqlx::query_scalar(
            "INSERT INTO users (email, full_name, password_hash) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(self.email)
        .bind(self.full_name)
        .bind(self.password_hash)
        .fetch_one(&self.pool)
        .await
        .unwrap()
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

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub async fn create(self) -> Uuid {
        sqlx::query_scalar("INSERT INTO workspaces (name) VALUES ($1) RETURNING id")
            .bind(self.name)
            .fetch_one(&self.pool)
            .await
            .unwrap()
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
        sqlx::query("INSERT INTO member_extra_roles (workspace_id, user_id, role, expires_at) VALUES ($1, $2, $3, $4)")
            .bind(ws_id)
            .bind(user_id)
            .bind(role)
            .bind(expires_at)
            .execute(&self.pool)
            .await
            .unwrap();
    }
}
