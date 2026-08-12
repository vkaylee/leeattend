use crate::diesel_pool::{DieselPools, TransactionConn};
use crate::errors::AppError;
use crate::models::diesel::monitor_alert::{MonitorAlert, MonitorAlertEvent};
use crate::repositories::monitor_alert_repository::{
    ListMonitorAlertsParams, MonitorAlertRepository,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel::sql_query;
use uuid::Uuid;

#[derive(Clone)]
pub struct DieselMonitorAlertRepository {
    pool: DieselPools,
}

impl DieselMonitorAlertRepository {
    pub fn new(pool: DieselPools) -> Self {
        Self { pool }
    }

    async fn api_conn(&self) -> Result<TransactionConn, AppError> {
        Ok(TransactionConn(self.pool.api.get().await.map_err(|error| {
            AppError::Internal(format!("Failed to get system DB connection: {error}"))
        })?))
    }
}

#[async_trait]
impl MonitorAlertRepository for DieselMonitorAlertRepository {
    async fn list_alerts(
        &self,
        params: ListMonitorAlertsParams,
    ) -> Result<Vec<MonitorAlert>, AppError> {
        let mut conn = self.api_conn().await?;
        let limit = params.limit.clamp(1, 100);
        let mut query = crate::schema::monitor_alerts::table.into_boxed();
        if let Some(status) = params.status {
            query = query.filter(crate::schema::monitor_alerts::status.eq(status));
        }
        if let Some(severity) = params.severity {
            query = query.filter(crate::schema::monitor_alerts::severity.eq(severity));
        }
        query
            .order((
                crate::schema::monitor_alerts::last_seen_at.desc(),
                crate::schema::monitor_alerts::id.desc(),
            ))
            .limit(limit)
            .load::<MonitorAlert>(&mut *conn)
            .await
            .map_err(|error| AppError::Internal(format!("Failed to list monitor alerts: {error}")))
    }

    async fn get_alert(&self, alert_id: Uuid) -> Result<Option<MonitorAlert>, AppError> {
        let mut conn = self.api_conn().await?;
        crate::schema::monitor_alerts::table
            .filter(crate::schema::monitor_alerts::id.eq(alert_id))
            .first::<MonitorAlert>(&mut *conn)
            .await
            .optional()
            .map_err(|error| AppError::Internal(format!("Failed to read monitor alert: {error}")))
    }

    async fn list_events(
        &self,
        alert_id: Uuid,
        limit: i64,
    ) -> Result<Vec<MonitorAlertEvent>, AppError> {
        let mut conn = self.api_conn().await?;
        crate::schema::monitor_alert_events::table
            .filter(crate::schema::monitor_alert_events::alert_id.eq(alert_id))
            .order((
                crate::schema::monitor_alert_events::occurred_at.asc(),
                crate::schema::monitor_alert_events::id.asc(),
            ))
            .limit(limit.clamp(1, 200))
            .load::<MonitorAlertEvent>(&mut *conn)
            .await
            .map_err(|error| AppError::Internal(format!("Failed to list monitor alert events: {error}")))
    }

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
    ) -> Result<MonitorAlert, AppError> {
        let mut conn = self.api_conn().await?;
        let alert_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();
        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                let alert = diesel::sql_query(
                    "INSERT INTO monitor_alerts
                     (id, fingerprint, alert_type, severity, status, title, summary, source,
                      first_seen_at, last_seen_at, occurrence_count, created_at, updated_at)
                     VALUES ($1,$2,$3,$4,'active',$5,$6,$7,$8,$8,1,$8,$8)
                     ON CONFLICT (fingerprint) WHERE resolved_at IS NULL
                     DO UPDATE SET severity = EXCLUDED.severity, title = EXCLUDED.title,
                       summary = EXCLUDED.summary, source = EXCLUDED.source,
                       last_seen_at = EXCLUDED.last_seen_at,
                       occurrence_count = monitor_alerts.occurrence_count + 1,
                       updated_at = EXCLUDED.updated_at
                     RETURNING id, fingerprint, alert_type, severity, status, title, summary,
                       source, first_seen_at, last_seen_at, occurrence_count, resolved_at,
                       created_at, updated_at",
                )
                .bind::<diesel::sql_types::Uuid, _>(alert_id)
                .bind::<diesel::sql_types::Varchar, _>(&fingerprint)
                .bind::<diesel::sql_types::Varchar, _>(&alert_type)
                .bind::<diesel::sql_types::Varchar, _>(&severity)
                .bind::<diesel::sql_types::Varchar, _>(&title)
                .bind::<diesel::sql_types::Text, _>(&summary)
                .bind::<diesel::sql_types::Varchar, _>(&source)
                .bind::<diesel::sql_types::Timestamptz, _>(occurred_at)
                .get_result::<MonitorAlert>(&mut *conn)
                .await
                .map_err(|error| AppError::Internal(format!("Failed to upsert monitor alert: {error}")))?;

                diesel::sql_query(
                    "INSERT INTO monitor_alert_events
                     (id, alert_id, event_type, severity, summary, occurred_at, created_at)
                     VALUES ($1,$2,$3,$4,$5,$6,$6)",
                )
                .bind::<diesel::sql_types::Uuid, _>(event_id)
                .bind::<diesel::sql_types::Uuid, _>(alert.id)
                .bind::<diesel::sql_types::Varchar, _>(&event_type)
                .bind::<diesel::sql_types::Varchar, _>(&severity)
                .bind::<diesel::sql_types::Text, _>(&summary)
                .bind::<diesel::sql_types::Timestamptz, _>(occurred_at)
                .execute(&mut *conn)
                .await
                .map_err(|error| AppError::Internal(format!("Failed to append monitor alert event: {error}")))?;

                Ok(alert)
            })
        })
        .await
    }
}
