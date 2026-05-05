use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{AttendancePolicyVersion, AttendanceRules};
use crate::repositories::PoolBacked;

#[async_trait::async_trait]
pub trait AttendancePolicyRepository: PoolBacked {
    async fn get_employee_effective_policy(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        effective_time: DateTime<Utc>,
    ) -> Result<Option<AttendancePolicyVersion>, sqlx::Error>;

    async fn get_employee_effective_policies_batch(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        effective_time: DateTime<Utc>,
    ) -> Result<std::collections::HashMap<Uuid, AttendancePolicyVersion>, sqlx::Error>;
}

pub struct SqlxAttendancePolicyRepository {
    pool: PgPool,
}

impl PoolBacked for SqlxAttendancePolicyRepository {
    fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait::async_trait]
impl AttendancePolicyRepository for SqlxAttendancePolicyRepository {
    async fn get_employee_effective_policy(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        effective_time: DateTime<Utc>,
    ) -> Result<Option<AttendancePolicyVersion>, sqlx::Error> {
        let row = sqlx::query_as!(
            AttendancePolicyVersion,
            r#"
            WITH dept_history AS (
                SELECT department_id
                FROM employee_department_history
                WHERE employee_id = $2
                  AND effective_date <= ($3::timestamptz)::date
                ORDER BY effective_date DESC
                LIMIT 1
            ),
            user_policy AS (
                SELECT ap.id as policy_id
                FROM attendance_policy_assignments apa
                JOIN attendance_policies ap ON ap.id = apa.policy_id
                WHERE apa.target_id = $2
                  AND apa.target_type = 'USER'
                  AND apa.valid_from <= $3
                  AND (apa.valid_to IS NULL OR apa.valid_to >= $3)
                  AND ap.workspace_id = $1
                ORDER BY apa.valid_from DESC
                LIMIT 1
            ),
            group_policy AS (
                SELECT ap.id as policy_id
                FROM attendance_policy_assignments apa
                JOIN attendance_policies ap ON ap.id = apa.policy_id
                JOIN dept_history dh ON dh.department_id = apa.target_id
                WHERE apa.target_type = 'GROUP'
                  AND apa.valid_from <= $3
                  AND (apa.valid_to IS NULL OR apa.valid_to >= $3)
                  AND ap.workspace_id = $1
                ORDER BY apa.valid_from DESC
                LIMIT 1
            ),
            default_policy AS (
                SELECT id as policy_id
                FROM attendance_policies
                WHERE workspace_id = $1 AND is_default = true
                LIMIT 1
            ),
            selected_policy AS (
                SELECT policy_id FROM user_policy
                UNION ALL
                SELECT policy_id FROM group_policy WHERE NOT EXISTS (SELECT 1 FROM user_policy)
                UNION ALL
                SELECT policy_id FROM default_policy WHERE NOT EXISTS (SELECT 1 FROM user_policy) AND NOT EXISTS (SELECT 1 FROM group_policy)
                LIMIT 1
            )
            SELECT 
                apv.id,
                apv.policy_id,
                apv.rules as "rules: sqlx::types::Json<AttendanceRules>",
                apv.valid_from,
                apv.valid_to,
                apv.created_by,
                apv.created_at
            FROM attendance_policy_versions apv
            JOIN selected_policy sp ON sp.policy_id = apv.policy_id
            WHERE apv.valid_from <= $3
              AND (apv.valid_to IS NULL OR apv.valid_to >= $3)
            ORDER BY apv.valid_from DESC
            LIMIT 1
            "#,
            workspace_id,
            employee_id,
            effective_time
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn get_employee_effective_policies_batch(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        effective_time: DateTime<Utc>,
    ) -> Result<std::collections::HashMap<Uuid, AttendancePolicyVersion>, sqlx::Error> {
        let mut result = std::collections::HashMap::new();

        if employee_ids.is_empty() {
            return Ok(result);
        }

        let rows = sqlx::query!(
            r#"
            WITH inputs AS (
                SELECT unnest($2::uuid[]) AS employee_id
            ),
            dept_history AS (
                SELECT i.employee_id, 
                    (
                        SELECT department_id
                        FROM employee_department_history
                        WHERE employee_id = i.employee_id
                          AND effective_date <= ($3::timestamptz)::date
                        ORDER BY effective_date DESC
                        LIMIT 1
                    ) as department_id
                FROM inputs i
            ),
            user_policy AS (
                SELECT i.employee_id,
                    (
                        SELECT ap.id
                        FROM attendance_policy_assignments apa
                        JOIN attendance_policies ap ON ap.id = apa.policy_id
                        WHERE apa.target_id = i.employee_id
                          AND apa.target_type = 'USER'
                          AND apa.valid_from <= $3
                          AND (apa.valid_to IS NULL OR apa.valid_to >= $3)
                          AND ap.workspace_id = $1
                        ORDER BY apa.valid_from DESC
                        LIMIT 1
                    ) as policy_id
                FROM inputs i
            ),
            group_policy AS (
                SELECT dh.employee_id,
                    (
                        SELECT ap.id
                        FROM attendance_policy_assignments apa
                        JOIN attendance_policies ap ON ap.id = apa.policy_id
                        WHERE apa.target_id = dh.department_id
                          AND apa.target_type = 'GROUP'
                          AND apa.valid_from <= $3
                          AND (apa.valid_to IS NULL OR apa.valid_to >= $3)
                          AND ap.workspace_id = $1
                        ORDER BY apa.valid_from DESC
                        LIMIT 1
                    ) as policy_id
                FROM dept_history dh
            ),
            default_policy AS (
                SELECT id as policy_id
                FROM attendance_policies
                WHERE workspace_id = $1 AND is_default = true
                LIMIT 1
            ),
            selected_policy AS (
                SELECT i.employee_id,
                    COALESCE(
                        up.policy_id,
                        gp.policy_id,
                        (SELECT policy_id FROM default_policy)
                    ) as policy_id
                FROM inputs i
                LEFT JOIN user_policy up ON up.employee_id = i.employee_id
                LEFT JOIN group_policy gp ON gp.employee_id = i.employee_id
            )
            SELECT 
                sp.employee_id,
                apv.id,
                apv.policy_id,
                apv.rules as "rules: sqlx::types::Json<AttendanceRules>",
                apv.valid_from,
                apv.valid_to,
                apv.created_by,
                apv.created_at
            FROM selected_policy sp
            INNER JOIN LATERAL (
                SELECT id, policy_id, rules, valid_from, valid_to, created_by, created_at
                FROM attendance_policy_versions v
                WHERE v.policy_id = sp.policy_id
                  AND v.valid_from <= $3
                  AND (v.valid_to IS NULL OR v.valid_to >= $3)
                ORDER BY v.valid_from DESC
                LIMIT 1
            ) apv ON true
            "#,
            workspace_id,
            employee_ids as &[Uuid],
            effective_time
        )
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            if let Some(employee_id) = row.employee_id {
                result.insert(
                    employee_id,
                    AttendancePolicyVersion {
                        id: row.id,
                        policy_id: row.policy_id,
                        rules: row.rules,
                        valid_from: row.valid_from,
                        valid_to: row.valid_to,
                        created_by: row.created_by,
                        created_at: row.created_at,
                    },
                );
            }
        }

        Ok(result)
    }
}
