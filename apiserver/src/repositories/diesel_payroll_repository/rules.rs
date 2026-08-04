use super::*;
use crate::errors::AppError;
use crate::models::diesel::payroll::PayslipRuleEvaluationEntity;
use crate::models::diesel::payroll_rule::PayrollRuleTierEntity;
use crate::models::PayslipRuleEvaluation;
use chrono::{NaiveDate, Utc};
use diesel_async::RunQueryDsl;
use std::str::FromStr;
use uuid::Uuid;

#[allow(clippy::too_many_lines)]
impl DieselPayrollRepository {
    pub(super) async fn create_payslip_rule_evaluation(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
        rule_code: String,
        amount: Decimal,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entity = PayslipRuleEvaluationEntity {
            id: Uuid::new_v4(),
            payslip_id,
            rule_code,
            amount: bigdecimal::BigDecimal::from_str(&amount.to_string()).unwrap_or_default(),
            created_at: Utc::now(),
        };

        diesel::insert_into(payslip_rule_evaluations::table)
            .values(&entity)
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn get_payslip_rule_evaluations(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
    ) -> Result<Vec<PayslipRuleEvaluation>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payslip_rule_evaluations::table
            .filter(payslip_rule_evaluations::payslip_id.eq(payslip_id))
            .select(PayslipRuleEvaluationEntity::as_select())
            .load::<PayslipRuleEvaluationEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn get_resolved_rule_assignments(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ResolvedRuleAssignment>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let sql = r#"
            WITH resolved_assignments AS (
                SELECT DISTINCT ON (rule_definition_id)
                    rule_definition_id, override_config, assignee_type
                FROM payroll_rule_assignments
                WHERE workspace_id = $1
                  AND effective_from <= $5
                  AND (effective_to IS NULL OR effective_to >= $4)
                  AND (
                      (assignee_type = 'employee' AND assignee_id = $2)
                      OR (assignee_type = 'department' AND assignee_id = $3)
                      OR (assignee_type = 'workspace')
                  )
                ORDER BY rule_definition_id,
                         CASE assignee_type
                            WHEN 'employee' THEN 1
                            WHEN 'department' THEN 2
                            WHEN 'workspace' THEN 3
                         END
            )
            SELECT
                r.rule_definition_id,
                r.override_config,
                d.code,
                d.calculation_method as "calculation_method",
                d.rule_config as default_config,
                d.category as "category"
            FROM resolved_assignments r
            JOIN payroll_rule_definitions d ON d.id = r.rule_definition_id
            WHERE d.is_active = true
            ORDER BY d.display_order
        "#;

        #[derive(QueryableByName)]
        struct ResolvedRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            pub rule_definition_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            pub override_config: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub code: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub calculation_method: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            pub default_config: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub category: String,
        }

        let rows = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(department_id)
            .bind::<diesel::sql_types::Date, _>(start_date)
            .bind::<diesel::sql_types::Date, _>(end_date)
            .load::<ResolvedRow>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(rows
            .into_iter()
            .map(|r| ResolvedRuleAssignment {
                rule_definition_id: r.rule_definition_id,
                override_config: r.override_config,
                code: r.code,
                calculation_method: r.calculation_method.parse().unwrap_or_default(),
                default_config: r.default_config,
                category: r.category.parse().unwrap_or_default(),
            })
            .collect())
    }

    pub(super) async fn get_payroll_rule_tiers(
        &self,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
    ) -> Result<Vec<crate::models::PayrollRuleTier>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payroll_rule_tiers::table
            .filter(payroll_rule_tiers::rule_definition_id.eq(rule_definition_id))
            .order(payroll_rule_tiers::tier_order.asc())
            .load::<PayrollRuleTierEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    #[allow(clippy::too_many_lines)]
    pub(super) async fn get_resolved_rule_assignments_batch(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        employee_departments: &std::collections::HashMap<Uuid, Option<Uuid>>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<ResolvedRuleAssignment>>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let sql = r#"
            SELECT
                r.rule_definition_id,
                r.override_config,
                r.assignee_type,
                r.assignee_id,
                d.code,
                d.calculation_method as "calculation_method",
                d.rule_config as default_config,
                d.category as "category",
                d.display_order as "display_order"
            FROM payroll_rule_assignments r
            JOIN payroll_rule_definitions d ON d.id = r.rule_definition_id
            WHERE r.workspace_id = $1
              AND d.is_active = true
              AND r.effective_from <= $3
              AND (r.effective_to IS NULL OR r.effective_to >= $2)
        "#;

        #[derive(QueryableByName)]
        struct RawAssignmentRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            pub rule_definition_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            pub override_config: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub assignee_type: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            pub assignee_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub code: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub calculation_method: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            pub default_config: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub category: String,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            pub display_order: i32,
        }

        let rows = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Date, _>(start_date)
            .bind::<diesel::sql_types::Date, _>(end_date)
            .load::<RawAssignmentRow>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        let mut result = std::collections::HashMap::new();

        for &emp_id in employee_ids {
            let dept_id = employee_departments.get(&emp_id).copied().flatten();

            // Map to store resolved assignments for this employee: rule_definition_id -> (assignee_type, ResolvedRuleAssignment, display_order)
            let mut resolved_rules: std::collections::HashMap<
                Uuid,
                (String, ResolvedRuleAssignment, i32),
            > = std::collections::HashMap::new();

            for row in &rows {
                let matches = match row.assignee_type.as_str() {
                    "employee" => row.assignee_id == Some(emp_id),
                    "department" => dept_id.is_some() && row.assignee_id == dept_id,
                    "workspace" => true,
                    _ => false,
                };

                if matches {
                    let precedence = match row.assignee_type.as_str() {
                        "employee" => 1,
                        "department" => 2,
                        "workspace" => 3,
                        _ => 4,
                    };

                    let should_update = if let Some((existing_type, _, _)) =
                        resolved_rules.get(&row.rule_definition_id)
                    {
                        let existing_precedence = match existing_type.as_str() {
                            "employee" => 1,
                            "department" => 2,
                            "workspace" => 3,
                            _ => 4,
                        };
                        precedence < existing_precedence
                    } else {
                        true
                    };

                    if should_update {
                        resolved_rules.insert(
                            row.rule_definition_id,
                            (
                                row.assignee_type.clone(),
                                ResolvedRuleAssignment {
                                    rule_definition_id: row.rule_definition_id,
                                    override_config: row.override_config.clone(),
                                    code: row.code.clone(),
                                    calculation_method: row
                                        .calculation_method
                                        .parse()
                                        .unwrap_or_default(),
                                    default_config: row.default_config.clone(),
                                    category: row.category.parse().unwrap_or_default(),
                                },
                                row.display_order,
                            ),
                        );
                    }
                }
            }

            let mut employee_tuples: Vec<(ResolvedRuleAssignment, i32)> = resolved_rules
                .into_values()
                .map(|(_, assignment, display_order)| (assignment, display_order))
                .collect();

            employee_tuples.sort_by_key(|t| t.1);
            let employee_assignments: Vec<ResolvedRuleAssignment> = employee_tuples
                .into_iter()
                .map(|(assignment, _)| assignment)
                .collect();

            result.insert(emp_id, employee_assignments);
        }

        Ok(result)
    }

    pub(super) async fn get_payroll_rule_tiers_batch(
        &self,
        workspace_id: Uuid,
        rule_definition_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<crate::models::PayrollRuleTier>>, AppError>
    {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payroll_rule_tiers::table
            .filter(payroll_rule_tiers::rule_definition_id.eq_any(rule_definition_ids))
            .order(payroll_rule_tiers::tier_order.asc())
            .load::<PayrollRuleTierEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        let mut map = std::collections::HashMap::new();
        for &def_id in rule_definition_ids {
            map.insert(def_id, Vec::new());
        }

        for ent in entities {
            let def_id = ent.rule_definition_id;
            map.entry(def_id).or_insert_with(Vec::new).push(ent.into());
        }

        Ok(map)
    }
}
