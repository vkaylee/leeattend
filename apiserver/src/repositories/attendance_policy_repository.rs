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
        
        // Very basic N+1 for now, easily optimizable later if needed, since batch size is small (50)
        for &employee_id in employee_ids {
            if let Some(pv) = self.get_employee_effective_policy(workspace_id, employee_id, effective_time).await? {
                result.insert(employee_id, pv);
            }
        }
        
        Ok(result)
    }
}
