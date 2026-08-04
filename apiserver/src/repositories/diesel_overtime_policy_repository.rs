use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::overtime_policy::{
    OvertimePolicyAssignmentEntity, OvertimePolicyEntity, OvertimeRuleEntity,
};
use crate::models::{OvertimePolicy, OvertimeRule};
use crate::repositories::overtime_policy_repository::OvertimePolicyRepositoryTrait;
use crate::schema::{overtime_policy_assignments, overtime_policy_definitions, overtime_rules};

pub struct DieselOvertimePolicyRepository {
    diesel_pools: DieselPools,
}

impl DieselOvertimePolicyRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl OvertimePolicyRepositoryTrait for DieselOvertimePolicyRepository {
    async fn create_policy_definition(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
    ) -> Result<OvertimePolicy, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::insert_into(overtime_policy_definitions::table)
            .values((
                overtime_policy_definitions::workspace_id.eq(workspace_id),
                overtime_policy_definitions::name.eq(name),
                overtime_policy_definitions::description.eq(description),
            ))
            .returning(OvertimePolicyEntity::as_returning())
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)
            .map(Into::into)
    }

    async fn get_policy_definitions(
        &self,
        workspace_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<OvertimePolicy>, i64), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let total: i64 = overtime_policy_definitions::table
            .filter(overtime_policy_definitions::workspace_id.eq(workspace_id))
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

        let rows: Vec<OvertimePolicyEntity> = overtime_policy_definitions::table
            .filter(overtime_policy_definitions::workspace_id.eq(workspace_id))
            .order(overtime_policy_definitions::created_at.desc())
            .limit(limit)
            .offset((page - 1) * limit)
            .select(OvertimePolicyEntity::as_select())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok((rows.into_iter().map(Into::into).collect(), total))
    }

    async fn get_policy_with_rules(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<(OvertimePolicy, Vec<OvertimeRule>)>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let policy: Option<OvertimePolicyEntity> = overtime_policy_definitions::table
            .filter(overtime_policy_definitions::id.eq(id))
            .filter(overtime_policy_definitions::workspace_id.eq(workspace_id))
            .select(OvertimePolicyEntity::as_select())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        if let Some(p) = policy {
            let rules: Vec<OvertimeRuleEntity> = overtime_rules::table
                .filter(overtime_rules::policy_id.eq(id))
                .order(overtime_rules::priority.desc())
                .select(OvertimeRuleEntity::as_select())
                .load(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

            Ok(Some((
                p.into(),
                rules.into_iter().map(Into::into).collect(),
            )))
        } else {
            Ok(None)
        }
    }

    async fn add_rule(
        &self,
        workspace_id: Uuid,
        rule: &OvertimeRule,
    ) -> Result<OvertimeRule, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::insert_into(overtime_rules::table)
            .values((
                overtime_rules::policy_id.eq(rule.policy_id()),
                overtime_rules::priority.eq(rule.priority()),
                overtime_rules::multiplier.eq(rule.multiplier()),
                overtime_rules::time_start.eq(rule.time_start()),
                overtime_rules::time_end.eq(rule.time_end()),
                overtime_rules::days_of_week.eq(rule.days_of_week()),
                overtime_rules::is_holiday.eq(rule.is_holiday()),
            ))
            .returning(OvertimeRuleEntity::as_returning())
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)
            .map(Into::into)
    }

    async fn resolve_policy_for_employee(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Uuid>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // 1. Employee Override
        let emp_policy: Option<Uuid> = overtime_policy_assignments::table
            .filter(overtime_policy_assignments::workspace_id.eq(workspace_id))
            .filter(overtime_policy_assignments::assignee_type.eq("employee"))
            .filter(overtime_policy_assignments::assignee_id.eq(employee_id))
            .filter(overtime_policy_assignments::effective_from.le(date))
            .filter(
                overtime_policy_assignments::effective_to
                    .is_null()
                    .or(overtime_policy_assignments::effective_to.ge(date)),
            )
            .order(overtime_policy_assignments::created_at.desc())
            .select(overtime_policy_assignments::policy_id)
            .first(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        if let Some(policy_id) = emp_policy {
            return Ok(Some(policy_id));
        }

        use crate::schema::employments;

        // 2. Department Override
        let dept_policy: Option<Uuid> = overtime_policy_assignments::table
            .inner_join(employments::table.on(employments::employee_id.eq(employee_id)))
            .filter(overtime_policy_assignments::workspace_id.eq(workspace_id))
            .filter(overtime_policy_assignments::assignee_type.eq("department"))
            .filter(employments::deleted_at.is_null())
            .filter(employments::department_id.eq(overtime_policy_assignments::assignee_id))
            .filter(employments::start_date.le(date))
            .filter(
                employments::end_date
                    .is_null()
                    .or(employments::end_date.ge(date)),
            )
            .filter(overtime_policy_assignments::effective_from.le(date))
            .filter(
                overtime_policy_assignments::effective_to
                    .is_null()
                    .or(overtime_policy_assignments::effective_to.ge(date)),
            )
            .order((
                employments::start_date.desc(),
                overtime_policy_assignments::created_at.desc(),
            ))
            .select(overtime_policy_assignments::policy_id)
            .first(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        if let Some(policy_id) = dept_policy {
            return Ok(Some(policy_id));
        }

        // 3. Workspace Default
        let ws_policy: Option<Uuid> = overtime_policy_assignments::table
            .filter(overtime_policy_assignments::workspace_id.eq(workspace_id))
            .filter(overtime_policy_assignments::assignee_type.eq("workspace"))
            .filter(overtime_policy_assignments::effective_from.le(date))
            .filter(
                overtime_policy_assignments::effective_to
                    .is_null()
                    .or(overtime_policy_assignments::effective_to.ge(date)),
            )
            .order(overtime_policy_assignments::created_at.desc())
            .select(overtime_policy_assignments::policy_id)
            .first(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(ws_policy)
    }

    async fn create_policy_assignment(
        &self,
        workspace_id: Uuid,
        assignee_type: &str,
        assignee_id: Option<Uuid>,
        policy_id: Uuid,
        effective_from: NaiveDate,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::insert_into(overtime_policy_assignments::table)
            .values((
                overtime_policy_assignments::workspace_id.eq(workspace_id),
                overtime_policy_assignments::assignee_type.eq(assignee_type),
                overtime_policy_assignments::assignee_id.eq(assignee_id),
                overtime_policy_assignments::policy_id.eq(policy_id),
                overtime_policy_assignments::effective_from.eq(effective_from),
            ))
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn get_workspace_overtime_data(
        &self,
        workspace_id: Uuid,
    ) -> Result<crate::repositories::overtime_policy_repository::WorkspaceOvertimeData, AppError>
    {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let policies_entities: Vec<OvertimePolicyEntity> = overtime_policy_definitions::table
            .filter(overtime_policy_definitions::workspace_id.eq(workspace_id))
            .select(OvertimePolicyEntity::as_select())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let policy_ids: Vec<Uuid> = policies_entities.iter().map(|p| p.id).collect();

        let all_rules: Vec<OvertimeRuleEntity> = if policy_ids.is_empty() {
            Vec::new()
        } else {
            overtime_rules::table
                .filter(overtime_rules::policy_id.eq_any(&policy_ids))
                .order((overtime_rules::policy_id, overtime_rules::priority.desc()))
                .select(OvertimeRuleEntity::as_select())
                .load(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?
        };

        let mut rules_by_policy: std::collections::HashMap<Uuid, Vec<OvertimeRule>> =
            std::collections::HashMap::new();
        for r in all_rules {
            rules_by_policy
                .entry(r.policy_id)
                .or_default()
                .push(r.into());
        }

        let mut policies_map = std::collections::HashMap::new();
        for p in policies_entities {
            let rules = rules_by_policy.remove(&p.id).unwrap_or_default();
            policies_map.insert(p.id, (p.into(), rules));
        }

        let assignments_entities: Vec<OvertimePolicyAssignmentEntity> =
            overtime_policy_assignments::table
                .filter(overtime_policy_assignments::workspace_id.eq(workspace_id))
                .select(OvertimePolicyAssignmentEntity::as_select())
                .load(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let assignments = assignments_entities.into_iter().map(Into::into).collect();

        Ok(
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                policies: policies_map,
                assignments,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::{DieselPools, TenantAuthToken};
    use crate::models::NewOvertimeRuleParams;
    use chrono::{NaiveDate, NaiveTime};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_overtime_policy_repository_flow() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://test_user:test_password@db_system:5432/leeattend_test_template".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselOvertimePolicyRepository::new(pools.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let mut conn = if let Ok(c) = pools.tenant_transaction_conn(&token).await {
            c
        } else {
            println!("Database not available, skipping integration assertions");
            return;
        };

        // 1. Insert Workspace to satisfy FK constraints
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

        // 2. Create policy definitions
        let policy = repo
            .create_policy_definition(ws_id, "Standard OT", Some("Standard rules"))
            .await
            .unwrap();
        assert_eq!(policy.name(), "Standard OT");

        let (list, _total) = repo.get_policy_definitions(ws_id, 1, 50).await.unwrap();
        assert!(list.iter().any(|p| p.id() == policy.id()));

        // 3. Add rules
        let rule = OvertimeRule::new(NewOvertimeRuleParams {
            id: Uuid::new_v4(),
            policy_id: policy.id(),
            priority: 1,
            multiplier: rust_decimal::Decimal::new(15, 1),
            time_start: NaiveTime::from_hms_opt(8, 0, 0),
            time_end: NaiveTime::from_hms_opt(17, 0, 0),
            days_of_week: 127,
            is_holiday: false,
            created_at: None,
            updated_at: None,
        });
        let added_rule = repo.add_rule(ws_id, &rule).await.unwrap();
        assert_eq!(added_rule.priority(), 1);

        // 4. Retrieve policy with rules
        let policy_with_rules = repo
            .get_policy_with_rules(ws_id, policy.id())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(policy_with_rules.0.id(), policy.id());
        assert_eq!(policy_with_rules.1.len(), 1);

        // 5. Assign policy to Workspace (default fallback)
        let assign_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        repo.create_policy_assignment(ws_id, "workspace", None, policy.id(), assign_date)
            .await
            .unwrap();

        // 6. Setup Employer, Department, and Employee to test override flows
        let employer_id = Uuid::new_v4();
        let dept_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        // Insert Employer
        diesel::sql_query("INSERT INTO employers (id, workspace_id, name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)")
            .bind::<diesel::sql_types::Uuid, _>(employer_id)
            .bind::<diesel::sql_types::Uuid, _>(ws_id)
            .bind::<diesel::sql_types::Text, _>("Test Employer")
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .execute(&mut *conn)
            .await
            .expect("insert employer");

        // Insert Department
        diesel::sql_query("INSERT INTO departments (id, workspace_id, name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)")
            .bind::<diesel::sql_types::Uuid, _>(dept_id)
            .bind::<diesel::sql_types::Uuid, _>(ws_id)
            .bind::<diesel::sql_types::Text, _>("Test Dept")
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .execute(&mut *conn)
            .await
            .expect("insert dept");

        // Insert Employee
        diesel::sql_query("INSERT INTO employees (id, workspace_id, employee_id_number, full_name, hire_date, original_hire_date, is_active) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Uuid, _>(ws_id)
            .bind::<diesel::sql_types::Text, _>("EMP-OT-TEST")
            .bind::<diesel::sql_types::Text, _>("Test OT Employee")
            .bind::<diesel::sql_types::Date, _>(assign_date)
            .bind::<diesel::sql_types::Date, _>(assign_date)
            .bind::<diesel::sql_types::Bool, _>(true)
            .execute(&mut *conn)
            .await
            .expect("insert employee");

        // Insert Employment
        diesel::sql_query("INSERT INTO employments (id, workspace_id, employee_id, employer_id, department_id, status, start_date, created_at, updated_at, employment_type) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
            .bind::<diesel::sql_types::Uuid, _>(employment_id)
            .bind::<diesel::sql_types::Uuid, _>(ws_id)
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Uuid, _>(employer_id)
            .bind::<diesel::sql_types::Uuid, _>(dept_id)
            .bind::<diesel::sql_types::Text, _>("active")
            .bind::<diesel::sql_types::Date, _>(assign_date)
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Text, _>("full_time")
            .execute(&mut *conn)
            .await
            .expect("insert employment");

        // 7. Resolve workspace fallback policy
        let resolved = repo
            .resolve_policy_for_employee(employee_id, ws_id, assign_date)
            .await
            .unwrap();
        assert_eq!(resolved, Some(policy.id()));

        // 8. Assign Department Override policy and resolve
        let policy_dept = repo
            .create_policy_definition(ws_id, "Dept OT", Some("Dept rules"))
            .await
            .unwrap();
        repo.create_policy_assignment(
            ws_id,
            "department",
            Some(dept_id),
            policy_dept.id(),
            assign_date,
        )
        .await
        .unwrap();

        let resolved_dept = repo
            .resolve_policy_for_employee(employee_id, ws_id, assign_date)
            .await
            .unwrap();
        assert_eq!(resolved_dept, Some(policy_dept.id()));

        // 9. Assign Employee Override policy and resolve
        let policy_emp = repo
            .create_policy_definition(ws_id, "Emp OT", Some("Emp rules"))
            .await
            .unwrap();
        repo.create_policy_assignment(
            ws_id,
            "employee",
            Some(employee_id),
            policy_emp.id(),
            assign_date,
        )
        .await
        .unwrap();

        let resolved_emp = repo
            .resolve_policy_for_employee(employee_id, ws_id, assign_date)
            .await
            .unwrap();
        assert_eq!(resolved_emp, Some(policy_emp.id()));

        // 10. Get Workspace Overtime Data
        let data = repo.get_workspace_overtime_data(ws_id).await.unwrap();
        assert!(data.policies.contains_key(&policy.id()));
        assert!(data.policies.contains_key(&policy_dept.id()));
        assert!(data.policies.contains_key(&policy_emp.id()));
        assert_eq!(data.assignments.len(), 3);
    }
}
