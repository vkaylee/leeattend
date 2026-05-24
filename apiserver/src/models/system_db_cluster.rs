use crate::schema::system_db_clusters;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = system_db_clusters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SystemDbCluster {
    pub id: Uuid,
    pub name: String,
    pub admin_database_url: String,
    pub region: String,
    pub max_tenants: i32,
    pub active_tenants: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = system_db_clusters)]
pub struct NewSystemDbCluster {
    pub id: Uuid,
    pub name: String,
    pub admin_database_url: String,
    pub region: String,
    pub max_tenants: i32,
    pub active_tenants: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
