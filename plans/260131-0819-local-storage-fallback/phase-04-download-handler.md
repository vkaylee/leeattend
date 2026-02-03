# Phase 4: Download Handler Updates

## Context Links
- [report_export.rs](../../../apiserver/src/handlers/report_export.rs) - Current download handler
- [Phase 2](./phase-02-storage-abstraction.md) - LocalStorage implementation

## Overview
- **Priority**: High (completes the flow)
- **Current Status**: Pending
- **Description**: Add endpoint to serve local files and update download routing

## Key Insights

Current download flow in `report_export.rs:269-313`:
```rust
pub async fn download_report(...) -> Result<impl IntoResponse, ...> {
    let status = coordinator.get_job_status(job_id).await?;

    match status.status.as_str() {
        "completed" => {
            if let Some(url) = status.result_url {
                Ok(Redirect::temporary(&url).into_response())  // Redirect to S3
            } else {
                Err((500, "Result URL not available"))
            }
        }
        // ...
    }
}
```

Current behavior:
- S3: Redirect to presigned URL
- Local: URL is `/api/reports/local/{key}` but no handler exists!

## Requirements

### Functional
- Add `GET /api/reports/local/{key}` endpoint for local file serving
- Serve file with correct Content-Type and Content-Disposition headers
- Validate key is a valid path (no traversal)
- Return 404 if file not found

### Non-Functional
- Stream file (don't load entire file into memory)
- Set appropriate cache headers (no-cache for reports)
- Security: prevent directory traversal attacks

## Architecture

```
Download Flow:
┌─────────────────────────────────────────────────────────┐
│  GET /api/workspaces/:ws_id/reports/jobs/:job_id/download│
│                                                          │
│  1. Get job status from Redis                            │
│  2. Get result_url                                       │
│  3. If S3 URL → Redirect                                 │
│  4. If Local URL → Serve file directly                   │
└─────────────────────────────────────────────────────────┘

Local File Serving:
┌─────────────────────────────────────────────────────────┐
│  GET /api/reports/local/{encoded_key}                    │
│                                                          │
│  1. URL-decode key                                       │
│  2. Validate path (prevent traversal)                    │
│  3. Open file from LOCAL_STORAGE_PATH/reports/{key}      │
│  4. Stream response with Content-Disposition             │
└─────────────────────────────────────────────────────────┘
```

## Related Code Files

### Files to Modify
- `apiserver/src/handlers/report_export.rs` - Add local file handler
- `apiserver/src/lib.rs` - Add route for local file endpoint

## Implementation Steps

1. **Add local file serving endpoint in report_export.rs**
   ```rust
   use axum::{
       body::Body,
       extract::Path,
       http::{header, StatusCode},
       response::IntoResponse,
   };
   use tokio::fs::File;
   use tokio_util::io::ReaderStream;

   /// GET /api/reports/local/{key}
   ///
   /// Serve locally stored report files.
   /// Only used when storage is configured for local filesystem.
   pub async fn serve_local_report(
       State(state): State<Arc<AppState>>,
       Path(encoded_key): Path<String>,
   ) -> Result<impl IntoResponse, (StatusCode, String)> {
       // 1. Decode the URL-encoded key
       let key = urlencoding::decode(&encoded_key)
           .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid key encoding".to_string()))?;

       // 2. Get local storage base path
       let base_path = match &state.storage {
           crate::StorageBackend::Local { base_path } => base_path.join("reports"),
           _ => {
               return Err((
                   StatusCode::NOT_FOUND,
                   "Local storage not configured".to_string(),
               ))
           }
       };

       // 3. Build and validate file path
       let file_path = base_path.join(key.as_ref());

       // Security: Prevent path traversal
       let canonical_base = base_path.canonicalize().unwrap_or(base_path.clone());
       let canonical_file = match file_path.canonicalize() {
           Ok(p) => p,
           Err(_) => {
               return Err((StatusCode::NOT_FOUND, "Report not found".to_string()))
           }
       };

       if !canonical_file.starts_with(&canonical_base) {
           tracing::warn!(
               requested = %key,
               "Path traversal attempt blocked"
           );
           return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
       }

       // 4. Open file
       let file = File::open(&canonical_file).await.map_err(|e| {
           if e.kind() == std::io::ErrorKind::NotFound {
               (StatusCode::NOT_FOUND, "Report not found".to_string())
           } else {
               tracing::error!(error = %e, path = %canonical_file.display(), "Failed to open report file");
               (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read report".to_string())
           }
       })?;

       // 5. Get file metadata for Content-Length
       let metadata = file.metadata().await.map_err(|e| {
           tracing::error!(error = %e, "Failed to get file metadata");
           (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read report".to_string())
       })?;

       // 6. Extract filename for Content-Disposition
       let filename = canonical_file
           .file_name()
           .and_then(|n| n.to_str())
           .unwrap_or("report.xlsx");

       // 7. Stream the file
       let stream = ReaderStream::new(file);
       let body = Body::from_stream(stream);

       // 8. Build response with headers
       let headers = [
           (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
           (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", filename)),
           (header::CONTENT_LENGTH, &metadata.len().to_string()),
           (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
       ];

       Ok((headers, body))
   }
   ```

2. **Add route in lib.rs**

   Find the router setup and add:
   ```rust
   // In the router configuration
   .route("/api/reports/local/*key", get(handlers::report_export::serve_local_report))
   ```

   Or if using nested routers:
   ```rust
   let report_routes = Router::new()
       // ... existing routes
       .route("/local/*key", get(serve_local_report));
   ```

3. **Update download_report to handle local URLs inline (optional optimization)**

   For local storage, we can serve files directly without redirect:
   ```rust
   pub async fn download_report(...) -> Result<impl IntoResponse, ...> {
       // ... existing status check ...

       match status.status.as_str() {
           "completed" => {
               if let Some(url) = status.result_url {
                   // Check if it's a local URL
                   if url.starts_with("/api/reports/local/") {
                       // Could serve directly here, but redirect is cleaner
                       // for consistent behavior and URL bookmarking
                   }
                   Ok(Redirect::temporary(&url).into_response())
               } else {
                   Err((500, "Result URL not available"))
               }
           }
           // ...
       }
   }
   ```

   Decision: Keep redirect for consistency. The `/api/reports/local/{key}` endpoint handles the actual serving.

4. **Add urlencoding to Cargo.toml (if not present)**

   Check if already available, otherwise add:
   ```toml
   urlencoding = "2.1"
   ```

## Todo List

- [ ] Add `serve_local_report` handler function
- [ ] Implement path traversal protection
- [ ] Set correct response headers (Content-Type, Content-Disposition)
- [ ] Add route to router in `lib.rs`
- [ ] Add `urlencoding` dependency if needed
- [ ] Test with various key formats
- [ ] Test 404 for missing files
- [ ] Test path traversal attempts are blocked

## Success Criteria

- [ ] `/api/reports/local/{key}` serves Excel files correctly
- [ ] Content-Disposition triggers browser download
- [ ] Path traversal attempts return 403
- [ ] Missing files return 404 (not 500)
- [ ] Files stream without loading entirely into memory

## Security Considerations

- **Path Traversal**: canonicalize() + starts_with() check
- **Information Disclosure**: Generic error messages, no path details in responses
- **Access Control**: Consider adding workspace validation (key contains workspace_id)

## Performance Notes

- `ReaderStream` provides async streaming without loading full file
- Content-Length header enables progress bars in browsers
- No caching headers to ensure fresh report downloads

## Next Steps

Proceed to [Phase 5: Testing](./phase-05-testing.md)
