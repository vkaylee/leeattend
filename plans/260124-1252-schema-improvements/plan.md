---
title: "Schema Design Improvements"
description: "Add leave_types reference table, missing indexes, and standardize PK types"
status: pending
priority: P2
effort: 1h
branch: main
tags: [database, schema, performance, migrations]
created: 2026-01-24
---

# Schema Design Improvements

## Overview

Address issues identified in Schema Design Review to improve data integrity, query performance, and consistency.

## Phases

| # | Phase | Status | Effort |
|---|-------|--------|--------|
| 1 | [Create leave_types reference table](./phase-01-leave-types-reference-table.md) | ✅ complete | 20m |
| 2 | [Add missing FK indexes](./phase-02-add-missing-indexes.md) | ✅ complete | 10m |
| 3 | [Standardize PK types](./phase-03-standardize-pk-types.md) | ✅ complete | 15m |
| 4 | [Add compound performance indexes](./phase-04-compound-indexes.md) | ✅ complete | 10m |
| 5 | [Update SQLx metadata](./phase-05-sqlx-prepare.md) | pending | 5m |

## Key Changes

1. **leave_types** - Reference table for leave type standardization
2. **Indexes** - Missing FK indexes + compound index on employees
3. **PK consistency** - Convert SERIAL to UUID in payslip_incentives

## Dependencies

- PostgreSQL with existing extensions (pg_trgm, btree_gist)
- SQLx for metadata preparation

## Success Criteria

- [x] All migrations apply cleanly
- [x] Existing data preserved
- [ ] `cargo sqlx prepare` succeeds
- [ ] Tests pass

## Review Notes

Code review completed 2026-01-24. Score: 8.5/10
- See [code-reviewer-260124-1600-schema-migrations.md](./reports/code-reviewer-260124-1600-schema-migrations.md)
- Minor improvements suggested: data migration validation, CONCURRENTLY indexes for production
- All migrations idempotent, secure, backward compatible
