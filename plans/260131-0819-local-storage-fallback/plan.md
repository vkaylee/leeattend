---
title: "Local File Storage Fallback for Report Exports"
description: "Add local filesystem storage when S3 is unavailable for report exports"
status: pending
priority: P2
effort: 3h
branch: main
tags: [storage, reports, fallback, infrastructure]
created: 2026-01-31
---

# Local File Storage Fallback Implementation Plan

## Overview

Add local filesystem storage fallback to the report export system when AWS S3 is not configured or unavailable.

**Problem**: Currently, when S3_BUCKET is not set or upload fails, generated Excel reports are discarded and users receive broken download links.

**Solution**: Implement a storage abstraction layer that:
1. Detects storage configuration (S3 vs Local)
2. Saves reports to local `storage/reports/` directory when S3 unavailable
3. Serves local files via Axum when download URLs point to local storage

## Phases

| # | Phase | Status | Est. Time |
|---|-------|--------|-----------|
| 1 | [Configuration Setup](./phase-01-configuration.md) | pending | 30m |
| 2 | [Storage Abstraction Layer](./phase-02-storage-abstraction.md) | pending | 1h |
| 3 | [Worker Handler Integration](./phase-03-worker-integration.md) | pending | 45m |
| 4 | [Download Handler Updates](./phase-04-download-handler.md) | pending | 30m |
| 5 | [Testing](./phase-05-testing.md) | pending | 15m |

## Key Architectural Decisions

1. **Storage Detection at Startup**: Check `STORAGE_TYPE` env var; fallback to local if S3_BUCKET missing
2. **Local URL Scheme**: Use `/api/reports/local/{job_id}/{filename}` for local files
3. **Directory Structure**: `./storage/reports/{workspace_id}/{year}/{month}/{filename}`
4. **Cleanup Strategy**: Rely on existing Redis TTL (24h) to trigger cleanup of local files

## Dependencies

- `tokio::fs` (already available via tokio full features)
- No new crate dependencies required

## Files to Modify

| File | Change Type |
|------|-------------|
| `apiserver/src/services/mod.rs` | Add storage module export |
| `apiserver/src/services/storage.rs` | New - Storage abstraction trait + impls |
| `apiserver/src/worker/handlers/merge.rs` | Replace S3 upload with storage abstraction |
| `apiserver/src/handlers/report_export.rs` | Add local file serving in download handler |
| `apiserver/src/lib.rs` | Add storage to AppState |
| `apiserver/src/main.rs` | Initialize storage provider |

## Risk Assessment

- **Low**: Path traversal - mitigated by UUID-based filenames and directory validation
- **Low**: Disk space - reports are temporary (24h TTL), workspace-scoped
- **Medium**: Concurrent access - handle with atomic file operations

## Success Criteria

- [ ] Reports saved locally when S3_BUCKET unset
- [ ] Download endpoint serves local files correctly
- [ ] No data loss during storage failures
- [ ] Backward compatible with S3 deployments
