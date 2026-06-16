use std::sync::Arc;

use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

use crate::utils::id::new_id;

use crate::models::{
    AdjustmentStatus, AttendanceAdjustmentRequest, CreateAttendanceAdjustmentRequest, TimeEntry,
};
use crate::repositories::attendance_adjustment_repository::AttendanceAdjustmentRepository;
use crate::repositories::TimeEntryRepository;

pub struct AttendanceAdjustmentService {
    adj_repo: Arc<dyn AttendanceAdjustmentRepository>,
    time_entry_repo: Arc<dyn TimeEntryRepository>,
}

impl AttendanceAdjustmentService {
    pub fn new(
        adj_repo: Arc<dyn AttendanceAdjustmentRepository>,
        time_entry_repo: Arc<dyn TimeEntryRepository>,
    ) -> Self {
        Self {
            adj_repo,
            time_entry_repo,
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn submit_request(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        req: CreateAttendanceAdjustmentRequest,
    ) -> Result<AttendanceAdjustmentRequest, AppError> {
        let attachment_file_ids = req
            .attachment_file_ids
            .map(|ids| ids.into_iter().map(Some).collect());

        let adjustment = AttendanceAdjustmentRequest::new(
            new_id(),
            req.employment_id,
            req.time_entry_id,
            req.requested_punch_in_time,
            req.requested_punch_out_time,
            req.is_break_overtime,
            req.reason,
            AdjustmentStatus::Pending,
            Some(user_id),
            None,
            Utc::now(),
            Utc::now(),
            attachment_file_ids,
        );

        self.adj_repo
            .create_request(workspace_id, &adjustment)
            .await
            .map_err(|e| {
                error!("SUBMIT_ADJUSTMENT_ERROR: {:?}", e);
                AppError::Internal(e.to_string())
            })?;

        Ok(adjustment)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn approve_request(
        &self,
        workspace_id: Uuid,
        approver_id: Uuid,
        request_id: Uuid,
        status: AdjustmentStatus,
    ) -> Result<(), AppError> {
        let request = self
            .adj_repo
            .get_request_by_id(workspace_id, request_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

        self.adj_repo
            .update_adjustment_status(workspace_id, request_id, status, Some(approver_id))
            .await?;

        if status == AdjustmentStatus::Approved {
            if let Some(time_entry_id) = request.time_entry_id() {
                let mut entry = self
                    .time_entry_repo
                    .get_time_entry_by_id(
                        &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                        time_entry_id,
                    )
                    .await?
                    .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?;

                entry.set_punch_in_time(request.requested_punch_in_time());
                entry.set_punch_out_time(request.requested_punch_out_time());
                entry.set_is_break_overtime(request.is_break_overtime());

                self.time_entry_repo
                    .update_time_entry(
                        &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                        &entry,
                    )
                    .await?;
            } else {
                let duration_minutes = request.requested_punch_out_time().map(|punch_out| {
                    let duration =
                        punch_out.signed_duration_since(request.requested_punch_in_time());
                    duration.num_minutes() as i32
                });

                let entry = TimeEntry::new(
                    new_id(),
                    request.employment_id(),
                    workspace_id,
                    None,
                    request.requested_punch_in_time(),
                    request.requested_punch_out_time(),
                    duration_minutes,
                    request.is_break_overtime(),
                    "work".to_string(),
                    request.reason().map(|r| format!("Adjustment: {r}")),
                    Some(serde_json::json!({ "source": "adjustment" })),
                    Utc::now(),
                    Utc::now(),
                );

                self.time_entry_repo
                    .create_time_entry(
                        &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                        &entry,
                    )
                    .await?;
            }
        }

        Ok(())
    }

    pub fn parse_datetime(
        dt_str: &str,
        timezone: &str,
        preferred_date: Option<&str>,
        preferred_time: Option<&str>,
    ) -> Result<chrono::DateTime<chrono::Utc>, AppError> {
        crate::utils::datetime::parse_datetime_with_tz(
            dt_str,
            timezone,
            preferred_date,
            preferred_time,
        )
        .map_err(AppError::validation_raw)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_workspace_requests(
        &self,
        workspace_id: Uuid,
        status: Option<AdjustmentStatus>,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, AppError> {
        self.adj_repo
            .get_workspace_adjustment_requests(workspace_id, status)
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_my_requests(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, AppError> {
        self.adj_repo
            .get_employment_adjustment_requests(workspace_id, employment_id)
            .await
    }
}

#[async_trait::async_trait]
impl crate::services::approval_workflow::WorkflowCallbackHandler for AttendanceAdjustmentService {
    async fn on_workflow_completed(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_id: Uuid,
        status: &str,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let adj_status = match status {
            "approved" => crate::models::AdjustmentStatus::Approved,
            "rejected" => crate::models::AdjustmentStatus::Rejected,
            _ => crate::models::AdjustmentStatus::Pending,
        };
        self.approve_request(workspace_id, user_id, entity_id, adj_status)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_parse_datetime_ambiguity() {
        // Case 1: Preferred DD/MM/YYYY, input 01/05/2026 (May 1st)
        let dt = AttendanceAdjustmentService::parse_datetime(
            "01/05/2026 14:00",
            "Asia/Ho_Chi_Minh",
            Some("DD/MM/YYYY"),
            Some("24h"),
        );
        assert!(dt.is_ok());
        let dt = dt.expect("parsing failed");
        // 14:00 VN = 07:00 UTC
        assert_eq!(dt.hour(), 7);
    }

    #[test]
    fn test_parse_datetime_iso() {
        // ISO with Z should ignore workspace tz
        let dt = AttendanceAdjustmentService::parse_datetime(
            "2026-01-14T08:30:00Z",
            "Asia/Ho_Chi_Minh",
            None,
            None,
        );
        assert!(dt.is_ok());
        assert_eq!(dt.expect("parsing failed").hour(), 8);
    }

    // =====================================================================
    // Additional parse_datetime edge cases
    // =====================================================================

    #[test]
    fn test_parse_datetime_invalid_string() {
        let dt = AttendanceAdjustmentService::parse_datetime(
            "not-a-date",
            "Asia/Ho_Chi_Minh",
            Some("DD/MM/YYYY"),
            Some("24h"),
        );
        assert!(dt.is_err());
    }

    #[test]
    fn test_parse_datetime_empty_string() {
        let dt = AttendanceAdjustmentService::parse_datetime(
            "",
            "Asia/Ho_Chi_Minh",
            Some("DD/MM/YYYY"),
            Some("24h"),
        );
        assert!(dt.is_err());
    }

    #[test]
    fn test_parse_datetime_iso_with_offset() {
        let dt = AttendanceAdjustmentService::parse_datetime(
            "2026-06-15T14:30:00+07:00",
            "UTC",
            None,
            None,
        );
        assert!(dt.is_ok());
        let dt = dt.unwrap();
        // 14:30 +07:00 = 07:30 UTC
        assert_eq!(dt.hour(), 7);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_datetime_utc_timezone() {
        let dt = AttendanceAdjustmentService::parse_datetime(
            "2026-01-14 08:30",
            "UTC",
            None,
            Some("24h"),
        );
        assert!(dt.is_ok());
        assert_eq!(dt.unwrap().hour(), 8);
    }

    #[test]
    fn test_parse_datetime_mm_dd_yyyy_format() {
        let dt = AttendanceAdjustmentService::parse_datetime(
            "05/01/2026 14:00",
            "Asia/Ho_Chi_Minh",
            Some("MM/DD/YYYY"),
            Some("24h"),
        );
        assert!(dt.is_ok());
        let dt = dt.unwrap();
        // May 1st, 14:00 VN = 07:00 UTC
        assert_eq!(dt.hour(), 7);
    }
}
