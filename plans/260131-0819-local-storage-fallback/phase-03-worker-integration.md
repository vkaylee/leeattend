# Phase 3: Worker Handler Integration

## Context Links
- [merge.rs](../../../apiserver/src/worker/handlers/merge.rs) - Report merge and upload
- [Phase 2](./phase-02-storage-abstraction.md) - Storage abstraction

## Overview
- **Priority**: High (core implementation)
- **Current Status**: Pending
- **Description**: Replace direct S3 calls with storage provider abstraction

## Key Insights

Current upload flow in `merge.rs:91-111`:
```rust
// 6. Upload to S3
let s3_key = format!("{}/{}/{}/{}", ws_id, year, month, file_name);

let download_url = match Self::upload_to_s3(&s3_key, excel_data).await {
    Ok(url) => url,
    Err(e) => {
        warn!("S3 upload failed, using placeholder URL");  // DATA LOST!
        format!("/api/reports/{}/download", parent_job_id)
    }
};
```

Problem: On S3 failure, `excel_data` bytes are discarded completely.

## Requirements

### Functional
- Pass storage provider to `MergeHandler::handle()`
- Use provider.upload() instead of direct S3 call
- Generate download URL using provider.get_download_url()
- Store both key and URL in Redis for download handler

### Non-Functional
- Maintain existing error logging
- No changes to job queue structure
- Backward compatible with existing Redis data

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                MergeHandler::handle()                    │
│                                                          │
│  1. Merge batch results                                  │
│  2. Generate Excel                                       │
│  3. storage.upload(key, data, content_type)  ← CHANGE   │
│  4. storage.get_download_url(key)            ← CHANGE   │
│  5. queue.set_result_url(job_id, url)                   │
│  6. Update job status → Completed                        │
└─────────────────────────────────────────────────────────┘
```

## Related Code Files

### Files to Modify
- `apiserver/src/worker/handlers/merge.rs` - Main changes
- `apiserver/src/worker/mod.rs` - Pass storage to handler (if needed)
- `apiserver/src/queue/job.rs` - Add storage_key field to JobMetadata (optional)

## Implementation Steps

1. **Update MergeHandler::handle signature**
   ```rust
   use crate::services::DynStorageProvider;

   impl MergeHandler {
       pub async fn handle(
           pool: &PgPool,
           queue: &ReportQueue,
           job: &ReportJob,
           storage: &DynStorageProvider,  // ADD PARAMETER
       ) -> Result<(), String> {
           // ... existing code
       }
   }
   ```

2. **Replace upload_to_s3 with storage provider**

   **Before (lines 91-111):**
   ```rust
   let s3_key = format!(...);
   let download_url = match Self::upload_to_s3(&s3_key, excel_data).await {
       Ok(url) => url,
       Err(e) => {
           warn!(...);
           format!("/api/reports/{}/download", parent_job_id)  // BROKEN
       }
   };
   ```

   **After:**
   ```rust
   // 6. Build storage key
   let storage_key = format!(
       "{}/{}/{}/{}",
       meta.workspace_id,
       Utc::now().format("%Y"),
       Utc::now().format("%m"),
       file_name
   );

   // 7. Upload to storage (S3 or local)
   let upload_result = storage
       .upload(&storage_key, excel_data, EXCEL_CONTENT_TYPE)
       .await;

   let download_url = match upload_result {
       Ok(key) => {
           // Get download URL from storage provider
           storage.get_download_url(&key).await.map_err(|e| {
               format!("Failed to generate download URL: {}", e)
           })?
       }
       Err(e) => {
           // Critical: Storage failure should fail the job, not silently discard data
           return Err(format!("Storage upload failed: {}. Report data lost!", e));
       }
   };

   info!(
       parent_job_id = %parent_job_id,
       storage_type = storage.storage_type(),
       key = %storage_key,
       "Report uploaded successfully"
   );
   ```

3. **Remove upload_to_s3 helper method**

   Delete lines 218-233:
   ```rust
   // REMOVE THIS METHOD
   async fn upload_to_s3(key: &str, data: Vec<u8>) -> Result<String, String> { ... }
   ```

4. **Update worker runner to pass storage**

   Find where MergeHandler::handle is called and add storage parameter:
   ```rust
   // In worker/mod.rs or wherever handle() is called:
   MergeHandler::handle(&pool, &queue, &job, &state.storage).await?
   ```

5. **Optional: Store storage_key in JobMetadata for debugging**

   In `queue/job.rs`, add field:
   ```rust
   pub struct JobMetadata {
       // ... existing fields
       /// Storage key for the generated report (for debugging)
       #[serde(default, skip_serializing_if = "Option::is_none")]
       pub storage_key: Option<String>,
   }
   ```

## Todo List

- [ ] Add `storage: &DynStorageProvider` parameter to `MergeHandler::handle()`
- [ ] Replace `upload_to_s3()` call with `storage.upload()`
- [ ] Replace presigned URL with `storage.get_download_url()`
- [ ] Remove `upload_to_s3()` helper method
- [ ] Update all call sites of `MergeHandler::handle()`
- [ ] Add storage type to log messages
- [ ] Test error handling path

## Success Criteria

- [ ] S3 uploads continue to work when S3_BUCKET is set
- [ ] Local uploads work when S3_BUCKET is unset
- [ ] Storage failures fail the job instead of silently losing data
- [ ] Download URLs are correct for both storage types
- [ ] Existing tests pass

## Error Handling Changes

**Before**: Storage failure = data loss + broken URL
**After**: Storage failure = job fails with error message

This is intentional - losing report data silently is worse than failing the job.

## Security Considerations

- Storage key uses workspace_id prefix for isolation
- No user-controlled input in storage key (only UUIDs and timestamps)

## Next Steps

Proceed to [Phase 4: Download Handler Updates](./phase-04-download-handler.md)
