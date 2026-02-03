# Phase 3: Repository Updates

## Overview
Create StagingLogRepository and update ErrorLogRepository with fingerprint support.

## Priority
High - Required for Phase 2 logic

## Related Files
- `apiserver/src/repositories/mod.rs` - Export new module
- `apiserver/src/repositories/staging_log_repository.rs` - NEW
- `apiserver/src/repositories/attendance_log_error_repository.rs` - Update

## Implementation

### New: `staging_log_repository.rs`

```rust
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_staging_log(
    pool: &PgPool,
    workspace_id: Uuid,
    device_id: Option<String>,
    raw_user_id: &str,
    punch_time_raw: &str,
    raw_data: Option<serde_json::Value>,
    fingerprint: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO attendance_staging_logs
        (workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint)
        VALUES ($1, $2, $3, $4::TIMESTAMPTZ, $5, $6)
        ON CONFLICT (workspace_id, fingerprint) DO NOTHING
        "#
    )
    .bind(workspace_id)
    .bind(device_id)
    .bind(raw_user_id)
    .bind(punch_time_raw)
    .bind(raw_data)
    .bind(fingerprint)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_pending_staging_logs(
    pool: &PgPool,
    workspace_id: Uuid,
    limit: i64,
) -> Result<Vec<StagingLog>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT id, workspace_id, device_id, raw_user_id, punch_time, raw_data, fingerprint, status, created_at
        FROM attendance_staging_logs
        WHERE workspace_id = $1 AND status = 'pending'
        ORDER BY created_at ASC
        LIMIT $2
        "#
    )
    .bind(workspace_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn update_staging_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE attendance_staging_logs SET status = $1 WHERE id = $2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
```

### Update: `attendance_log_error_repository.rs`

Add fingerprint-aware version:

```rust
pub async fn create_error_log_with_fingerprint(
    pool: &PgPool,
    workspace_id: Uuid,
    agent_id: Option<String>,
    raw_content: Option<serde_json::Value>,
    error_message: &str,
    source: &str,
    fingerprint: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO attendance_log_errors
        (workspace_id, agent_id, raw_content, error_message, source, fingerprint)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (workspace_id, fingerprint) WHERE fingerprint IS NOT NULL DO NOTHING
        "#
    )
    .bind(workspace_id)
    .bind(agent_id)
    .bind(raw_content)
    .bind(error_message)
    .bind(source)
    .bind(fingerprint)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
```

### Model: Add StagingLog struct

In `models.rs`:

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StagingLog {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub device_id: Option<String>,
    pub raw_user_id: String,
    pub punch_time: DateTime<Utc>,
    pub raw_data: Option<serde_json::Value>,
    pub fingerprint: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
```

## Todo
- [ ] Create staging_log_repository.rs
- [ ] Add StagingLog model
- [ ] Update attendance_log_error_repository.rs
- [ ] Export in repositories/mod.rs
- [ ] Update service imports

## Success Criteria
- create_staging_log works with ON CONFLICT DO NOTHING
- create_error_log_with_fingerprint works with ON CONFLICT
- get_pending_staging_logs retrieves pending items
