use crate::errors::AppError;
use crate::models::{OvertimePolicy, OvertimeRule};
use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: OvertimePolicyRepositoryTrait — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait OvertimePolicyRepositoryTrait: super::PoolBacked + Send + Sync {
    async fn create_policy_definition(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
    ) -> Result<OvertimePolicy, AppError>;
    async fn get_policy_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<OvertimePolicy>, AppError>;
    async fn get_policy_with_rules(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<(OvertimePolicy, Vec<OvertimeRule>)>, AppError>;
    async fn add_rule(&self, rule: &OvertimeRule) -> Result<OvertimeRule, AppError>;
    async fn resolve_policy_for_employee(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Uuid>, AppError>;
    async fn create_policy_assignment(
        &self,
        workspace_id: Uuid,
        assignee_type: &str,
        assignee_id: Option<Uuid>,
        policy_id: Uuid,
        effective_from: NaiveDate,
    ) -> Result<(), AppError>;
}

// ---------------------------------------------------------------------------
// SqlxOvertimePolicyRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxOvertimePolicyRepository {
    pub pool: PgPool,
}

impl SqlxOvertimePolicyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OvertimePolicyRepositoryTrait for SqlxOvertimePolicyRepository {
    async fn create_policy_definition(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
    ) -> Result<OvertimePolicy, AppError> {
        OvertimePolicyRepository::create_policy_definition(
            &self.pool,
            workspace_id,
            name,
            description,
        )
        .await
        .map_err(Into::into)
    }
    async fn get_policy_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<OvertimePolicy>, AppError> {
        OvertimePolicyRepository::get_policy_definitions(&self.pool, workspace_id).await.map_err(Into::into)
    }
    async fn get_policy_with_rules(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<(OvertimePolicy, Vec<OvertimeRule>)>, AppError> {
        OvertimePolicyRepository::get_policy_with_rules(&self.pool, workspace_id, id).await.map_err(Into::into)
    }
    async fn add_rule(&self, rule: &OvertimeRule) -> Result<OvertimeRule, AppError> {
        OvertimePolicyRepository::add_rule(&self.pool, rule).await.map_err(Into::into)
    }
    async fn resolve_policy_for_employee(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Uuid>, AppError> {
        OvertimePolicyRepository::resolve_policy_for_employee(
            &self.pool,
            employee_id,
            workspace_id,
            date,
        )
        .await
        .map_err(Into::into)
    }
    async fn create_policy_assignment(
        &self,
        workspace_id: Uuid,
        assignee_type: &str,
        assignee_id: Option<Uuid>,
        policy_id: Uuid,
        effective_from: NaiveDate,
    ) -> Result<(), AppError> {
        OvertimePolicyRepository::create_policy_assignment(
            &self.pool,
            workspace_id,
            assignee_type,
            assignee_id,
            policy_id,
            effective_from,
        )
        .await
        .map_err(Into::into)
    }
}

/// Backwards-compatible static method repository.
pub struct OvertimePolicyRepository;

impl OvertimePolicyRepository {
    pub async fn create_policy_definition(
        pool: &PgPool,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
    ) -> Result<OvertimePolicy, sqlx::Error> {
        sqlx::query_as!(
            OvertimePolicy,
            r#"
            INSERT INTO overtime_policy_definitions (workspace_id, name, description)
            VALUES ($1, $2, $3)
            RETURNING id, workspace_id, name, description, created_at, updated_at
            "#,
            workspace_id,
            name,
            description
        )
        .fetch_one(pool)
        .await
    }

    pub async fn get_policy_definitions(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Vec<OvertimePolicy>, sqlx::Error> {
        sqlx::query_as!(
            OvertimePolicy,
            r#"
            SELECT id, workspace_id, name, description, created_at, updated_at
            FROM overtime_policy_definitions
            WHERE workspace_id = $1
            ORDER BY created_at DESC
            "#,
            workspace_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn get_policy_with_rules(
        pool: &PgPool,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<(OvertimePolicy, Vec<OvertimeRule>)>, sqlx::Error> {
        let policy = sqlx::query_as!(
            OvertimePolicy,
            r#"
            SELECT id, workspace_id, name, description, created_at, updated_at
            FROM overtime_policy_definitions
            WHERE id = $1 AND workspace_id = $2
            "#,
            id,
            workspace_id
        )
        .fetch_optional(pool)
        .await?;

        if let Some(p) = policy {
            let rules = sqlx::query_as!(
                OvertimeRule,
                r#"
                SELECT id, policy_id, priority, multiplier, time_start, time_end, days_of_week, is_holiday, created_at, updated_at
                FROM overtime_rules
                WHERE policy_id = $1
                ORDER BY priority DESC
                "#,
                id
            )
            .fetch_all(pool)
            .await?;
            Ok(Some((p, rules)))
        } else {
            Ok(None)
        }
    }

    pub async fn add_rule(pool: &PgPool, rule: &OvertimeRule) -> Result<OvertimeRule, sqlx::Error> {
        sqlx::query_as!(
            OvertimeRule,
            r#"
            INSERT INTO overtime_rules (policy_id, priority, multiplier, time_start, time_end, days_of_week, is_holiday)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, policy_id, priority, multiplier, time_start, time_end, days_of_week, is_holiday, created_at, updated_at
            "#,
            rule.policy_id,
            rule.priority,
            rule.multiplier,
            rule.time_start,
            rule.time_end,
            rule.days_of_week,
            rule.is_holiday
        )
        .fetch_one(pool)
        .await
    }

    pub async fn resolve_policy_for_employee(
        pool: &PgPool,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        // 1. Employee Override
        let emp_policy = sqlx::query!(
            r#"
            SELECT policy_id FROM overtime_policy_assignments
            WHERE workspace_id = $1 AND assignee_type = 'employee' AND assignee_id = $2
            AND effective_from <= $3 AND (effective_to IS NULL OR effective_to >= $3)
            ORDER BY created_at DESC LIMIT 1
            "#,
            workspace_id,
            employee_id,
            date
        )
        .fetch_optional(pool)
        .await?;

        if let Some(r) = emp_policy {
            return Ok(Some(r.policy_id));
        }

        // 2. Department Override
        let dept_policy = sqlx::query!(
            r#"
            SELECT opa.policy_id 
            FROM overtime_policy_assignments opa
            JOIN employee_department_history edh ON edh.department_id = opa.assignee_id
            WHERE opa.workspace_id = $1 AND opa.assignee_type = 'department' 
            AND edh.employee_id = $2 AND edh.effective_date <= $3
            AND opa.effective_from <= $3 AND (opa.effective_to IS NULL OR opa.effective_to >= $3)
            ORDER BY edh.effective_date DESC, opa.created_at DESC LIMIT 1
            "#,
            workspace_id,
            employee_id,
            date
        )
        .fetch_optional(pool)
        .await?;

        if let Some(r) = dept_policy {
            return Ok(Some(r.policy_id));
        }

        // 3. Workspace Default
        let ws_policy: Option<_> = sqlx::query!(
            r#"
            SELECT policy_id FROM overtime_policy_assignments
            WHERE workspace_id = $1 AND assignee_type = 'workspace'
            AND effective_from <= $2 AND (effective_to IS NULL OR effective_to >= $2)
            ORDER BY created_at DESC LIMIT 1
            "#,
            workspace_id,
            date
        )
        .fetch_optional(pool)
        .await?;

        Ok(ws_policy.map(|r| r.policy_id as Uuid))
    }

    pub async fn create_policy_assignment(
        pool: &PgPool,
        workspace_id: Uuid,
        assignee_type: &str,
        assignee_id: Option<Uuid>,
        policy_id: Uuid,
        effective_from: NaiveDate,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO overtime_policy_assignments (workspace_id, assignee_type, assignee_id, policy_id, effective_from)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            workspace_id,
            assignee_type,
            assignee_id,
            policy_id,
            effective_from
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl super::PoolBacked for SqlxOvertimePolicyRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
