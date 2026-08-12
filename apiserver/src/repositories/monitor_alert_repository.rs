use crate::errors::AppError;
use crate::models::diesel::monitor_alert::{MonitorAlert, MonitorAlertEvent};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MonitorAlertCursor {
    pub last_seen_at: DateTime<Utc>,
    pub id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct ListMonitorAlertsParams {
    pub status: Option<String>,
    pub severity: Option<String>,
    pub cursor: Option<MonitorAlertCursor>,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct MonitorAlertPage {
    pub alerts: Vec<MonitorAlert>,
    pub total: i64,
    pub next_cursor: Option<MonitorAlertCursor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorAlertAction {
    Acknowledge,
    Resolve,
}

#[async_trait]
pub trait MonitorAlertRepository: Send + Sync {
    async fn list_alerts(
        &self,
        params: ListMonitorAlertsParams,
    ) -> Result<MonitorAlertPage, AppError>;

    async fn get_alert(&self, alert_id: Uuid) -> Result<Option<MonitorAlert>, AppError>;

    async fn mutate_alert(
        &self,
        alert_id: Uuid,
        action: MonitorAlertAction,
        reason: String,
        actor_id: Uuid,
        occurred_at: DateTime<Utc>,
    ) -> Result<MonitorAlert, AppError>;

    async fn list_events(
        &self,
        alert_id: Uuid,
        limit: i64,
    ) -> Result<Vec<MonitorAlertEvent>, AppError>;

    #[allow(dead_code, clippy::too_many_arguments)]
    async fn insert_or_refresh_active(
        &self,
        fingerprint: String,
        alert_type: String,
        severity: String,
        title: String,
        summary: String,
        source: String,
        event_type: String,
        occurred_at: DateTime<Utc>,
    ) -> Result<MonitorAlert, AppError>;
}
