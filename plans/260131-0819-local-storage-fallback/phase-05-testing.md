# Phase 5: Testing

## Context Links
- [Code Standards - Testing](../../../docs/code-standards.md#testing-standards)
- [WSL Performance Protocol](../../../CLAUDE.md) - MUST use `./run-tests.sh`

## Overview
- **Priority**: High (validation)
- **Current Status**: Pending
- **Description**: Verify local storage works end-to-end and existing S3 behavior unchanged

## Key Insights

Testing constraints:
1. **WSL Performance**: MUST use `./run-tests.sh`, never direct `cargo test`
2. **No S3 in CI**: Tests should work without AWS credentials
3. **Local storage is default**: When S3_BUCKET unset, local storage activates

## Requirements

### Functional Tests
- Storage detection logic
- Local storage upload/download
- Path traversal prevention
- Error handling

### Non-Functional
- No external dependencies (S3) required for local tests
- Tests clean up temporary files

## Test Strategy

```
┌─────────────────────────────────────────────────────────┐
│                    Test Categories                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  1. Unit Tests (storage.rs)                              │
│     - StorageBackend::from_env() detection               │
│     - LocalStorage path handling                         │
│     - Path traversal prevention                          │
│                                                          │
│  2. Integration Tests (new file)                         │
│     - Full upload → download cycle for local storage     │
│     - Concurrent access handling                         │
│     - File cleanup verification                          │
│                                                          │
│  3. Manual Verification                                  │
│     - Start server without S3_BUCKET                     │
│     - Create export job                                  │
│     - Download completed report                          │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Related Code Files

### Files to Create
- `apiserver/src/services/storage.rs` - Unit tests at bottom
- `apiserver/tests/local_storage_tests.rs` - Integration tests (optional)

### Files to Modify
- None (tests are additive)

## Implementation Steps

1. **Add unit tests to storage.rs**

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use tempfile::TempDir;

       #[test]
       fn test_storage_backend_from_env_defaults_to_local() {
           // Clear env vars
           std::env::remove_var("STORAGE_TYPE");
           std::env::remove_var("S3_BUCKET");
           std::env::set_var("LOCAL_STORAGE_PATH", "/tmp/test-storage");

           let backend = StorageBackend::from_env();
           assert!(matches!(backend, StorageBackend::Local { .. }));

           // Cleanup
           std::env::remove_var("LOCAL_STORAGE_PATH");
       }

       #[test]
       fn test_storage_backend_s3_when_bucket_set() {
           std::env::remove_var("STORAGE_TYPE");
           std::env::set_var("S3_BUCKET", "test-bucket");

           let backend = StorageBackend::from_env();
           assert!(matches!(backend, StorageBackend::S3 { bucket } if bucket == "test-bucket"));

           // Cleanup
           std::env::remove_var("S3_BUCKET");
       }

       #[test]
       #[should_panic(expected = "STORAGE_TYPE=s3 requires S3_BUCKET")]
       fn test_storage_backend_s3_requires_bucket() {
           std::env::set_var("STORAGE_TYPE", "s3");
           std::env::remove_var("S3_BUCKET");

           let _ = StorageBackend::from_env();

           // Cleanup (won't reach due to panic)
           std::env::remove_var("STORAGE_TYPE");
       }

       #[tokio::test]
       async fn test_local_storage_upload_creates_file() {
           let temp_dir = TempDir::new().unwrap();
           let storage = LocalStorage::new(
               temp_dir.path().to_path_buf(),
               "/api/reports/local".to_string(),
           );

           let key = "test-workspace/2026/01/report.xlsx";
           let data = b"test content".to_vec();

           let result = storage.upload(key, data.clone(), "application/xlsx").await;
           assert!(result.is_ok());

           // Verify file exists
           let file_path = temp_dir.path().join(key);
           assert!(file_path.exists());

           // Verify content
           let content = tokio::fs::read(&file_path).await.unwrap();
           assert_eq!(content, data);
       }

       #[tokio::test]
       async fn test_local_storage_prevents_path_traversal() {
           let temp_dir = TempDir::new().unwrap();
           let storage = LocalStorage::new(
               temp_dir.path().to_path_buf(),
               "/api/reports/local".to_string(),
           );

           let malicious_key = "../../../etc/passwd";
           let data = b"malicious".to_vec();

           let result = storage.upload(malicious_key, data, "text/plain").await;
           assert!(result.is_err());
           assert!(result.unwrap_err().0.contains("traversal"));
       }

       #[tokio::test]
       async fn test_local_storage_download_url_format() {
           let temp_dir = TempDir::new().unwrap();
           let storage = LocalStorage::new(
               temp_dir.path().to_path_buf(),
               "/api/reports/local".to_string(),
           );

           let key = "ws-123/2026/01/report.xlsx";
           let url = storage.get_download_url(key).await.unwrap();

           assert!(url.starts_with("/api/reports/local/"));
           assert!(url.contains("ws-123"));
       }

       #[test]
       fn test_storage_error_display() {
           let err = StorageError("test error".to_string());
           assert_eq!(format!("{}", err), "test error");
       }
   }
   ```

2. **Manual verification steps**

   ```bash
   # 1. Build and start server without S3
   cd apiserver
   unset S3_BUCKET
   unset AWS_ACCESS_KEY_ID
   unset AWS_SECRET_ACCESS_KEY
   export LOCAL_STORAGE_PATH="./test-storage"
   cargo run

   # 2. In another terminal, check logs for:
   # "Initializing local storage provider"

   # 3. Create an export job via API
   curl -X POST http://localhost:3000/api/workspaces/{ws_id}/reports/export \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer {token}" \
     -d '{"report_type":"summary","start_date":"2026-01-01","end_date":"2026-01-31"}'

   # 4. Poll status until completed
   curl http://localhost:3000/api/workspaces/{ws_id}/reports/jobs/{job_id}

   # 5. Download report
   curl -L http://localhost:3000/api/workspaces/{ws_id}/reports/jobs/{job_id}/download \
     -o report.xlsx

   # 6. Verify file
   file report.xlsx  # Should show: Microsoft Excel 2007+

   # 7. Check storage directory
   ls -la ./test-storage/reports/
   ```

3. **Run test suite**

   ```bash
   # IMPORTANT: Use run-tests.sh on WSL
   ./run-tests.sh --api storage

   # Or for full API test suite
   ./run-tests.sh --api
   ```

## Todo List

- [ ] Add unit tests to `storage.rs`
- [ ] Add path traversal test
- [ ] Add URL format test
- [ ] Manual test: server startup with local storage
- [ ] Manual test: full export → download cycle
- [ ] Verify file content matches expected Excel format
- [ ] Run `./run-tests.sh --api` to ensure no regressions

## Success Criteria

- [ ] All unit tests pass
- [ ] Path traversal attempts are blocked
- [ ] Manual export/download works
- [ ] Downloaded Excel file opens correctly
- [ ] No regressions in existing tests

## Testing Notes

**Environment isolation**: Unit tests should use `tempfile::TempDir` to avoid polluting filesystem.

**Concurrency**: If needed, add test for concurrent uploads to same directory.

**Cleanup**: Temp directories auto-cleanup on drop. Production cleanup relies on external job (e.g., cron) or Redis TTL-triggered cleanup.

## Unresolved Questions

1. **File cleanup**: Should we add a background task to clean up expired local files, or rely on manual/cron cleanup?
   - Recommendation: Start simple with manual cleanup, add background task if needed

2. **Disk space monitoring**: Should we add health check for storage space?
   - Recommendation: Out of scope for this feature; handle at infra level

## Security Test Cases

| Test | Input | Expected Result |
|------|-------|-----------------|
| Normal key | `ws-123/2026/01/report.xlsx` | Success |
| Path traversal | `../../../etc/passwd` | Error: traversal detected |
| Encoded traversal | `..%2F..%2Fetc%2Fpasswd` | Error: traversal detected |
| Null byte | `report\x00.xlsx` | Error or safe handling |
| Very long key | 1000+ char path | Error: path too long |

## Next Steps

After all phases complete:
1. Code review
2. Merge to main
3. Update deployment docs if needed
