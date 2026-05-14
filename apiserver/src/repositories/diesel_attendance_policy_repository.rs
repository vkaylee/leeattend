use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{AttendancePolicyVersion, AttendanceRules};
use crate::repositories::attendance_policy_repository::AttendancePolicyRepository;

use diesel::sql_types::{Array, Nullable, Timestamptz, Uuid as UuidType};
use diesel::{OptionalExtension, QueryableByName};
use diesel_async::RunQueryDsl;
use serde_json::Value;

#[derive(QueryableByName)]
struct SinglePolicyRow {
    #[diesel(sql_type = UuidType)]
    id: Uuid,
    #[diesel(sql_type = UuidType)]
    policy_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    rules: Value,
    #[diesel(sql_type = Timestamptz)]
    valid_from: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    valid_to: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<UuidType>)]
    created_by: Option<Uuid>,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct BatchPolicyRow {
    #[diesel(sql_type = UuidType)]
    employee_id: Uuid,
    #[diesel(sql_type = UuidType)]
    id: Uuid,
    #[diesel(sql_type = UuidType)]
    policy_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    rules: Value,
    #[diesel(sql_type = Timestamptz)]
    valid_from: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    valid_to: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<UuidType>)]
    created_by: Option<Uuid>,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

pub struct DieselAttendancePolicyRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselAttendancePolicyRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl AttendancePolicyRepository for DieselAttendancePolicyRepository {
    async fn get_employee_effective_policy(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        effective_time: DateTime<Utc>,
    ) -> Result<Option<AttendancePolicyVersion>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let row: Option<SinglePolicyRow> = diesel::sql_query(
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
                apv.rules,
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
            "#
        )
        .bind::<UuidType, _>(workspace_id)
        .bind::<UuidType, _>(employee_id)
        .bind::<Timestamptz, _>(effective_time)
        .get_result(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|r| AttendancePolicyVersion {
            id: r.id,
            policy_id: r.policy_id,
            rules: sqlx::types::Json(serde_json::from_value(r.rules).unwrap_or_else(|_| {
                AttendanceRules {
                    work_unit_minutes: 0,
                    rounding_policy: "none".to_string(),
                    early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
                }
            })),
            valid_from: r.valid_from,
            valid_to: r.valid_to,
            created_by: r.created_by,
            created_at: r.created_at,
        }))
    }

    async fn get_employee_effective_policies_batch(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        effective_time: DateTime<Utc>,
    ) -> Result<HashMap<Uuid, AttendancePolicyVersion>, AppError> {
        let mut result = std::collections::HashMap::new();

        if employee_ids.is_empty() {
            return Ok(result);
        }

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let rows: Vec<BatchPolicyRow> = diesel::sql_query(
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
                apv.rules,
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
        )
        .bind::<UuidType, _>(workspace_id)
        .bind::<Array<UuidType>, _>(employee_ids.to_vec())
        .bind::<Timestamptz, _>(effective_time)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        for row in rows {
            result.insert(
                row.employee_id,
                AttendancePolicyVersion {
                    id: row.id,
                    policy_id: row.policy_id,
                    rules: sqlx::types::Json(serde_json::from_value(row.rules).unwrap_or_else(
                        |_| AttendanceRules {
                            work_unit_minutes: 0,
                            rounding_policy: "none".to_string(),
                            early_arrival_strategy:
                                crate::models::enums::EarlyArrivalStrategy::Block,
                        },
                    )),
                    valid_from: row.valid_from,
                    valid_to: row.valid_to,
                    created_by: row.created_by,
                    created_at: row.created_at,
                },
            );
        }

        Ok(result)
    }
}
