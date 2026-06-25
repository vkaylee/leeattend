use crate::errors::AppError;
use crate::models::{
    CreateWorkSegmentRequest, UpdateWorkSegmentRequest, WorkSegment, WorkSegmentFilter,
};
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WorkSegmentRepository: Send + Sync {
    async fn create_segment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &CreateWorkSegmentRequest,
    ) -> Result<WorkSegment, AppError>;

    async fn get_segment_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        segment_id: Uuid,
    ) -> Result<Option<WorkSegment>, AppError>;

    async fn list_segments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        filter: &WorkSegmentFilter,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<WorkSegment>, i64), AppError>;

    async fn get_segments_by_date_range(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<WorkSegment>, AppError>;

    async fn update_segment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        segment_id: Uuid,
        req: &UpdateWorkSegmentRequest,
    ) -> Result<WorkSegment, AppError>;

    async fn delete_segment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        segment_id: Uuid,
    ) -> Result<(), AppError>;

    async fn bulk_create_segments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        segments: &[CreateWorkSegmentRequest],
        replace_existing: bool,
    ) -> Result<Vec<WorkSegment>, AppError>;

    async fn get_schedule(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<WorkSegment>, AppError>;
}
