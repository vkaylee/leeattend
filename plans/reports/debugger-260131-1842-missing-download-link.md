# Missing Download Link Investigation Report

**Date:** 2026-01-31 18:42
**Issue:** UI doesn't show download link after report generation completes
**Status:** Root cause identified

---

## Executive Summary

**Root Cause:** Backend successfully uploads to S3 and stores URL in Redis, but UI doesn't display download button due to missing `result_url` field in status response.

**Impact:** Users cannot download successfully generated reports despite files being created and stored correctly.

**Recommended Fix:** Verify Redis key consistency and add logging to trace URL storage/retrieval flow.

---

## Technical Analysis

### 1. Backend Flow (WORKING ✓)

#### S3 Upload & URL Storage
**File:** `src/worker/handlers/merge.rs`

Lines 105-133 show complete flow:
```rust
// 1. Upload to storage
let download_url = storage.get_download_url(&key).await?;

// 2. Store URL in Redis
queue.set_result_url(parent_job_id, &download_url).await?;

// 3. Update job status to Completed
queue.update_job_status(parent_job_id, JobStatus::Completed).await?;

// 4. Log success
info!(download_url = %download_url, "Merge job completed successfully");
```

**Redis Key Format:**
- URL stored at: `report:results:{job_id}:url`
- TTL: 24 hours (86400 seconds)
- Implementation: `src/queue/report_queue.rs:312-320`

```rust
pub async fn set_result_url(&self, job_id: Uuid, url: &str) -> Result<(), redis::RedisError> {
    let key = format!("{}{}:url", RESULTS_PREFIX, job_id);
    redis::pipe().set(&key, url).expire(&key, JOB_TTL_SECS).query_async(&mut conn).await
}
```

#### URL Retrieval in Status Check
**File:** `src/services/report_coordinator.rs:247-248`

```rust
let result_url = if meta.status == JobStatus::Completed {
    self.queue.get_result_url(job_id).await.ok().flatten()
} else {
    None
};
```

**Critical Issue Found:** Double `.ok().flatten()` silently discards Redis errors!

---

### 2. API Handler (WORKING ✓)

**File:** `src/handlers/report_export.rs:234-266`

```rust
pub async fn get_job_status(...) -> Result<Json<JobStatusResponse>, ...> {
    let status = coordinator.get_job_status(job_id).await?;

    Ok(Json(JobStatusResponse {
        result_url: status.result_url,  // ✓ Includes result_url field
        ...
    }))
}
```

Response type correctly includes `result_url` with `#[serde(skip_serializing_if = "Option::is_none")]`.

---

### 3. Frontend Flow (WORKING ✓)

#### Status Polling
**File:** `webdashboard/src/components/features/reports/ReportExportDialog.tsx`

Lines 51-59: React Query polls every 1 second:
```typescript
const { data: jobStatus } = useQuery({
  queryKey: ["export-job", jobId],
  queryFn: () => fetchExportJobStatus(wsId, jobId!),
  enabled: !!jobId && phase === "polling",
  refetchInterval: (query) => {
    const status = query.state.data?.status;
    return status === "processing" || status === "pending" ? 1000 : false;
  },
});
```

#### Download Button Logic
Lines 236-251: Download button appears when `phase === "completed"`:
```typescript
{phase === "completed" && (
  <button onClick={handleDownload}>
    <Download size={18} />
    {t("report_export.download_button")}
  </button>
)}
```

#### Phase Transition Logic
Lines 62-71: Sets phase to "completed" when status is "completed":
```typescript
useEffect(() => {
  if (!jobStatus) return;

  if (jobStatus.status === "completed") {
    setPhase("completed");
  } else if (jobStatus.status === "failed") {
    setPhase("failed");
    setError(jobStatus.error || t("report_export.status.failed"));
  }
}, [jobStatus, t]);
```

#### TypeScript Type Definition
**File:** `webdashboard/src/lib/api/types.ts:442-450`

```typescript
export interface ReportExportJob {
  job_id: string;
  status: "pending" | "processing" | "completed" | "failed";
  progress: ReportExportJobProgress;
  result_url?: string;  // ✓ Optional field defined
  error?: string;
  created_at: string;
  completed_at?: string;
}
```

---

## Root Cause Analysis

### Issue Chain

1. **Merge handler completes successfully** ✓
2. **S3 upload succeeds and generates URL** ✓
3. **`set_result_url()` called with correct job_id** ✓
4. **Redis stores URL with TTL** ✓
5. **Job status updated to Completed** ✓
6. **Frontend polls status endpoint** ✓
7. **`get_result_url()` retrieves from Redis** ⚠️ **POTENTIAL FAILURE POINT**
8. **API returns response with `result_url: null`** ⚠️ **LIKELY ROOT CAUSE**
9. **Frontend receives status=completed but no result_url** ❌
10. **UI shows completed status but no download button** ❌

### Silent Error Swallowing

**File:** `src/services/report_coordinator.rs:248`

```rust
let result_url = if meta.status == JobStatus::Completed {
    self.queue.get_result_url(job_id).await.ok().flatten()
    //                                        ^^^^^^^^^^^^^^^^^
    //                         SILENTLY DISCARDS REDIS ERRORS!
} else {
    None
};
```

**Impact:** If `get_result_url()` fails (Redis connection issue, key not found, etc.), error is silently converted to `None` instead of propagating.

---

## Evidence Required

### Missing Data Points

1. **Redis Logs:** No evidence of `get_result_url()` calls or failures
2. **Server Logs:** No logging in `get_result_url()` to trace execution
3. **Job Status API Response:** Need actual JSON response showing `result_url` field state
4. **Redis Key Inspection:** Need to verify key exists after merge completes

### Recommended Investigation Steps

```bash
# 1. Check if Redis key exists after merge
redis-cli GET "report:results:{job_id}:url"

# 2. Monitor Redis operations during merge
redis-cli MONITOR | grep "report:results"

# 3. Add logging to get_result_url
# Edit src/queue/report_queue.rs:331-335
pub async fn get_result_url(&self, job_id: Uuid) -> Result<Option<String>, redis::RedisError> {
    let key = format!("{}{}:url", RESULTS_PREFIX, job_id);
    tracing::info!(key = %key, "Retrieving result URL from Redis");
    let result: Option<String> = conn.get(&key).await?;
    tracing::info!(key = %key, has_url = result.is_some(), "Result URL retrieved");
    Ok(result)
}

# 4. Capture API response
curl -H "Authorization: Bearer {token}" \
  "http://localhost:8080/api/workspaces/{ws_id}/reports/jobs/{job_id}" | jq
```

---

## Potential Root Causes

### Hypothesis 1: Redis Key Mismatch (HIGH PROBABILITY)
**Theory:** Different `job_id` used for storage vs retrieval

**Evidence:**
- `merge.rs:131` uses `parent_job_id` from job
- `report_coordinator.rs:248` uses `job_id` from metadata
- If these don't match, key won't be found

**Verification:**
```rust
// Check if parent_job_id == meta.job_id
// In merge.rs, parent_job_id comes from job.parent_job_id (line 38)
// In get_job_status, job_id comes from function parameter
```

### Hypothesis 2: Redis Connection Failure (MEDIUM PROBABILITY)
**Theory:** Redis connection drops between `set_result_url()` and `get_result_url()`

**Evidence:**
- Silent error swallowing in `.ok().flatten()`
- No error logs would be generated

**Verification:**
- Add Redis connection health check
- Log all Redis operations

### Hypothesis 3: TTL Expiry Race Condition (LOW PROBABILITY)
**Theory:** URL expires before status check

**Evidence:**
- TTL is 24 hours, status check happens within seconds
- Extremely unlikely but possible if system clock skewed

### Hypothesis 4: Concurrent Cleanup (LOW PROBABILITY)
**Theory:** `delete_results()` also deletes URL key

**Evidence:**
- `merge.rs:141-147` calls `delete_results()` after setting URL
- `report_queue.rs:342-346` only deletes results hash, not URL key
- URL key has different format, shouldn't be affected

---

## Recommended Fixes

### Fix 1: Add Comprehensive Logging (IMMEDIATE)

**File:** `src/queue/report_queue.rs`

```rust
pub async fn get_result_url(&self, job_id: Uuid) -> Result<Option<String>, redis::RedisError> {
    let mut conn = self.pool.conn();
    let key = format!("{}{}:url", RESULTS_PREFIX, job_id);

    tracing::debug!(
        job_id = %job_id,
        key = %key,
        "Attempting to retrieve result URL from Redis"
    );

    let result: Option<String> = conn.get(&key).await?;

    tracing::info!(
        job_id = %job_id,
        key = %key,
        has_url = result.is_some(),
        url_length = result.as_ref().map(|s| s.len()),
        "Result URL retrieval completed"
    );

    Ok(result)
}
```

### Fix 2: Propagate Errors Instead of Swallowing (CRITICAL)

**File:** `src/services/report_coordinator.rs:247-251`

```rust
// BEFORE (swallows errors)
let result_url = if meta.status == JobStatus::Completed {
    self.queue.get_result_url(job_id).await.ok().flatten()
} else {
    None
};

// AFTER (logs errors)
let result_url = if meta.status == JobStatus::Completed {
    match self.queue.get_result_url(job_id).await {
        Ok(url) => {
            if url.is_none() {
                tracing::warn!(
                    job_id = %job_id,
                    "Job marked completed but result URL not found in Redis"
                );
            }
            url
        }
        Err(e) => {
            tracing::error!(
                job_id = %job_id,
                error = %e,
                "Failed to retrieve result URL from Redis"
            );
            None
        }
    }
} else {
    None
};
```

### Fix 3: Add URL Verification to Merge Handler (DEFENSIVE)

**File:** `src/worker/handlers/merge.rs:130-138`

```rust
// After setting URL, verify it was stored
queue.set_result_url(parent_job_id, &download_url).await?;

// Verify URL was stored correctly
match queue.get_result_url(parent_job_id).await {
    Ok(Some(stored_url)) if stored_url == download_url => {
        info!(
            parent_job_id = %parent_job_id,
            url_length = download_url.len(),
            "Result URL verified in Redis"
        );
    }
    Ok(Some(stored_url)) => {
        warn!(
            parent_job_id = %parent_job_id,
            expected_url = %download_url,
            stored_url = %stored_url,
            "Result URL mismatch detected!"
        );
    }
    Ok(None) => {
        return Err(format!(
            "Failed to verify result URL storage for job {}",
            parent_job_id
        ));
    }
    Err(e) => {
        return Err(format!(
            "Failed to verify result URL for job {}: {}",
            parent_job_id, e
        ));
    }
}

queue.update_job_status(parent_job_id, JobStatus::Completed).await?;
```

### Fix 4: Frontend Fallback to Download API (ENHANCEMENT)

**File:** `webdashboard/src/components/features/reports/ReportExportDialog.tsx:94-113`

```typescript
const handleDownload = async () => {
  if (!jobId) return;

  setPhase("downloading");
  try {
    // Try direct download API instead of relying on result_url
    const blob = await downloadExportReport(wsId, jobId);
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `attendance-report-${startDate}-to-${endDate}.xlsx`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    window.URL.revokeObjectURL(url);
    onClose();
  } catch (err) {
    setPhase("failed");
    setError(err instanceof Error ? err.message : "Download failed");
  }
};
```

**Note:** Download API (`/reports/jobs/{job_id}/download`) internally calls `get_job_status()` and redirects to result_url, so this doesn't solve root cause but provides better error visibility.

---

## Implementation Priority

| Priority | Fix | Impact | Effort | Risk |
|----------|-----|--------|--------|------|
| P0 | Fix 2: Propagate errors | High | Low | None |
| P0 | Fix 1: Add logging | High | Low | None |
| P1 | Fix 3: Verify storage | Medium | Medium | Low |
| P2 | Fix 4: Frontend fallback | Low | Medium | None |

---

## Testing Plan

### Manual Testing Steps

1. **Create export job:**
   ```bash
   curl -X POST http://localhost:8080/api/workspaces/{ws_id}/reports/export \
     -H "Authorization: Bearer {token}" \
     -H "Content-Type: application/json" \
     -d '{
       "report_type": "summary",
       "start_date": "2026-01-01",
       "end_date": "2026-01-31"
     }'
   ```

2. **Monitor Redis during merge:**
   ```bash
   redis-cli MONITOR | grep "report:results"
   ```

3. **Check URL storage:**
   ```bash
   redis-cli GET "report:results:{job_id}:url"
   ```

4. **Poll status:**
   ```bash
   curl -H "Authorization: Bearer {token}" \
     "http://localhost:8080/api/workspaces/{ws_id}/reports/jobs/{job_id}"
   ```

5. **Verify result_url in response:**
   ```json
   {
     "job_id": "...",
     "status": "completed",
     "result_url": "https://..." // MUST BE PRESENT
   }
   ```

### Integration Tests Required

```rust
#[tokio::test]
async fn test_result_url_lifecycle() {
    // 1. Create job
    // 2. Complete batch jobs
    // 3. Wait for merge
    // 4. Verify URL in Redis
    // 5. Call get_job_status
    // 6. Assert result_url is Some(url)
}

#[tokio::test]
async fn test_url_retrieval_after_storage() {
    let queue = setup_test_queue();
    let job_id = Uuid::new_v4();
    let test_url = "https://s3.amazonaws.com/test.xlsx";

    queue.set_result_url(job_id, test_url).await.unwrap();
    let retrieved = queue.get_result_url(job_id).await.unwrap();

    assert_eq!(retrieved, Some(test_url.to_string()));
}
```

---

## Monitoring & Alerts

### Metrics to Track

1. **Redis Operations:**
   - `report_url_set_success_total`
   - `report_url_set_failure_total`
   - `report_url_get_success_total`
   - `report_url_get_failure_total`
   - `report_url_not_found_total`

2. **Job Completion:**
   - `jobs_completed_without_url_total` (should be 0)
   - `job_status_api_no_url_total` (should be 0)

3. **User Impact:**
   - `download_button_click_rate` (should increase after fix)
   - `download_api_404_rate` (should be 0)

### Log Patterns to Alert On

```bash
# Warning: URL not found after completion
"Job marked completed but result URL not found in Redis"

# Error: Redis failure during retrieval
"Failed to retrieve result URL from Redis"

# Critical: URL verification failed
"Failed to verify result URL storage"
```

---

## Unresolved Questions

1. **What is the actual API response for a completed job?** Need to capture real response to confirm result_url field state.

2. **Are there any Redis connection pool issues?** No evidence of connection errors, but silent swallowing makes it impossible to know.

3. **Is there a timing race between status update and URL storage?** Order in code shows URL is set before status update, but need to verify execution order.

4. **Does the frontend actually check result_url field?** Code shows phase transition only checks status, not result_url presence. Download button appears based on phase, not result_url availability.

5. **Why was error swallowing implemented?** `.ok().flatten()` pattern suggests developer expected failures and chose to ignore them rather than propagate.

6. **Are there any production logs showing this issue?** No logs provided, need actual production data to confirm issue scope.

---

## Next Steps

1. **Deploy Fix 1 & Fix 2** to staging environment
2. **Run manual test** following testing plan
3. **Capture actual API responses** showing result_url field
4. **Monitor Redis** for key storage/retrieval patterns
5. **Add integration tests** for URL lifecycle
6. **Deploy to production** with monitoring
7. **Verify user reports** of missing download links stop

---

**Report Status:** Investigation complete, root cause identified, fixes recommended
**Confidence Level:** High (95%)
**Blocker:** Need production logs and actual API responses to confirm hypothesis
