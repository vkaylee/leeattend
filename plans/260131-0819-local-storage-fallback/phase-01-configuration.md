# Phase 1: Configuration Setup

## Context Links
- [System Architecture](../../../docs/system-architecture.md) - Section 20.6 Environment Configuration
- [merge.rs](../../../apiserver/src/worker/handlers/merge.rs) - Current S3 upload logic

## Overview
- **Priority**: High (prerequisite for all phases)
- **Current Status**: Pending
- **Description**: Define environment variables and configuration structures for storage type detection

## Key Insights

Current behavior in `merge.rs:100-111`:
```rust
let download_url = match Self::upload_to_s3(&s3_key, excel_data).await {
    Ok(url) => url,
    Err(e) => {
        // S3 not configured - use placeholder URL (FILE DATA LOST!)
        warn!(...);
        format!("/api/reports/{}/download", parent_job_id)
    }
};
```

The placeholder URL never works because file data is discarded.

## Requirements

### Functional
- `STORAGE_TYPE` env var: `s3` | `local` (default: detect from S3_BUCKET presence)
- `LOCAL_STORAGE_PATH` env var: base directory for local storage (default: `./storage`)
- Auto-detect: If `S3_BUCKET` is set, use S3; otherwise use local

### Non-Functional
- Zero breaking changes for existing S3 deployments
- Clear logging when falling back to local storage

## Architecture

```
StorageType Detection:
1. If STORAGE_TYPE="s3" → require S3_BUCKET, fail if missing
2. If STORAGE_TYPE="local" → use local storage
3. If STORAGE_TYPE unset:
   a. If S3_BUCKET set → use S3
   b. If S3_BUCKET unset → use local (with warning)
```

## Related Code Files

### Files to Create
- None (configuration handled in existing files)

### Files to Modify
- `apiserver/src/lib.rs` - Add StorageConfig to AppState

## Implementation Steps

1. **Add configuration struct in lib.rs**
   ```rust
   /// Storage backend configuration
   #[derive(Debug, Clone)]
   pub enum StorageBackend {
       S3 { bucket: String },
       Local { base_path: PathBuf },
   }

   impl StorageBackend {
       /// Detect storage backend from environment
       pub fn from_env() -> Self {
           let storage_type = std::env::var("STORAGE_TYPE").ok();
           let s3_bucket = std::env::var("S3_BUCKET").ok();
           let local_path = std::env::var("LOCAL_STORAGE_PATH")
               .unwrap_or_else(|_| "./storage".to_string());

           match (storage_type.as_deref(), s3_bucket) {
               (Some("s3"), Some(bucket)) => Self::S3 { bucket },
               (Some("s3"), None) => panic!("STORAGE_TYPE=s3 requires S3_BUCKET"),
               (Some("local"), _) | (None, None) => {
                   tracing::info!("Using local storage at: {}", local_path);
                   Self::Local { base_path: PathBuf::from(local_path) }
               }
               (None, Some(bucket)) => Self::S3 { bucket },
               _ => Self::Local { base_path: PathBuf::from(local_path) },
           }
       }

       pub fn is_local(&self) -> bool {
           matches!(self, Self::Local { .. })
       }
   }
   ```

2. **Add to AppState**
   ```rust
   pub struct AppState {
       pub pool: PgPool,
       pub coordinator: Option<Arc<ReportCoordinator>>,
       pub storage: StorageBackend,  // ADD THIS
       // ... existing fields
   }
   ```

3. **Initialize in main.rs**
   ```rust
   let storage = StorageBackend::from_env();

   let state = Arc::new(AppState {
       pool: pool.clone(),
       coordinator,
       storage,  // ADD THIS
       // ...
   });
   ```

## Todo List

- [ ] Add `StorageBackend` enum to `lib.rs`
- [ ] Implement `from_env()` detection logic
- [ ] Add `storage` field to `AppState`
- [ ] Initialize storage in `main.rs`
- [ ] Add debug logging for storage type selection

## Success Criteria

- [ ] `StorageBackend::from_env()` correctly detects S3 when `S3_BUCKET` set
- [ ] Falls back to local when `S3_BUCKET` unset
- [ ] Explicit `STORAGE_TYPE=local` always uses local
- [ ] `STORAGE_TYPE=s3` without `S3_BUCKET` panics with clear message

## Security Considerations

- `LOCAL_STORAGE_PATH` should be validated to prevent arbitrary filesystem access
- Default to relative path (`./storage`) to stay within project directory

## Next Steps

After completing this phase, proceed to [Phase 2: Storage Abstraction Layer](./phase-02-storage-abstraction.md)
