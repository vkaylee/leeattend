use super::DieselWorkspaceRepository;
use crate::errors::AppError;
use crate::models::diesel::core::{WorkspaceEntity, WorkspaceMemberEntity};
use crate::models::Workspace;
use crate::schema::{workspace_members, workspaces};
use crate::utils::i18n::I18nKey;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

impl DieselWorkspaceRepository {
    pub(super) async fn create_workspace_with_owner_impl(
        &self,
        id: Uuid,
        name: &str,
        owner_id: Uuid,
        custom_db_url_encrypted: Option<String>,
    ) -> Result<Workspace, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = WorkspaceEntity {
            id,
            name: name.to_string(),
            created_at: Some(Utc::now()),
            date_format: "yyyy-MM-dd".to_string(),
            time_format: "HH:mm".to_string(),
            timezone: "UTC".to_string(),
            theme_primary_color: Some("#1890ff".to_string()),
            default_language: "en".to_string(),
            logo_url: None,
            tagline: None,
            custom_db_url_encrypted,
        };

        let owner_role_id = Uuid::parse_str("a0000000-0000-0000-0000-000000000001")
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let member_entity = WorkspaceMemberEntity {
            workspace_id: id,
            user_id: owner_id,
            role_id: owner_role_id,
        };

        let entity_clone = entity.clone();
        conn.transaction(|conn| {
            Box::pin(async move {
                diesel::insert_into(workspaces::table)
                    .values(&entity)
                    .execute(&mut *conn)
                    .await?;

                diesel::insert_into(workspace_members::table)
                    .values(&member_entity)
                    .execute(&mut *conn)
                    .await?;

                Ok::<_, diesel::result::Error>(())
            })
        })
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity_clone.into())
    }

    pub(super) async fn get_workspace_by_id_impl(
        &self,
        id: Uuid,
    ) -> Result<Option<Workspace>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = workspaces::table
            .find(id)
            .select(WorkspaceEntity::as_select())
            .first::<WorkspaceEntity>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(entity.map(Into::into))
    }

    pub(super) async fn get_all_workspaces_impl(&self) -> Result<Vec<Workspace>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results = workspaces::table
            .select(WorkspaceEntity::as_select())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    pub(super) async fn get_user_workspaces_impl(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Workspace>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::workspace_members;

        let results: Vec<WorkspaceEntity> = workspaces::table
            .inner_join(
                workspace_members::table.on(workspaces::id.eq(workspace_members::workspace_id)),
            )
            .filter(workspace_members::user_id.eq(user_id))
            .select(WorkspaceEntity::as_select())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    pub(super) async fn clear_workspace_data_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        group: &str,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        conn.transaction(|conn| {
            Box::pin(async move {
                let ws_id = workspace_id;

                match group {
                    "attendance" => clear_attendance_data(conn, ws_id).await?,
                    "schedules" => clear_schedule_data(conn, ws_id).await?,
                    "leave" => clear_leave_data(conn, ws_id).await?,
                    "payroll" => clear_payroll_data(conn, ws_id).await?,
                    "full_reset" => {
                        // Order matters: children before parents to satisfy FK constraints.
                        clear_attendance_data(conn, ws_id).await?;
                        clear_schedule_data(conn, ws_id).await?;
                        clear_leave_data(conn, ws_id).await?;
                        clear_payroll_data(conn, ws_id).await?;
                        clear_org_structure(conn, ws_id).await?;
                    }
                    _ => return Err(diesel::result::Error::RollbackTransaction),
                }
                Ok::<_, diesel::result::Error>(())
            })
        })
        .await
        .map_err(|e| match e {
            diesel::result::Error::RollbackTransaction => {
                AppError::validation(I18nKey::InvalidGroup)
            }
            e => AppError::Internal(format!("db error: {e}")),
        })?;

        Ok(())
    }
}

/// Subquery selecting every employment id owned by `ws_id`.
///
/// Child tables key off `employment_id` rather than `workspace_id`, so deletes
/// scope through this instead of filtering directly.
macro_rules! employment_ids {
    ($ws_id:expr) => {
        crate::schema::employments::dsl::employments
            .filter(crate::schema::employments::dsl::workspace_id.eq($ws_id))
            .select(crate::schema::employments::dsl::id)
    };
}

async fn clear_attendance_data(
    conn: &mut AsyncPgConnection,
    ws_id: Uuid,
) -> Result<(), diesel::result::Error> {
    use crate::schema::{
        attendance_adjustment_requests, attendance_import_jobs, raw_attendance_logs, time_entries,
    };

    diesel::delete(
        time_entries::dsl::time_entries
            .filter(time_entries::dsl::employment_id.eq_any(employment_ids!(ws_id))),
    )
    .execute(conn)
    .await?;
    diesel::delete(
        raw_attendance_logs::dsl::raw_attendance_logs
            .filter(raw_attendance_logs::dsl::workspace_id.eq(ws_id)),
    )
    .execute(conn)
    .await?;
    diesel::delete(
        attendance_adjustment_requests::dsl::attendance_adjustment_requests.filter(
            attendance_adjustment_requests::dsl::employment_id.eq_any(employment_ids!(ws_id)),
        ),
    )
    .execute(conn)
    .await?;
    diesel::delete(
        attendance_import_jobs::dsl::attendance_import_jobs
            .filter(attendance_import_jobs::dsl::workspace_id.eq(ws_id)),
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn clear_schedule_data(
    conn: &mut AsyncPgConnection,
    ws_id: Uuid,
) -> Result<(), diesel::result::Error> {
    use crate::schema::{employee_shifts, shift_rosters};

    diesel::delete(
        shift_rosters::dsl::shift_rosters
            .filter(shift_rosters::dsl::employment_id.eq_any(employment_ids!(ws_id))),
    )
    .execute(conn)
    .await?;
    diesel::delete(
        employee_shifts::dsl::employee_shifts
            .filter(employee_shifts::dsl::employment_id.eq_any(employment_ids!(ws_id))),
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn clear_leave_data(
    conn: &mut AsyncPgConnection,
    ws_id: Uuid,
) -> Result<(), diesel::result::Error> {
    use crate::schema::{leave_accrual_logs, leave_balances, leave_requests};

    diesel::delete(
        leave_accrual_logs::dsl::leave_accrual_logs
            .filter(leave_accrual_logs::dsl::workspace_id.eq(ws_id)),
    )
    .execute(conn)
    .await?;
    diesel::delete(
        leave_requests::dsl::leave_requests
            .filter(leave_requests::dsl::employment_id.eq_any(employment_ids!(ws_id))),
    )
    .execute(conn)
    .await?;
    diesel::delete(
        leave_balances::dsl::leave_balances
            .filter(leave_balances::dsl::employment_id.eq_any(employment_ids!(ws_id))),
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn clear_payroll_data(
    conn: &mut AsyncPgConnection,
    ws_id: Uuid,
) -> Result<(), diesel::result::Error> {
    use crate::schema::{employee_salary_components, payslips, salary_component_definitions};

    diesel::delete(
        payslips::dsl::payslips.filter(payslips::dsl::employment_id.eq_any(employment_ids!(ws_id))),
    )
    .execute(conn)
    .await?;
    diesel::delete(
        employee_salary_components::dsl::employee_salary_components
            .filter(employee_salary_components::dsl::employment_id.eq_any(employment_ids!(ws_id))),
    )
    .execute(conn)
    .await?;
    diesel::delete(
        salary_component_definitions::dsl::salary_component_definitions
            .filter(salary_component_definitions::dsl::workspace_id.eq(ws_id)),
    )
    .execute(conn)
    .await?;

    Ok(())
}

/// Deletes policy, contract, and org-hierarchy rows. Only used by `full_reset`,
/// which runs it last so all employment-scoped children are already gone.
async fn clear_org_structure(
    conn: &mut AsyncPgConnection,
    ws_id: Uuid,
) -> Result<(), diesel::result::Error> {
    use crate::schema::{
        departments, employee_assignments, employee_contracts, employees, employments,
        leave_policy_assignments, leave_policy_definitions, shifts,
    };

    diesel::delete(
        leave_policy_assignments::dsl::leave_policy_assignments
            .filter(leave_policy_assignments::dsl::workspace_id.eq(ws_id)),
    )
    .execute(conn)
    .await?;
    diesel::delete(
        leave_policy_definitions::dsl::leave_policy_definitions
            .filter(leave_policy_definitions::dsl::workspace_id.eq(ws_id)),
    )
    .execute(conn)
    .await?;

    diesel::delete(
        employee_contracts::dsl::employee_contracts
            .filter(employee_contracts::dsl::employment_id.eq_any(employment_ids!(ws_id))),
    )
    .execute(conn)
    .await?;
    diesel::delete(
        employee_assignments::dsl::employee_assignments
            .filter(employee_assignments::dsl::employment_id.eq_any(employment_ids!(ws_id))),
    )
    .execute(conn)
    .await?;

    diesel::delete(employments::dsl::employments.filter(employments::dsl::workspace_id.eq(ws_id)))
        .execute(conn)
        .await?;

    diesel::delete(employees::dsl::employees.filter(employees::dsl::workspace_id.eq(ws_id)))
        .execute(conn)
        .await?;
    diesel::delete(shifts::dsl::shifts.filter(shifts::dsl::workspace_id.eq(ws_id)))
        .execute(conn)
        .await?;
    diesel::delete(departments::dsl::departments.filter(departments::dsl::workspace_id.eq(ws_id)))
        .execute(conn)
        .await?;

    Ok(())
}
