# Migration catalog oracle

Status: disposable catalog captured; phased SQL not generated.

## Capture

Schema-only export captured from development database through repository wrapper on 2026-11-17. Wrapper diagnostics were separated from SQL output:

```bash
leedevkit dev exec db pg_dump \
  -U admin \
  -d leeattend \
  --schema-only \
  --no-owner \
  --no-acl \
  2>/tmp/migration-oracle-wrapper.stderr \
  | awk 'BEGIN{started=0} /^-- PostgreSQL database dump/{started=1} started{print}' \
  > /tmp/leeattend-schema-oracle.sql
```

Captured clean SQL contains 11,243 lines. No data, owner, ACL, secret, or PII export was requested.

## Catalog summary

Captured current schema reports:

| Object | Count |
|---|---:|
| Tables/partitions | 161 |
| Views | 1 |
| Public functions | 258 |
| Non-internal triggers | 78 |
| Indexes | 537 |

This is the oracle for the five fresh-install phases. Phase SQL must reproduce final schema behavior, not historical migration steps.

## Required comparison dimensions

- extensions
- enums and other types
- tables and partitions
- columns, types, nullability, defaults, generated expressions
- primary, foreign, unique, check, and exclusion constraints
- indexes and predicates
- functions and trigger definitions
- views and definitions
- required seed contracts

## Known special cases

- `20261116000000_atomic_official_shift_swaps` now has transaction and marker boundaries. Existing dev DB was created before this fix and therefore has columns/index/constraints but no marker; fresh probe must apply corrected migration.
- Current dev schema has 30 migration markers because the markerless shift-swap migration was applied without recording its marker. Do not repair shared dev marker state manually; recreate disposable DB for final verification.
- Existing dev schema is not used as proof of clean fresh install.
- Applying the full chain through `leedevkit dev exec` exceeded timeout in disposable probe at baseline. The probe was stopped and the disposable database was removed. Retry must use test compose's existing migration lifecycle or a dedicated disposable database runner with stdin detached.

## Acceptance gate

Do not archive current numeric migrations until:

1. fresh five-phase install applies on disposable database;
2. phase rerun is idempotent;
3. phase SQL failure leaves no marker or partial objects;
4. catalog matches this oracle across all dimensions;
5. schema contract passes with phase markers replacing historical markers;
6. standalone bootstrap succeeds after schema phases;
7. `leedevkit test api` and `leedevkit test all` complete with logs inspected.
