use crate::errors::AppError;
use crate::models::diesel::monitor_alert::{MonitorAlert, MonitorAlertEvent};
use crate::repositories::monitor_alert_repository::{
    ListMonitorAlertsParams, MonitorAlertAction, MonitorAlertRepository,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct MonitorAlertService {
    repository: Arc<dyn MonitorAlertRepository>,
}

impl MonitorAlertService {
    pub fn new(repository: Arc<dyn MonitorAlertRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_alerts(
        &self,
        params: ListMonitorAlertsParams,
    ) -> Result<Vec<MonitorAlert>, AppError> {
        self.repository.list_alerts(params).await
    }

    pub async fn mutate_alert(
        &self,
        alert_id: Uuid,
        action: MonitorAlertAction,
        reason: String,
    ) -> Result<MonitorAlert, AppError> {
        self.repository
            .mutate_alert(alert_id, action, reason, chrono::Utc::now())
            .await
    }

    pub async fn get_alert(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<(MonitorAlert, Vec<MonitorAlertEvent>)>, AppError> {
        let Some(alert) = self.repository.get_alert(alert_id).await? else {
            return Ok(None);
        };
        let events = self.repository.list_events(alert_id, 200).await?;
        Ok(Some((alert, events)))
    }
}
