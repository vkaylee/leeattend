use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{AttendancePolicyVersion, AttendanceRules};
use crate::repositories::attendance_policy_repository::AttendancePolicyRepository;
use crate::utils::i18n::I18nKey;

use diesel::sql_types::{Array, Nullable, Timestamptz, Uuid as UuidType};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, QueryableByName};
use diesel_async::{AsyncConnection, RunQueryDsl};
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

#[derive(QueryableByName)]
struct BatchEmploymentPolicyRow {
    #[diesel(sql_type = UuidType)]
    employment_id: Uuid,
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
    diesel_pools: crate::diesel_pool::DieselPools,
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
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        effective_time: DateTime<Utc>,
    ) -> Result<Option<AttendancePolicyVersion>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let row: Option<SinglePolicyRow> = diesel::sql_query(
            r"
            WITH dept_history AS (
                SELECT department_id
                FROM employments
                WHERE employee_id = $2
                  AND deleted_at IS NULL
                  AND start_date <= ($3::timestamptz)::date
                  AND (end_date IS NULL OR end_date >= ($3::timestamptz)::date)
                ORDER BY start_date DESC
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
            "
        )
        .bind::<UuidType, _>(workspace_id)
        .bind::<UuidType, _>(employee_id)
        .bind::<Timestamptz, _>(effective_time)
        .get_result(&mut *conn)
        .await
        .optional()
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|r| AttendancePolicyVersion {
            id: r.id,
            policy_id: r.policy_id,
            rules: serde_json::from_value(r.rules).unwrap_or_else(|_| AttendanceRules {
                work_unit_minutes: 0,
                rounding_policy: "none".to_string(),
                early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
                grace_period_minutes: None,
            }),
            valid_from: r.valid_from,
            valid_to: r.valid_to,
            created_by: r.created_by,
            created_at: r.created_at,
        }))
    }

    async fn get_employee_effective_policies_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        effective_time: DateTime<Utc>,
    ) -> Result<HashMap<Uuid, AttendancePolicyVersion>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut result = std::collections::HashMap::new();

        if employee_ids.is_empty() {
            return Ok(result);
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let rows: Vec<BatchPolicyRow> = diesel::sql_query(
            r"
            WITH inputs AS (
                SELECT unnest($2::uuid[]) AS employee_id
            ),
            dept_history AS (
                SELECT i.employee_id,
                    (
                        SELECT department_id
                        FROM employments
                        WHERE employee_id = i.employee_id
                          AND deleted_at IS NULL
                          AND start_date <= ($3::timestamptz)::date
                          AND (end_date IS NULL OR end_date >= ($3::timestamptz)::date)
                        ORDER BY start_date DESC
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
            ",
        )
        .bind::<UuidType, _>(workspace_id)
        .bind::<Array<UuidType>, _>(employee_ids.to_vec())
        .bind::<Timestamptz, _>(effective_time)
        .load(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        for row in rows {
            result.insert(
                row.employee_id,
                AttendancePolicyVersion {
                    id: row.id,
                    policy_id: row.policy_id,
                    rules: serde_json::from_value(row.rules).unwrap_or_else(|_| AttendanceRules {
                        work_unit_minutes: 0,
                        rounding_policy: "none".to_string(),
                        early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
                        grace_period_minutes: None,
                    }),
                    valid_from: row.valid_from,
                    valid_to: row.valid_to,
                    created_by: row.created_by,
                    created_at: row.created_at,
                },
            );
        }

        Ok(result)
    }

    async fn get_employment_effective_policy(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        effective_time: DateTime<Utc>,
    ) -> Result<Option<AttendancePolicyVersion>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let row: Option<SinglePolicyRow> = diesel::sql_query(
            r"
            WITH emp_info AS (
                SELECT employee_id, department_id
                FROM employments
                WHERE id = $2
                  AND deleted_at IS NULL
                LIMIT 1
            ),
            employment_policy AS (
                SELECT ap.id as policy_id
                FROM attendance_policy_assignments apa
                JOIN attendance_policies ap ON ap.id = apa.policy_id
                WHERE apa.target_id = $2
                  AND apa.target_type = 'EMPLOYMENT'
                  AND apa.valid_from <= $3
                  AND (apa.valid_to IS NULL OR apa.valid_to >= $3)
                  AND ap.workspace_id = $1
                ORDER BY apa.valid_from DESC
                LIMIT 1
            ),
            user_policy AS (
                SELECT ap.id as policy_id
                FROM attendance_policy_assignments apa
                JOIN attendance_policies ap ON ap.id = apa.policy_id
                JOIN emp_info ei ON ei.employee_id = apa.target_id
                WHERE apa.target_type = 'USER'
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
                JOIN emp_info ei ON ei.department_id = apa.target_id
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
                SELECT policy_id FROM employment_policy
                UNION ALL
                SELECT policy_id FROM user_policy WHERE NOT EXISTS (SELECT 1 FROM employment_policy)
                UNION ALL
                SELECT policy_id FROM group_policy WHERE NOT EXISTS (SELECT 1 FROM employment_policy) AND NOT EXISTS (SELECT 1 FROM user_policy)
                UNION ALL
                SELECT policy_id FROM default_policy WHERE NOT EXISTS (SELECT 1 FROM employment_policy) AND NOT EXISTS (SELECT 1 FROM user_policy) AND NOT EXISTS (SELECT 1 FROM group_policy)
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
            "
        )
        .bind::<UuidType, _>(workspace_id)
        .bind::<UuidType, _>(employment_id)
        .bind::<Timestamptz, _>(effective_time)
        .get_result(&mut *conn)
        .await
        .optional()
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(row.map(|r| AttendancePolicyVersion {
            id: r.id,
            policy_id: r.policy_id,
            rules: serde_json::from_value(r.rules).unwrap_or_else(|_| AttendanceRules {
                work_unit_minutes: 0,
                rounding_policy: "none".to_string(),
                early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
                grace_period_minutes: None,
            }),
            valid_from: r.valid_from,
            valid_to: r.valid_to,
            created_by: r.created_by,
            created_at: r.created_at,
        }))
    }

    async fn get_employment_effective_policies_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        effective_time: DateTime<Utc>,
    ) -> Result<HashMap<Uuid, AttendancePolicyVersion>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut result = std::collections::HashMap::new();

        if employment_ids.is_empty() {
            return Ok(result);
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let rows: Vec<BatchEmploymentPolicyRow> = diesel::sql_query(
            r"
            WITH inputs AS (
                SELECT unnest($2::uuid[]) AS employment_id
            ),
            emp_info AS (
                SELECT i.employment_id, e.employee_id, e.department_id
                FROM inputs i
                JOIN employments e ON e.id = i.employment_id
                WHERE e.deleted_at IS NULL
            ),
            employment_policy AS (
                SELECT i.employment_id,
                    (
                        SELECT ap.id
                        FROM attendance_policy_assignments apa
                        JOIN attendance_policies ap ON ap.id = apa.policy_id
                        WHERE apa.target_id = i.employment_id
                          AND apa.target_type = 'EMPLOYMENT'
                          AND apa.valid_from <= $3
                          AND (apa.valid_to IS NULL OR apa.valid_to >= $3)
                          AND ap.workspace_id = $1
                        ORDER BY apa.valid_from DESC
                        LIMIT 1
                    ) as policy_id
                FROM inputs i
            ),
            user_policy AS (
                SELECT ei.employment_id,
                    (
                        SELECT ap.id
                        FROM attendance_policy_assignments apa
                        JOIN attendance_policies ap ON ap.id = apa.policy_id
                        WHERE apa.target_id = ei.employee_id
                          AND apa.target_type = 'USER'
                          AND apa.valid_from <= $3
                          AND (apa.valid_to IS NULL OR apa.valid_to >= $3)
                          AND ap.workspace_id = $1
                        ORDER BY apa.valid_from DESC
                        LIMIT 1
                    ) as policy_id
                FROM emp_info ei
            ),
            group_policy AS (
                SELECT ei.employment_id,
                    (
                        SELECT ap.id
                        FROM attendance_policy_assignments apa
                        JOIN attendance_policies ap ON ap.id = apa.policy_id
                        WHERE apa.target_id = ei.department_id
                          AND apa.target_type = 'GROUP'
                          AND apa.valid_from <= $3
                          AND (apa.valid_to IS NULL OR apa.valid_to >= $3)
                          AND ap.workspace_id = $1
                        ORDER BY apa.valid_from DESC
                        LIMIT 1
                    ) as policy_id
                FROM emp_info ei
            ),
            default_policy AS (
                SELECT id as policy_id
                FROM attendance_policies
                WHERE workspace_id = $1 AND is_default = true
                LIMIT 1
            ),
            selected_policy AS (
                SELECT i.employment_id,
                    COALESCE(
                        ep.policy_id,
                        up.policy_id,
                        gp.policy_id,
                        (SELECT policy_id FROM default_policy)
                    ) as policy_id
                FROM inputs i
                LEFT JOIN employment_policy ep ON ep.employment_id = i.employment_id
                LEFT JOIN user_policy up ON up.employment_id = i.employment_id
                LEFT JOIN group_policy gp ON gp.employment_id = i.employment_id
            )
            SELECT
                sp.employment_id,
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
            ",
        )
        .bind::<UuidType, _>(workspace_id)
        .bind::<Array<UuidType>, _>(employment_ids.to_vec())
        .bind::<Timestamptz, _>(effective_time)
        .load(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        for row in rows {
            result.insert(
                row.employment_id,
                AttendancePolicyVersion {
                    id: row.id,
                    policy_id: row.policy_id,
                    rules: serde_json::from_value(row.rules).unwrap_or_else(|_| AttendanceRules {
                        work_unit_minutes: 0,
                        rounding_policy: "none".to_string(),
                        early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
                        grace_period_minutes: None,
                    }),
                    valid_from: row.valid_from,
                    valid_to: row.valid_to,
                    created_by: row.created_by,
                    created_at: row.created_at,
                },
            );
        }

        Ok(result)
    }

    async fn get_default_policy_rules(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<crate::models::AttendanceRules>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        use crate::schema::{attendance_policies, attendance_policy_versions};

        // 1. Get default policy ID
        let policy_id: Option<Uuid> = attendance_policies::table
            .filter(attendance_policies::workspace_id.eq(workspace_id))
            .filter(attendance_policies::is_default.eq(true))
            .select(attendance_policies::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let Some(p_id) = policy_id else {
            return Ok(None);
        };

        // 2. Get latest rules
        let rules: Option<serde_json::Value> = attendance_policy_versions::table
            .filter(attendance_policy_versions::policy_id.eq(p_id))
            .order(attendance_policy_versions::valid_from.desc())
            .select(attendance_policy_versions::rules)
            .first::<serde_json::Value>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        match rules {
            Some(r) => Ok(Some(serde_json::from_value(r).unwrap_or_else(|_| {
                AttendanceRules {
                    work_unit_minutes: 15,
                    rounding_policy: "none".to_string(),
                    early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
                    grace_period_minutes: None,
                }
            }))),
            None => Ok(None),
        }
    }

    async fn get_default_policy_versions(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<AttendancePolicyVersion>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        use crate::schema::{attendance_policies, attendance_policy_versions};

        let policy_id: Option<Uuid> = attendance_policies::table
            .filter(attendance_policies::workspace_id.eq(workspace_id))
            .filter(attendance_policies::is_default.eq(true))
            .select(attendance_policies::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let Some(p_id) = policy_id else {
            return Ok(vec![]);
        };

        let versions = attendance_policy_versions::table
            .filter(attendance_policy_versions::policy_id.eq(p_id))
            .order(attendance_policy_versions::valid_from.desc())
            .load::<crate::models::diesel::attendance_policy::AttendancePolicyVersionEntity>(
                &mut *conn,
            )
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(versions.into_iter().map(Into::into).collect())
    }

    async fn update_default_policy_rules(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        req: crate::handlers::attendance::policies::UpdateAttendanceRulesRequest,
    ) -> Result<AttendancePolicyVersion, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        use crate::schema::{attendance_policies, attendance_policy_versions};

        conn.transaction(|conn| {
            Box::pin(async move {
                let policy_id_opt: Option<Uuid> = attendance_policies::table
                    .filter(attendance_policies::workspace_id.eq(workspace_id))
                    .filter(attendance_policies::is_default.eq(true))
                    .select(attendance_policies::id)
                    .first::<Uuid>(&mut *conn)
                    .await
                    .optional()
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                let policy_id = if let Some(id) = policy_id_opt {
                    id
                } else {
                    let new_id = crate::utils::id::new_id();
                    let entity = crate::models::diesel::attendance_policy::AttendancePolicyEntity {
                        id: new_id,
                        workspace_id,
                        name: "Default Policy".to_string(),
                        is_default: true,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };
                    diesel::insert_into(attendance_policies::table)
                        .values(&entity)
                        .execute(&mut *conn)
                        .await
                        .map_err(|e| AppError::Internal(e.to_string()))?;
                    new_id
                };

                let current_rules_val: Option<serde_json::Value> =
                    attendance_policy_versions::table
                        .filter(attendance_policy_versions::policy_id.eq(policy_id))
                        .order(attendance_policy_versions::valid_from.desc())
                        .select(attendance_policy_versions::rules)
                        .first::<serde_json::Value>(&mut *conn)
                        .await
                        .optional()
                        .map_err(|e| AppError::Internal(e.to_string()))?;

                let mut new_rules = match current_rules_val {
                    Some(v) => serde_json::from_value(v).unwrap_or_else(|_| AttendanceRules {
                        work_unit_minutes: 15,
                        rounding_policy: "none".to_string(),
                        early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
                        grace_period_minutes: None,
                    }),
                    None => AttendanceRules {
                        work_unit_minutes: 15,
                        rounding_policy: "none".to_string(),
                        early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
                        grace_period_minutes: None,
                    },
                };

                if let Some(w) = req.work_unit_minutes {
                    new_rules.work_unit_minutes = w;
                }
                if let Some(r) = req.rounding_policy {
                    new_rules.rounding_policy = r;
                }
                if let Some(s) = req.early_arrival_strategy {
                    new_rules.early_arrival_strategy = s;
                }
                if let Some(gp) = req.grace_period_minutes {
                    new_rules.grace_period_minutes = Some(gp);
                }

                let valid_from = match &req.valid_from {
                    Some(s) => chrono::DateTime::parse_from_rfc3339(s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .or_else(|_| {
                            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                                .map_err(|e| e.to_string())
                                .and_then(|d| {
                                    d.and_hms_opt(0, 0, 0)
                                        .map(|dt| dt.and_utc())
                                        .ok_or_else(|| "Invalid time".to_string())
                                })
                        })
                        .map_err(|_e| AppError::validation(I18nKey::InvalidDateFormat))?,
                    None => Utc::now(),
                };

                diesel::update(
                    attendance_policy_versions::table
                        .filter(attendance_policy_versions::policy_id.eq(policy_id))
                        .filter(attendance_policy_versions::valid_to.is_null()),
                )
                .set(attendance_policy_versions::valid_to.eq(Some(valid_from)))
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

                let version_id = crate::utils::id::new_id();
                let now = Utc::now();
                let version_entity =
                    crate::models::diesel::attendance_policy::AttendancePolicyVersionEntity {
                        id: version_id,
                        policy_id,
                        rules: serde_json::to_value(&new_rules).unwrap_or_default(),
                        valid_from,
                        valid_to: None,
                        created_by: Some(user_id),
                        created_at: now,
                    };

                diesel::insert_into(attendance_policy_versions::table)
                    .values(&version_entity)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                Ok(AttendancePolicyVersion {
                    id: version_id,
                    policy_id,
                    rules: new_rules,
                    valid_from,
                    valid_to: None,
                    created_by: Some(user_id),
                    created_at: now,
                })
            })
        })
        .await
    }

    async fn list_policy_assignments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<
        Vec<crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse>,
        AppError,
    > {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        #[derive(diesel::QueryableByName, diesel::Queryable)]
        struct PolicyAssignmentRow {
            #[diesel(sql_type = UuidType)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            target_type: String,
            #[diesel(sql_type = UuidType)]
            target_id: Uuid,
            #[diesel(sql_type = Timestamptz)]
            valid_from: DateTime<Utc>,
            #[diesel(sql_type = Nullable<Timestamptz>)]
            valid_to: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            rules: Value,
        }

        use crate::schema::{
            attendance_policies, attendance_policy_assignments, attendance_policy_versions,
        };
        use diesel::{BoolExpressionMethods, JoinOnDsl};

        let now = Utc::now();

        let rows: Vec<PolicyAssignmentRow> = attendance_policy_assignments::table
            .inner_join(
                attendance_policies::table
                    .on(attendance_policies::id.eq(attendance_policy_assignments::policy_id)),
            )
            .inner_join(
                attendance_policy_versions::table
                    .on(attendance_policy_versions::policy_id.eq(attendance_policies::id)),
            )
            .filter(attendance_policies::workspace_id.eq(workspace_id))
            .filter(attendance_policy_versions::valid_from.le(now))
            .filter(
                attendance_policy_versions::valid_to
                    .is_null()
                    .or(attendance_policy_versions::valid_to.ge(now)),
            )
            .filter(attendance_policy_assignments::valid_from.le(now))
            .filter(
                attendance_policy_assignments::valid_to
                    .is_null()
                    .or(attendance_policy_assignments::valid_to.ge(now)),
            )
            .select((
                attendance_policy_assignments::id,
                attendance_policy_assignments::target_type,
                attendance_policy_assignments::target_id,
                attendance_policy_assignments::valid_from,
                attendance_policy_assignments::valid_to,
                attendance_policy_versions::rules,
            ))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let assignments = rows
            .into_iter()
            .map(
                |row| crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse {
                    id: row.id.to_string(),
                    target_type: row.target_type,
                    target_id: row.target_id.to_string(),
                    valid_from: row.valid_from.to_rfc3339(),
                    valid_to: row.valid_to.map(|d| d.to_rfc3339()),
                    rules: serde_json::from_value(row.rules).unwrap_or_else(|_| AttendanceRules {
                        work_unit_minutes: 0,
                        rounding_policy: "none".to_string(),
                        early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy::Block,
                        grace_period_minutes: None,
                    }),
                },
            )
            .collect();

        Ok(assignments)
    }

    async fn upsert_policy_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        req: crate::handlers::attendance::policy_assignments::UpsertPolicyAssignmentRequest,
    ) -> Result<crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse, AppError>
    {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        use crate::schema::{
            attendance_policies, attendance_policy_assignments, attendance_policy_versions,
        };

        conn.transaction(|conn| {
            Box::pin(async move {
                let valid_from = match &req.valid_from {
                    Some(s) => chrono::DateTime::parse_from_rfc3339(s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .or_else(|_| {
                            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                                .map_err(|e| e.to_string())
                                .and_then(|d| {
                                    d.and_hms_opt(0, 0, 0)
                                        .map(|dt| dt.and_utc())
                                        .ok_or_else(|| "Invalid time".to_string())
                                })
                        })
                        .map_err(|_e| AppError::validation(I18nKey::InvalidDateFormat))?,
                    None => Utc::now(),
                };

                let existing: Option<(Uuid, Uuid)> = attendance_policy_assignments::table
                    .filter(attendance_policy_assignments::target_type.eq(&req.target_type))
                    .filter(attendance_policy_assignments::target_id.eq(&req.target_id))
                    .filter(attendance_policy_assignments::valid_to.is_null())
                    .select((attendance_policy_assignments::id, attendance_policy_assignments::policy_id))
                    .first::<(Uuid, Uuid)>(&mut *conn)
                    .await
                    .optional()
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                let policy_id;
                let assignment_id;

                if let Some((ext_assignment_id, ext_policy_id)) = existing {
                    policy_id = ext_policy_id;
                    assignment_id = ext_assignment_id;

                    diesel::update(
                        attendance_policy_versions::table
                            .filter(attendance_policy_versions::policy_id.eq(policy_id))
                            .filter(attendance_policy_versions::valid_to.is_null()),
                    )
                    .set(attendance_policy_versions::valid_to.eq(Some(valid_from)))
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                } else {
                    policy_id = crate::utils::id::new_id();
                    let policy_entity = crate::models::diesel::attendance_policy::AttendancePolicyEntity {
                        id: policy_id,
                        workspace_id,
                        name: format!("{} {} Override", req.target_type, req.target_id),
                        is_default: false,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };
                    diesel::insert_into(attendance_policies::table)
                        .values(&policy_entity)
                        .execute(&mut *conn)
                        .await
                        .map_err(|e| AppError::Internal(e.to_string()))?;

                    assignment_id = crate::utils::id::new_id();
                    let assignment_entity = crate::models::diesel::attendance_policy::AttendancePolicyAssignmentEntity {
                        id: assignment_id,
                        policy_id,
                        target_type: req.target_type.clone(),
                        target_id: req.target_id,
                        valid_from,
                        valid_to: None,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };
                    diesel::insert_into(attendance_policy_assignments::table)
                        .values(&assignment_entity)
                        .execute(&mut *conn)
                        .await
                        .map_err(|e| AppError::Internal(e.to_string()))?;
                }

                let version_id = crate::utils::id::new_id();
                let rules = AttendanceRules {
                    work_unit_minutes: req.work_unit_minutes,
                    rounding_policy: req.rounding_policy.clone(),
                    early_arrival_strategy: req.early_arrival_strategy,
                    grace_period_minutes: req.grace_period_minutes,
                };

                let version_entity = crate::models::diesel::attendance_policy::AttendancePolicyVersionEntity {
                    id: version_id,
                    policy_id,
                    rules: serde_json::to_value(&rules).unwrap_or_default(),
                    valid_from,
                    valid_to: None,
                    created_by: Some(user_id),
                    created_at: Utc::now(),
                };

                diesel::insert_into(attendance_policy_versions::table)
                    .values(&version_entity)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                Ok(crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse {
                    id: assignment_id.to_string(),
                    target_type: req.target_type,
                    target_id: req.target_id.to_string(),
                    valid_from: valid_from.to_rfc3339(),
                    valid_to: None,
                    rules,
                })
            })
        })
        .await
    }

    async fn delete_policy_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        assignment_id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        use crate::schema::{attendance_policies, attendance_policy_assignments};

        let now = Utc::now();

        diesel::update(
            attendance_policy_assignments::table
                .filter(attendance_policy_assignments::id.eq(assignment_id))
                .filter(
                    attendance_policy_assignments::policy_id.eq_any(
                        attendance_policies::table
                            .select(attendance_policies::id)
                            .filter(attendance_policies::workspace_id.eq(workspace_id)),
                    ),
                ),
        )
        .set((
            attendance_policy_assignments::valid_to.eq(Some(now)),
            attendance_policy_assignments::updated_at.eq(now),
        ))
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselAttendancePolicyRepository::new(pools);
        // DI connection test passes if it compiles
    }

    #[tokio::test]
    async fn test_delete_policy_assignment() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselAttendancePolicyRepository::new(pools.clone());
        let ws_id = Uuid::new_v4();
        let token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);

        let mut conn = if let Ok(c) = pools.tenant_transaction_conn(&token).await {
            c
        } else {
            println!("Database not available, skipping integration assertions");
            return;
        };

        // Insert Workspace to satisfy FK constraints
        let workspace_entity = crate::models::diesel::core::WorkspaceEntity {
            id: ws_id,
            name: "Test Workspace".to_string(),
            created_at: Some(chrono::Utc::now()),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: "HH:mm".to_string(),
            timezone: "UTC".to_string(),
            theme_primary_color: None,
            default_language: "en".to_string(),
            logo_url: None,
            tagline: None,
            custom_db_url_encrypted: None,
        };
        diesel::insert_into(crate::schema::workspaces::table)
            .values(&workspace_entity)
            .execute(&mut *conn)
            .await
            .expect("insert workspace");

        // Insert an attendance policy
        let policy_id = Uuid::new_v4();
        diesel::sql_query("INSERT INTO attendance_policies (id, workspace_id, name, is_default, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind::<diesel::sql_types::Uuid, _>(policy_id)
            .bind::<diesel::sql_types::Uuid, _>(ws_id)
            .bind::<diesel::sql_types::Text, _>("Test Policy")
            .bind::<diesel::sql_types::Bool, _>(false)
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .execute(&mut *conn)
            .await
            .expect("insert policy");

        // Insert a policy assignment
        let assignment_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();

        // Insert an employee to satisfy FK trigger check on public.employees
        diesel::sql_query("INSERT INTO employees (id, workspace_id, employee_id_number, full_name, hire_date, original_hire_date, is_active) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind::<diesel::sql_types::Uuid, _>(target_id)
            .bind::<diesel::sql_types::Uuid, _>(ws_id)
            .bind::<diesel::sql_types::Text, _>("EMP-TEST-999")
            .bind::<diesel::sql_types::Text, _>("Test Employee")
            .bind::<diesel::sql_types::Date, _>(chrono::Utc::now().date_naive())
            .bind::<diesel::sql_types::Date, _>(chrono::Utc::now().date_naive())
            .bind::<diesel::sql_types::Bool, _>(true)
            .execute(&mut *conn)
            .await
            .expect("insert employee");

        diesel::sql_query("INSERT INTO attendance_policy_assignments (id, policy_id, target_type, target_id, valid_from, valid_to, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind::<diesel::sql_types::Uuid, _>(assignment_id)
            .bind::<diesel::sql_types::Uuid, _>(policy_id)
            .bind::<diesel::sql_types::Text, _>("USER")
            .bind::<diesel::sql_types::Uuid, _>(target_id)
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(None::<chrono::DateTime<chrono::Utc>>)
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .execute(&mut *conn)
            .await
            .expect("insert assignment");

        drop(conn);

        // Call delete_policy_assignment
        let res = repo.delete_policy_assignment(&token, assignment_id).await;
        assert!(res.is_ok());

        // Verify it was soft-deleted (valid_to is set)
        let mut conn = pools.tenant_transaction_conn(&token).await.unwrap();
        let valid_to_val: Option<chrono::DateTime<chrono::Utc>> =
            crate::schema::attendance_policy_assignments::table
                .filter(crate::schema::attendance_policy_assignments::id.eq(assignment_id))
                .select(crate::schema::attendance_policy_assignments::valid_to)
                .first(&mut *conn)
                .await
                .unwrap();
        assert!(valid_to_val.is_some());
    }
}
