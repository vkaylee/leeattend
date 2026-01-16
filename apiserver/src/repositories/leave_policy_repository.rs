use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use crate::models::{LeavePolicyDefinition, LeavePolicyAssignment, LeavePolicyConfig};

pub async fn create_policy_definition(
    pool: &PgPool,
    workspace_id: Uuid,
    name: &str,
    description: Option<&str>,
    rule_config: LeavePolicyConfig,
) -> Result<LeavePolicyDefinition, sqlx::Error> {
    sqlx::query_as!(
        LeavePolicyDefinition,
        r#"
        INSERT INTO leave_policy_definitions (workspace_id, name, description, rule_config)
        VALUES ($1, $2, $3, $4)
        RETURNING id, workspace_id, name, description, rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", created_at as "created_at!", updated_at as "updated_at!"
        "#,
        workspace_id,
        name,
        description,
        sqlx::types::Json(rule_config) as _
    )
    .fetch_one(pool)
    .await
}

pub async fn get_policy_definitions(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<LeavePolicyDefinition>, sqlx::Error> {
    sqlx::query_as!(
        LeavePolicyDefinition,
        r#"
        SELECT id, workspace_id, name, description, rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", created_at as "created_at!", updated_at as "updated_at!"
        FROM leave_policy_definitions
        WHERE workspace_id = $1
        ORDER BY created_at DESC
        "#,
        workspace_id
    )
    .fetch_all(pool)
    .await
}

pub async fn update_policy_definition(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    rule_config: Option<LeavePolicyConfig>,
) -> Result<LeavePolicyDefinition, sqlx::Error> {
    sqlx::query_as!(
        LeavePolicyDefinition,
        r#"
        UPDATE leave_policy_definitions
        SET name = COALESCE($3, name),
            description = COALESCE($4, description),
            rule_config = COALESCE($5, rule_config),
            updated_at = NOW()
        WHERE id = $1 AND workspace_id = $2
        RETURNING id, workspace_id, name, description, rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", created_at as "created_at!", updated_at as "updated_at!"
        "#,
        id,
        workspace_id,
        name,
        description,
        rule_config.map(|c| sqlx::types::Json(c)) as _
    )
    .fetch_one(pool)
    .await
}

pub async fn delete_policy_definition(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
) -> Result<u64, sqlx::Error> {
    // First delete assignments to avoid FK constraint issues
    sqlx::query!(
        r#"
        DELETE FROM leave_policy_assignments
        WHERE policy_id = $1 AND workspace_id = $2
        "#,
        id,
        workspace_id
    )
    .execute(pool)
    .await?;

    let result = sqlx::query!(
        r#"
        DELETE FROM leave_policy_definitions
        WHERE id = $1 AND workspace_id = $2
        "#,
        id,
        workspace_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn create_policy_assignment(
    pool: &PgPool,
    assignment: &LeavePolicyAssignment,
) -> Result<LeavePolicyAssignment, sqlx::Error> {
    sqlx::query_as!(
        LeavePolicyAssignment,
        r#"
        INSERT INTO leave_policy_assignments (workspace_id, assignee_type, assignee_id, policy_id, effective_from, effective_to)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, workspace_id, assignee_type, assignee_id, policy_id, effective_from, effective_to, created_at as "created_at!"
        "#,
        assignment.workspace_id,
        assignment.assignee_type,
        assignment.assignee_id,
        assignment.policy_id,
        assignment.effective_from,
        assignment.effective_to
    )
    .fetch_one(pool)
    .await
}

/// Finds the active policy definition for a specific entity (Employee, Dept, Workspace) on a given date.
/// It respects the hierarchy: Employee > Department > Workspace (Global).
pub async fn find_active_policy_for_employee(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    department_id: Option<Uuid>,
    date: NaiveDate,
) -> Result<Option<LeavePolicyDefinition>, sqlx::Error> {
    // Priority 1: Employee specific assignment
    let employee_policy = sqlx::query_as!(
        LeavePolicyDefinition,
        r#"
        SELECT d.id, d.workspace_id, d.name, d.description, d.rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", d.created_at as "created_at!", d.updated_at as "updated_at!"
        FROM leave_policy_assignments a
        JOIN leave_policy_definitions d ON a.policy_id = d.id
        WHERE a.workspace_id = $1
          AND a.assignee_type = 'employee'
          AND a.assignee_id = $2
          AND a.effective_from <= $3
          AND (a.effective_to IS NULL OR a.effective_to > $3)
        LIMIT 1
        "#,
        workspace_id,
        employee_id,
        date
    )
    .fetch_optional(pool)
    .await?;

    if let Some(p) = employee_policy {
        return Ok(Some(p));
    }

    // Priority 2: Department assignment
    if let Some(dept_id) = department_id {
        let dept_policy = sqlx::query_as!(
            LeavePolicyDefinition,
            r#"
            SELECT d.id, d.workspace_id, d.name, d.description, d.rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", d.created_at as "created_at!", d.updated_at as "updated_at!"
            FROM leave_policy_assignments a
            JOIN leave_policy_definitions d ON a.policy_id = d.id
            WHERE a.workspace_id = $1
              AND a.assignee_type = 'department'
              AND a.assignee_id = $2
              AND a.effective_from <= $3
              AND (a.effective_to IS NULL OR a.effective_to > $3)
            LIMIT 1
            "#,
            workspace_id,
            dept_id,
            date
        )
        .fetch_optional(pool)
        .await?;

        if let Some(p) = dept_policy {
            return Ok(Some(p));
        }
    }

    // Priority 3: Global Workspace assignment
    let global_policy = sqlx::query_as!(
        LeavePolicyDefinition,
        r#"
        SELECT d.id, d.workspace_id, d.name, d.description, d.rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", d.created_at as "created_at!", d.updated_at as "updated_at!"
        FROM leave_policy_assignments a
        JOIN leave_policy_definitions d ON a.policy_id = d.id
        WHERE a.workspace_id = $1
          AND a.assignee_type = 'workspace'
          AND a.effective_from <= $2
          AND (a.effective_to IS NULL OR a.effective_to > $2)
        LIMIT 1
        "#,
        workspace_id,
        date
    )
    .fetch_optional(pool)
    .await?;

    Ok(global_policy)
}