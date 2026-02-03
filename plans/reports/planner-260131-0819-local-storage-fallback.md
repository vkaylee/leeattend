# Planner Report: Local Storage Fallback

**Date**: 2026-01-31 08:19
**Agent**: planner-260131-0819
**Plan**: `/home/elt1541/leeattend/plans/260131-0819-local-storage-fallback/`

## Summary

Created comprehensive implementation plan for adding local file storage fallback to the report export system when AWS S3 is unavailable.

## Problem Analysis

Current behavior in `merge.rs`:
- When S3 upload fails, generates placeholder URL `/api/reports/{job_id}/download`
- **Critical bug**: Excel file data (`excel_data` bytes) is discarded completely
- Result: Broken download links and data loss for users without S3

## Solution Architecture

```
STORAGE_TYPE / S3_BUCKET Detection
         │
         ├── S3_BUCKET set → S3Storage (existing behavior)
         │
         └── S3_BUCKET unset → LocalStorage
                                  │
                                  ├── Save to: ./storage/reports/{ws_id}/{year}/{month}/{file}
                                  │
                                  └── Serve via: GET /api/reports/local/{encoded_key}
```

## Plan Structure

| Phase | File | Description | Est. Time |
|-------|------|-------------|-----------|
| 1 | `phase-01-configuration.md` | StorageBackend enum, env var detection | 30m |
| 2 | `phase-02-storage-abstraction.md` | StorageProvider trait + impls | 1h |
| 3 | `phase-03-worker-integration.md` | Update merge handler | 45m |
| 4 | `phase-04-download-handler.md` | Add local file serving endpoint | 30m |
| 5 | `phase-05-testing.md` | Unit tests + manual verification | 15m |

**Total Estimated Effort**: 3 hours

## Key Design Decisions

1. **Fail-fast on storage errors**: Storage failures now fail the job instead of silently losing data
2. **Atomic file writes**: Use temp file + rename pattern to prevent partial reads
3. **Path traversal protection**: canonicalize() + starts_with() validation
4. **No new dependencies**: Uses existing tokio::fs, urlencoding available in crate

## Files to Modify

| File | Change |
|------|--------|
| `apiserver/src/lib.rs` | Add StorageBackend enum, AppState field |
| `apiserver/src/main.rs` | Initialize storage provider |
| `apiserver/src/services/mod.rs` | Export storage module |
| `apiserver/src/services/storage.rs` | **NEW** - Trait + S3/Local impls |
| `apiserver/src/worker/handlers/merge.rs` | Use storage abstraction |
| `apiserver/src/handlers/report_export.rs` | Add local file serving |

## Risk Assessment

- **Low**: Path traversal - mitigated by canonicalize + prefix check
- **Low**: Disk space - reports are temporary, workspace-scoped
- **Medium**: Concurrent file access - handled with atomic writes

## Unresolved Questions

1. **File cleanup strategy**: Plan recommends relying on Redis TTL (24h) for now; add background cleanup task if needed later
2. **Disk space monitoring**: Out of scope; handle at infrastructure level
