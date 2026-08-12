use crate::schema::{monitor_alert_events, monitor_alerts};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, QueryableByName, Selectable)]
#[diesel(table_name = monitor_alerts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MonitorAlert {
    pub id: Uuid,
    pub fingerprint: String,
    pub alert_type: String,
    pub severity: String,
    pub status: String,
    pub title: String,
    pub summary: String,
    pub source: String,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub occurrence_count: i32,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, QueryableByName, Selectable)]
#[diesel(table_name = monitor_alert_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MonitorAlertEvent {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub summary: String,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
