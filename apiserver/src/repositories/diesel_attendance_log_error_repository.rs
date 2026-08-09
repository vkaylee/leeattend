use crate::diesel_pool::DieselPoolsTenantTxExt;
use async_trait::async_trait;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::errors::AppError;
use crate::models::diesel::attendance_log::AttendanceLogErrorEntity;
use crate::repositories::attendance_log_error_repository::AttendanceLogErrorRepository;
use crate::schema::attendance_log_errors;

pub struct DieselAttendanceLogErrorRepository {
    diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselAttendanceLogErrorRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl AttendanceLogErrorRepository for DieselAttendanceLogErrorRepository {
    async fn create_error_log(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn: &mut AsyncPgConnection| {
                Box::pin(async move {
                    let entity = AttendanceLogErrorEntity {
                        id: crate::utils::id::new_id(),
                        workspace_id,
                        agent_id,
                        raw_content,
                        error_message: error_message.to_string(),
                        source: source.to_string(),
                        created_at: Some(chrono::Utc::now()),
                        fingerprint: None,
                    };

                    diesel::insert_into(attendance_log_errors::table)
                        .values(&entity)
                        .execute(&mut *conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(())
                })
            })
            .await
    }

    async fn create_error_log_with_fingerprint(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        agent_id: Option<String>,
        raw_content: Option<serde_json::Value>,
        error_message: &str,
        source: &str,
        fingerprint: &str,
    ) -> Result<bool, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn: &mut AsyncPgConnection| {
                Box::pin(async move {
                    let entity = AttendanceLogErrorEntity {
                        id: crate::utils::id::new_id(),
                        workspace_id,
                        agent_id,
                        raw_content,
                        error_message: error_message.to_string(),
                        source: source.to_string(),
                        created_at: Some(chrono::Utc::now()),
                        fingerprint: Some(fingerprint.to_string()),
                    };

                    let rows_affected = diesel::insert_into(attendance_log_errors::table)
                        .values(&entity)
                        .on_conflict((
                            attendance_log_errors::workspace_id,
                            attendance_log_errors::fingerprint,
                        ))
                        .do_nothing()
                        .execute(&mut *conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(rows_affected > 0)
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::{DieselPools, TenantAuthToken};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_repo_errors() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselAttendanceLogErrorRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let auth_token = TenantAuthToken(ws_id);

        let _ = repo
            .create_error_log(&auth_token, None, None, "test", "device")
            .await;
        let _ = repo
            .create_error_log_with_fingerprint(&auth_token, None, None, "test", "device", "fprint")
            .await;
    }
}
