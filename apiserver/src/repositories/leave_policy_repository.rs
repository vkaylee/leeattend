use crate::models::{LeavePolicyAssignment, LeavePolicyConfig, LeavePolicyDefinition};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_policy_definition(
    pool: &PgPool,
    workspace_id: Uuid,
    name: &str,
    description: Option<&str>,
    leave_type: &str,
    rule_config: LeavePolicyConfig,
) -> Result<LeavePolicyDefinition, sqlx::Error> {
    sqlx::query_as!(
        LeavePolicyDefinition,
        r#"
        INSERT INTO leave_policy_definitions (workspace_id, name, description, leave_type, rule_config)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, workspace_id, name, description, leave_type, rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", created_at as "created_at!", updated_at as "updated_at!"
        "#,
        workspace_id,
        name,
        description,
        leave_type,
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
        SELECT id, workspace_id, name, description, leave_type, rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", created_at as "created_at!", updated_at as "updated_at!"
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
    leave_type: Option<&str>,
    rule_config: Option<LeavePolicyConfig>,
) -> Result<LeavePolicyDefinition, sqlx::Error> {
    sqlx::query_as!(
        LeavePolicyDefinition,
        r#"
        UPDATE leave_policy_definitions
        SET name = COALESCE($3, name),
            description = COALESCE($4, description),
            leave_type = COALESCE($5, leave_type),
            rule_config = COALESCE($6, rule_config),
            updated_at = NOW()
        WHERE id = $1 AND workspace_id = $2
        RETURNING id, workspace_id, name, description, leave_type, rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", created_at as "created_at!", updated_at as "updated_at!"
        "#,
        id,
        workspace_id,
        name,
        description,
        leave_type,
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
    let mut tx = pool.begin().await?;

    // 1. Get the leave_type of the policy being assigned
    let leave_type = sqlx::query!(
        "SELECT leave_type FROM leave_policy_definitions WHERE id = $1",
        assignment.policy_id
    )
    .fetch_one(&mut *tx)
    .await?
    .leave_type;

    // 2. Auto-close existing assignments of the same type and category that overlap
    // Only close assignments that started before or on the new effective_from date
    // and are either still open (NULL) or end after the new effective_from.
    sqlx::query!(
        r#"
        UPDATE leave_policy_assignments a
        SET effective_to = $5
        FROM leave_policy_definitions d
        WHERE a.policy_id = d.id
          AND a.workspace_id = $1
          AND a.assignee_type = $2
          AND (a.assignee_id = $3 OR (a.assignee_id IS NULL AND $3 IS NULL))
          AND d.leave_type = $4
          AND a.effective_from < $6
          AND (a.effective_to IS NULL OR a.effective_to >= $6)
        "#,
        assignment.workspace_id,
        assignment.assignee_type,
        assignment.assignee_id,
        leave_type,
        assignment.effective_from.pred_opt(), // End one day before the new one starts
        assignment.effective_from
    )
    .execute(&mut *tx)
    .await?;

    // 3. Insert the new assignment
    let saved = sqlx::query_as!(
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
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(saved)
}

/// Finds the active policy definition and its assignment ID for a specific entity on a given date.
pub async fn find_active_assignment_for_employee(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    department_id: Option<Uuid>,
    date: NaiveDate,
) -> Result<Option<(LeavePolicyDefinition, Uuid)>, sqlx::Error> {
    // Priority 1: Employee specific assignment
    let employee_result = sqlx::query!(
        r#"
        SELECT d.id, d.workspace_id, d.name, d.description, d.leave_type, d.rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", d.created_at as "created_at!", d.updated_at as "updated_at!", a.id as "assignment_id!"
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

    if let Some(r) = employee_result {
        let def = LeavePolicyDefinition {
            id: r.id,
            workspace_id: r.workspace_id,
            name: r.name,
            description: r.description,
            leave_type: r.leave_type,
            rule_config: r.rule_config,
            created_at: r.created_at,
            updated_at: r.updated_at,
        };
        return Ok(Some((def, r.assignment_id)));
    }

    // Priority 2: Department assignment
    if let Some(dept_id) = department_id {
        let dept_result = sqlx::query!(
            r#"
            SELECT d.id, d.workspace_id, d.name, d.description, d.leave_type, d.rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", d.created_at as "created_at!", d.updated_at as "updated_at!", a.id as "assignment_id!"
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

        if let Some(r) = dept_result {
            let def = LeavePolicyDefinition {
                id: r.id,
                workspace_id: r.workspace_id,
                name: r.name,
                description: r.description,
                leave_type: r.leave_type,
                rule_config: r.rule_config,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            return Ok(Some((def, r.assignment_id)));
        }
    }

    // Priority 3: Global Workspace assignment
    let global_result = sqlx::query!(
        r#"
        SELECT d.id, d.workspace_id, d.name, d.description, d.leave_type, d.rule_config as "rule_config: sqlx::types::Json<LeavePolicyConfig>", d.created_at as "created_at!", d.updated_at as "updated_at!", a.id as "assignment_id!"
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

    if let Some(r) = global_result {
        let def = LeavePolicyDefinition {
            id: r.id,
            workspace_id: r.workspace_id,
            name: r.name,
            description: r.description,
            leave_type: r.leave_type,
            rule_config: r.rule_config,
            created_at: r.created_at,
            updated_at: r.updated_at,
        };
        return Ok(Some((def, r.assignment_id)));
    }

    Ok(None)
}
