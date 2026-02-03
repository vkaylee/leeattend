# Phase 4: Application-Side Join

## Context Links
- [Plan Overview](./plan.md)
- [Phase 3: Generic Worker](./phase-03-generic-worker.md)
- [Scout Report](../reports/scout-260130-1711-attendance-report-architecture.md)

## Overview
- **Priority**: P2 (Optimization)
- **Status**: pending
- **Effort**: 4h
- **Description**: Optimize data fetching by caching static data in Redis and performing joins in application code.

## Key Insights

- **Static data** (shifts, policies, patterns) changes rarely - cache in Redis
- **Dynamic data** (time entries, raw logs) must come from DB
- Application-side join reduces DB query complexity and CPU usage
- Cache warm-up at worker startup, refresh on TTL

## Requirements

### Functional
- F1: Cache workspace settings, shifts, policies in Redis
- F2: Fetch dynamic data (time entries) from PostgreSQL
- F3: Perform join in application memory
- F4: Cache invalidation on data change (optional webhook)

### Non-Functional
- NF1: Cache hit rate > 95% for static data
- NF2: Cache TTL 5 minutes (balance freshness vs performance)
- NF3: DB query count reduced by 50%
- NF4: DB CPU usage < 80% under load

## Architecture

### Data Classification

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        DATA CLASSIFICATION                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  STATIC (Cache in Redis)           │  DYNAMIC (Query from PostgreSQL)       │
│  ─────────────────────────          │  ────────────────────────────────       │
│  TTL: 5 minutes                    │  Always fresh                          │
│                                     │                                         │
│  • Workspace settings              │  • raw_attendance_logs                 │
│    - timezone                      │  • time_entries                         │
│    - rounding_policy               │  • attendance_daily_summaries          │
│    - work_unit_minutes             │  • leave_requests                      │
│    - early_arrival_strategy        │                                         │
│                                     │                                         │
│  • Shifts (per workspace)          │                                         │
│    - id, name, start, end, break   │                                         │
│                                     │                                         │
│  • Working patterns                │                                         │
│    - week definitions              │                                         │
│                                     │                                         │
│  • Policy rules                    │                                         │
│    - default rules per workspace   │                                         │
│                                     │                                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Cache Key Schema

```
# Workspace config
cache:workspace:{workspace_id}
  → { timezone, rounding_policy, work_unit_minutes, ... }

# All shifts for workspace
cache:shifts:{workspace_id}
  → [{ id, name, start_time, end_time, break_duration, ... }, ...]

# Working patterns
cache:patterns:{workspace_id}
  → { pattern_id: { weeks: [...] }, ... }

# Default policy rules
cache:policy:{workspace_id}
  → { grace_period_minutes, ... }

# Employee basic info (for name lookup)
cache:employees:{workspace_id}
  → { employee_id: { full_name, department_id }, ... }
```

### Join Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     APPLICATION-SIDE JOIN FLOW                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  1. Worker receives job with employee_ids                                    │
│                                                                               │
│  2. Fetch from REDIS (cached):                                               │
│     ┌─────────────────────────────────────────────────────────────────┐     │
│     │ GET cache:workspace:{ws_id}      → workspace_config             │     │
│     │ GET cache:shifts:{ws_id}         → all_shifts                   │     │
│     │ GET cache:patterns:{ws_id}       → all_patterns                 │     │
│     │ GET cache:employees:{ws_id}      → employee_names (if needed)   │     │
│     └─────────────────────────────────────────────────────────────────┘     │
│                                                                               │
│  3. Fetch from POSTGRESQL (fresh):                                           │
│     ┌─────────────────────────────────────────────────────────────────┐     │
│     │ SELECT * FROM time_entries                                       │     │
│     │ WHERE employee_id = ANY($1) AND punch_in_time BETWEEN $2 AND $3 │     │
│     │                                                                  │     │
│     │ SELECT * FROM rosters                                            │     │
│     │ WHERE employee_id = ANY($1) AND date BETWEEN $2 AND $3          │     │
│     └─────────────────────────────────────────────────────────────────┘     │
│                                                                               │
│  4. Join in application memory:                                              │
│     ┌─────────────────────────────────────────────────────────────────┐     │
│     │ for employee in job.employee_ids:                                │     │
│     │     schedule = resolve_schedule(rosters, patterns, employee)     │     │
│     │     entries = time_entries_map[employee]                         │     │
│     │     shift = shifts_map[schedule.shift_id]                        │     │
│     │     result = calculate(entries, shift, workspace_config)         │     │
│     └─────────────────────────────────────────────────────────────────┘     │
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Related Code Files

### Files to Create
- `apiserver/src/cache/mod.rs` - Cache module
- `apiserver/src/cache/workspace_cache.rs` - Workspace data caching
- `apiserver/src/cache/shift_cache.rs` - Shift data caching

### Files to Modify
- `apiserver/src/lib.rs` - Export cache module
- `apiserver/src/worker/handlers/attendance.rs` - Use cached data

## Implementation Steps

### Step 1: Create Cache Trait

```rust
// cache/mod.rs
pub mod workspace_cache;
pub mod shift_cache;

use async_trait::async_trait;
use std::time::Duration;

pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

#[async_trait]
pub trait CacheLayer: Send + Sync {
    async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T>;
    async fn set<T: serde::Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), String>;
    async fn delete(&self, key: &str) -> Result<(), String>;
}
```

### Step 2: Implement Redis Cache Layer

```rust
// cache/redis_cache.rs
use crate::queue::RedisPool;
use redis::AsyncCommands;
use std::time::Duration;

pub struct RedisCache {
    pool: RedisPool,
}

impl RedisCache {
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = self.pool.conn();
        let json: Option<String> = conn.get(key).await.ok()?;
        json.and_then(|s| serde_json::from_str(&s).ok())
    }

    pub async fn set<T: serde::Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), String> {
        let mut conn = self.pool.conn();
        let json = serde_json::to_string(value)
            .map_err(|e| format!("Serialize error: {}", e))?;

        conn.set_ex(key, json, ttl.as_secs())
            .await
            .map_err(|e| format!("Redis set error: {}", e))
    }

    pub async fn get_or_fetch<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        fetch_fn: F,
    ) -> Result<T, String>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Clone,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        // Try cache first
        if let Some(cached) = self.get::<T>(key).await {
            return Ok(cached);
        }

        // Cache miss - fetch from source
        let value = fetch_fn().await?;

        // Store in cache (fire and forget)
        let _ = self.set(key, &value, ttl).await;

        Ok(value)
    }
}
```

### Step 3: Implement Workspace Cache

```rust
// cache/workspace_cache.rs
use crate::cache::{RedisCache, DEFAULT_CACHE_TTL};
use crate::models::EarlyArrivalStrategy;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedWorkspaceConfig {
    pub id: Uuid,
    pub timezone: String,
    pub rounding_policy: String,
    pub work_unit_minutes: i32,
    pub early_arrival_strategy: EarlyArrivalStrategy,
}

pub struct WorkspaceCache {
    cache: RedisCache,
    db: PgPool,
}

impl WorkspaceCache {
    pub fn new(cache: RedisCache, db: PgPool) -> Self {
        Self { cache, db }
    }

    fn key(workspace_id: Uuid) -> String {
        format!("cache:workspace:{}", workspace_id)
    }

    pub async fn get(&self, workspace_id: Uuid) -> Result<CachedWorkspaceConfig, String> {
        let key = Self::key(workspace_id);
        let db = self.db.clone();

        self.cache.get_or_fetch(&key, DEFAULT_CACHE_TTL, || async {
            sqlx::query_as!(
                CachedWorkspaceConfig,
                r#"
                SELECT id, timezone, rounding_policy, work_unit_minutes,
                       early_arrival_strategy as "early_arrival_strategy: EarlyArrivalStrategy"
                FROM workspaces
                WHERE id = $1
                "#,
                workspace_id
            )
            .fetch_one(&db)
            .await
            .map_err(|e| format!("DB error: {}", e))
        }).await
    }

    pub async fn invalidate(&self, workspace_id: Uuid) -> Result<(), String> {
        self.cache.delete(&Self::key(workspace_id)).await
    }
}
```

### Step 4: Implement Shift Cache

```rust
// cache/shift_cache.rs
use crate::cache::{RedisCache, DEFAULT_CACHE_TTL};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedShift {
    pub id: Uuid,
    pub name: String,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub break_duration_minutes: Option<i32>,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: Option<bool>,
    pub allow_early_ot: Option<bool>,
    pub early_punch_in_grace_period_minutes: Option<i32>,
}

pub struct ShiftCache {
    cache: RedisCache,
    db: PgPool,
}

impl ShiftCache {
    pub fn new(cache: RedisCache, db: PgPool) -> Self {
        Self { cache, db }
    }

    fn key(workspace_id: Uuid) -> String {
        format!("cache:shifts:{}", workspace_id)
    }

    /// Get all shifts for workspace as a HashMap for O(1) lookup
    pub async fn get_map(&self, workspace_id: Uuid) -> Result<HashMap<Uuid, CachedShift>, String> {
        let key = Self::key(workspace_id);
        let db = self.db.clone();

        let shifts: Vec<CachedShift> = self.cache.get_or_fetch(&key, DEFAULT_CACHE_TTL, || async {
            sqlx::query_as!(
                CachedShift,
                r#"
                SELECT id, name, start_time, end_time, break_duration_minutes,
                       break_start, break_end, is_break_auto_deduct, allow_early_ot,
                       early_punch_in_grace_period_minutes
                FROM shifts
                WHERE workspace_id = $1 AND is_active = true
                "#,
                workspace_id
            )
            .fetch_all(&db)
            .await
            .map_err(|e| format!("DB error: {}", e))
        }).await?;

        Ok(shifts.into_iter().map(|s| (s.id, s)).collect())
    }

    pub async fn invalidate(&self, workspace_id: Uuid) -> Result<(), String> {
        self.cache.delete(&Self::key(workspace_id)).await
    }
}
```

### Step 5: Create Cache Manager

```rust
// cache/manager.rs
use crate::cache::{WorkspaceCache, ShiftCache, RedisCache};
use crate::queue::RedisPool;
use sqlx::PgPool;
use std::sync::Arc;

pub struct CacheManager {
    pub workspace: WorkspaceCache,
    pub shifts: ShiftCache,
    // Future: patterns, policies, etc.
}

impl CacheManager {
    pub fn new(redis: RedisPool, db: PgPool) -> Self {
        let cache = RedisCache::new(redis);

        Self {
            workspace: WorkspaceCache::new(cache.clone(), db.clone()),
            shifts: ShiftCache::new(cache.clone(), db.clone()),
        }
    }

    /// Warm up cache for a workspace (call at job start)
    pub async fn warm_up(&self, workspace_id: Uuid) {
        // Fire-and-forget cache warming
        let _ = tokio::join!(
            self.workspace.get(workspace_id),
            self.shifts.get_map(workspace_id),
        );
    }

    /// Invalidate all caches for a workspace
    pub async fn invalidate_workspace(&self, workspace_id: Uuid) {
        let _ = tokio::join!(
            self.workspace.invalidate(workspace_id),
            self.shifts.invalidate(workspace_id),
        );
    }
}
```

### Step 6: Update Attendance Handler to Use Cache

```rust
// worker/handlers/attendance.rs (updated)
impl AttendanceHandler {
    pub async fn handle(
        pool: &PgPool,
        queue: &ReportQueue,
        cache: &CacheManager,  // NEW parameter
        job: &ReportJob,
    ) -> Result<(), String> {
        let workspace_id = job.workspace_id;

        // 1. Get cached workspace config
        let workspace = cache.workspace.get(workspace_id).await?;
        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        // 2. Get cached shifts map
        let shifts_map = cache.shifts.get_map(workspace_id).await?;

        // 3. Fetch dynamic data from DB (same as before)
        let start_dt = /* ... */;
        let end_dt = /* ... */;

        let entries_map = time_entry_repository::get_workspace_time_entries_batch(
            pool,
            &job.employee_ids,
            start_dt,
            end_dt,
        ).await.map_err(|e| format!("Failed to fetch: {}", e))?;

        // 4. Fetch rosters (dynamic, but could be cached for short periods)
        let rosters = roster_repository::get_rosters_batch(
            pool,
            &job.employee_ids,
            job.date_range.start,
            job.date_range.end,
        ).await.map_err(|e| format!("Failed to fetch rosters: {}", e))?;

        // 5. Application-side join
        for employee_id in &job.employee_ids {
            let employee_rosters = rosters.get(employee_id).unwrap_or(&vec![]);
            let employee_entries = entries_map.get(employee_id).unwrap_or(&vec![]);

            for roster in employee_rosters {
                // Get shift from cache (O(1) lookup)
                let shift = shifts_map.get(&roster.shift_id);

                // Calculate using cached shift data
                let result = Self::calculate_for_day(
                    roster,
                    shift,
                    employee_entries,
                    &workspace,
                    tz,
                )?;

                // ...
            }
        }

        // 6. Store result
        queue.store_result(job.parent_job_id, job.batch_index, &result_json).await?;

        Ok(())
    }
}
```

## Todo List

- [ ] Create `cache/mod.rs` with trait definition
- [ ] Implement `RedisCache` wrapper with get_or_fetch
- [ ] Implement `WorkspaceCache` for workspace settings
- [ ] Implement `ShiftCache` for shift data
- [ ] Create `CacheManager` to bundle all caches
- [ ] Add cache warm-up in worker job processing
- [ ] Update `AttendanceHandler` to use cached data
- [ ] Add cache invalidation endpoints (optional)
- [ ] Write unit tests for cache layer
- [ ] Write integration tests with cache hit/miss scenarios

## Success Criteria

- [ ] Cache hit rate > 95% for static data
- [ ] DB query count reduced by 40%+
- [ ] No stale data issues (5-minute TTL acceptable)
- [ ] Graceful degradation when cache unavailable

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Stale cache data | 5-minute TTL, optional invalidation webhook |
| Cache stampede | Mutex/lock during fetch (or probabilistic refresh) |
| Memory pressure on Redis | TTL ensures cleanup, monitor memory usage |
| Cache unavailable | Fallback to direct DB queries |

## Security Considerations

- Cache keys should not expose sensitive data in key names
- Consider encrypting cached data if contains PII
- Validate cached data integrity on retrieval

## Next Steps

After this phase:
1. Phase 5: Result Aggregation - Merge cached results
2. Phase 6: API Integration - Expose cache metrics
