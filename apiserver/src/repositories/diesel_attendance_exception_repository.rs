use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::models::diesel::attendance_exception::AttendanceExceptionEntity;
use crate::repositories::attendance_exception_repository::AttendanceExceptionRepository;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[derive(Clone)]
pub struct DieselAttendanceExceptionRepository {
    diesel_pools: DieselPools,
}

impl DieselAttendanceExceptionRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl AttendanceExceptionRepository for DieselAttendanceExceptionRepository {
    async fn get_exception_for_date(
        &self,
        auth_token: &TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<AttendanceExceptionEntity>, AppError> {
        use crate::schema::attendance_exceptions::dsl as ae;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        let result = ae::attendance_exceptions
            .filter(ae::workspace_id.eq(auth_token.workspace_id()))
            .filter(ae::employee_id.eq(employee_id))
            .filter(ae::exception_from.le(date))
            .filter(ae::exception_to.ge(date))
            .select(AttendanceExceptionEntity::as_select())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("fetch exception: {e}")))?;

        Ok(result)
    }
}
