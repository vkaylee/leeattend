# Architectural Recommendation: Move Report Results to Database

**Date**: 2026-02-03 11:17
**Type**: Architecture Proposal
**Priority**: High

---

## Problem Statement

Currently, report export results are stored in Redis with 24-hour TTL:
- High memory usage for large Excel files
- Results lost after 24 hours
- No audit trail
- Data loss risk if Redis crashes

## Proposed Solution

**Store results in PostgreSQL, use Redis only for messaging**

---

## Current Architecture

### Data Flow
```
User → API → Job created in Redis
              ↓
         Worker processes batch
              ↓
         Results stored in Redis (24hr TTL)
              ↓
         Pub/Sub notification via Redis
              ↓
         User downloads from Redis
              ↓
         After 24hr: Automatic deletion
```

### Storage Distribution
- **Redis**: Job metadata + Results + Counters + Queue
- **PostgreSQL**: User data, shifts, attendance records

### Issues
1. **Memory pressure**: Large Excel files consume Redis memory
2. **Time limitation**: Results vanish after 24 hours
3. **No history**: Can't track who exported what
4. **Risk**: Redis failure = data loss
5. **Cost**: High memory usage

---

## Proposed Architecture

### New Data Flow
```
User → API → Job created in PostgreSQL
              ↓
         Worker processes batch
              ↓
         Results stored in PostgreSQL
              ↓
         Pub/Sub notification via Redis (ephemeral)
              ↓
         User downloads from PostgreSQL
              ↓
         Optional: Cleanup after 30 days
```

### Storage Distribution
- **Redis**: Job queue + Pub/Sub + Progress counters (ephemeral)
- **PostgreSQL**: Job metadata + Results + Audit log + All business data

### Database Schema (New Table)

```sql
CREATE TABLE report_export_jobs (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id),
    requester_id UUID NOT NULL REFERENCES users(id),
    report_type VARCHAR(50) NOT NULL, -- 'summary' | 'detail'
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    employee_ids UUID[] NULL,

    -- Status tracking
    status VARCHAR(20) NOT NULL, -- 'pending' | 'processing' | 'completed' | 'failed'
    total_batches INT NOT NULL,
    completed_batches INT NOT NULL DEFAULT 0,
    failed_batches INT NOT NULL DEFAULT 0,

    -- Result storage
    file_data BYTEA NULL, -- Excel file binary
    file_name VARCHAR(255) NULL,
    file_size_bytes BIGINT NULL,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ NULL,
    completed_at TIMESTAMPTZ NULL,
    expires_at TIMESTAMPTZ NULL, -- For cleanup

    -- Error tracking
    error_message TEXT NULL,

    -- Audit
    download_count INT NOT NULL DEFAULT 0,
    last_downloaded_at TIMESTAMPTZ NULL,

    CONSTRAINT valid_status CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    CONSTRAINT valid_batches CHECK (completed_batches >= 0 AND failed_batches >= 0)
);

-- Indexes
CREATE INDEX idx_report_jobs_workspace ON report_export_jobs(workspace_id);
CREATE INDEX idx_report_jobs_status ON report_export_jobs(status);
CREATE INDEX idx_report_jobs_created ON report_export_jobs(created_at DESC);
CREATE INDEX idx_report_jobs_expires ON report_export_jobs(expires_at) WHERE expires_at IS NOT NULL;
```

---

## Benefits

### 1. Persistence
- Results stored permanently (or configurable retention)
- Users can re-download past exports
- Audit trail for compliance

### 2. Reduced Redis Memory
- Redis only stores:
  - Job queue (minimal)
  - Pub/Sub messages (ephemeral)
  - Progress counters (small integers)
- No large binary files in memory

### 3. Better Scalability
- PostgreSQL optimized for blob storage
- Can offload to S3/object storage later
- Redis handles messaging only (its strength)

### 4. Audit & Compliance
- Track all export activity
- Who exported what, when
- Download history
- Retention policies

### 5. Disaster Recovery
- Database backups include results
- No data loss if Redis crashes
- Point-in-time recovery

---

## Migration Plan

### Phase 1: Database Schema
1. Create `report_export_jobs` table
2. Add migration script
3. Deploy schema changes

### Phase 2: Worker Changes
1. Update `ReportQueue` to write results to PostgreSQL
2. Keep Redis for progress tracking
3. Pub/Sub remains unchanged

### Phase 3: API Changes
1. Update `create_export_job()` to write to PostgreSQL
2. Update `get_job_status()` to read from PostgreSQL
3. Update `download_report()` to fetch from PostgreSQL
4. Track download count

### Phase 4: Cleanup Strategy
1. Add background job to delete old exports
2. Configurable retention (default: 30 days)
3. Option to never delete (archive mode)

### Phase 5: Deprecation
1. Remove Redis result storage code
2. Keep Redis for messaging only
3. Update documentation

---

## Redis Role After Migration

**Keep for**:
- ✅ Job queue (BLPOP for workers)
- ✅ Pub/Sub (WebSocket real-time updates)
- ✅ Progress counters (atomic increment)
- ✅ Distributed locks (if needed)

**Remove**:
- ❌ Job metadata storage
- ❌ Result file storage
- ❌ 24hr TTL limitations

---

## Code Changes Required

### 1. Repository Layer (New)
```rust
// src/repositories/report_export_repository.rs
pub struct ReportExportRepository {
    pool: PgPool,
}

impl ReportExportRepository {
    pub async fn create_job(&self, job: &ReportExportJob) -> Result<()>;
    pub async fn get_job(&self, job_id: Uuid) -> Result<Option<ReportExportJob>>;
    pub async fn update_status(&self, job_id: Uuid, status: JobStatus) -> Result<()>;
    pub async fn save_result(&self, job_id: Uuid, file_data: Vec<u8>) -> Result<()>;
    pub async fn get_result(&self, job_id: Uuid) -> Result<Vec<u8>>;
    pub async fn increment_download_count(&self, job_id: Uuid) -> Result<()>;
    pub async fn cleanup_expired(&self, older_than: DateTime<Utc>) -> Result<u64>;
}
```

### 2. Queue Changes
```rust
// src/queue/report_queue.rs
impl ReportQueue {
    // Keep for queue management
    pub async fn enqueue_jobs(&self, jobs: Vec<ReportJob>) -> Result<()>;
    pub async fn dequeue_job(&self) -> Result<Option<ReportJob>>;

    // Remove (move to repository)
    // pub async fn store_result(&self, job_id: Uuid, data: Vec<u8>) -> Result<()>;
    // pub async fn get_result(&self, job_id: Uuid) -> Result<Vec<u8>>;

    // Keep for counters
    pub async fn increment_completed(&self, job_id: Uuid) -> Result<u32>;

    // Keep for Pub/Sub
    pub async fn publish_chunk(&self, job_id: Uuid, chunk: ChunkMessage) -> Result<()>;
}
```

### 3. Handler Changes
```rust
// src/handlers/report_export.rs
pub async fn download_report(
    State(state): State<AppState>,
    Path((ws_id, job_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    // Old: Fetch from Redis
    // let data = state.report_queue.get_result(job_id).await?;

    // New: Fetch from PostgreSQL
    let result = state.report_export_repo.get_result(job_id).await?;
    state.report_export_repo.increment_download_count(job_id).await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")],
        result
    ))
}
```

---

## Performance Considerations

### Database Write Performance
- **Concern**: Writing large Excel files to PostgreSQL
- **Solution**:
  - Use connection pooling
  - Compress files before storage (gzip)
  - Consider TOAST for large objects
  - Optional: Offload to S3 later

### Query Performance
- **Indexes**: workspace_id, status, created_at
- **Partitioning**: By created_at (monthly) if high volume
- **Archival**: Move old exports to archive table

### Redis Performance
- **Improvement**: Lower memory usage
- **Faster**: No large file operations
- **Stable**: Messaging-only workload

---

## Comparison

| Aspect | Current (Redis) | Proposed (PostgreSQL) |
|--------|----------------|----------------------|
| **Persistence** | 24 hours | Permanent/Configurable |
| **Memory Usage** | High (files in memory) | Low (Redis for messaging only) |
| **Audit Trail** | None | Full history |
| **Re-download** | Only within 24hr | Anytime |
| **Disaster Recovery** | Risky (memory-only) | Safe (database backups) |
| **Scalability** | Limited by Redis memory | Better (DB optimized for blobs) |
| **Cost** | High memory cost | Lower overall cost |
| **Compliance** | Limited | Full audit capability |

---

## Risks & Mitigation

### Risk 1: Database Size Growth
- **Mitigation**: Automatic cleanup after 30 days
- **Option**: Configurable retention policy
- **Alternative**: Archive to S3 after 7 days

### Risk 2: Write Performance
- **Mitigation**: Use PostgreSQL TOAST for large objects
- **Option**: Compress files (gzip) before storage
- **Alternative**: Async writes with job queue

### Risk 3: Migration Complexity
- **Mitigation**: Phased rollout, keep Redis during transition
- **Rollback**: Easy rollback to Redis if issues

---

## Timeline Estimate

- **Phase 1** (Schema): 1 day
- **Phase 2** (Worker changes): 2 days
- **Phase 3** (API changes): 2 days
- **Phase 4** (Cleanup strategy): 1 day
- **Phase 5** (Testing & deployment): 2 days

**Total**: ~8 days (1.5 weeks)

---

## Recommendation

**Strongly recommended** to proceed with migration:
1. Better architectural separation (Redis = messaging, PostgreSQL = data)
2. Improves reliability and persistence
3. Enables audit trail and compliance
4. Reduces operational costs (lower Redis memory)
5. Aligns with best practices

---

## Next Steps

1. **User approval** for architecture change
2. **Create detailed implementation plan** (phase breakdown)
3. **Write database migration script**
4. **Implement repository layer**
5. **Update workers and API handlers**
6. **Test thoroughly** (unit + integration)
7. **Deploy with gradual rollout**

---

## Unresolved Questions

1. **Retention policy**: How long to keep exports? (Suggest: 30 days default, configurable)
2. **Compression**: Should we compress files? (Suggest: Yes, use gzip)
3. **S3 offload**: Future consideration for very large exports? (Optional)
4. **Migration strategy**: Big bang or gradual? (Suggest: Gradual with feature flag)
5. **Backward compatibility**: Support old exports in Redis during transition? (Suggest: Yes, 7-day grace period)
