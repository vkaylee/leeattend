# Phase F — Migration rollout and rollback evidence

Migration: `20261117000000_schedule_day_offsets`

## Forward

- Additive columns on `shift_default_intervals`: day offsets and explicit UTC offsets, defaulting offsets to `0` for old writers.
- Additive nullable evidence columns on `work_segments`: offsets, `shift_group_key`, explicit UTC offsets.
- CHECK constraints enforce nonnegative ordered offsets and valid quarter-hour UTC offsets.
- Own `schema_migrations` marker inserted in same transaction.

## Rollback

- `down.sql` runs transactionally, drops only Phase C constraints/columns, and deletes only own marker.
- Gap found in `20261117000000_schedule_day_offsets/down.sql`: unlike `20261113000000`..`20261116000000`, it has no in-SQL official-evidence guard (`DO $$ IF EXISTS ... RAISE EXCEPTION`). After official evidence exists, roll-forward is required operationally; do not rely on `20261117000000/down.sql` as self-guarding until guarded or permanently retired.
- Existing legacy rows remain readable because new work-segment fields are nullable.

## Mixed-version boundary

- Old readers ignore additive fields.
- Old writers remain compatible for defaulted interval offsets and nullable segment evidence until lifecycle activation requires complete official evidence.
- New readers require complete offset/group evidence only for newly published official segments.

## Verification

- Automated regression: `leedevkit test all --json` passed 3122/3122 across 6 phases (reported prior state; does not substitute for migration/rollout/production acceptance).
- Disposable PostgreSQL runtime rehearsal (`postgres:18-alpine`, isolated container removed on exit, 2026-11-25):
  - **M1 forward apply (`20261110000000` → `20261117000000_schedule_day_offsets`):** passed; 9 schedule lifecycle tables, 4 `shift_default_intervals` columns, 5 `work_segments` columns, 4 offset/UTC CHECK constraints, and `20261117000000_schedule_day_offsets` marker present.
  - **Phase F empty-schema rollback (`20261117000000_schedule_day_offsets/down.sql`):** passed; dropped all 4 `shift_default_intervals` columns, 5 `work_segments` columns, 4 CHECK constraints, and own marker while retaining `20261113000000_schedule_lifecycle` marker.
  - **M2 full schedule-lifecycle empty-schema rollback (`20261116000000` → `20261113000000`):** passed in reverse order; `remaining_lifecycle_tables=0`, `remaining_lifecycle_markers=0`.
  - **M3 official-evidence rollback refusal:** re-applied `20261113000000`..`20261117000000`, inserted 1 `published` `schedule_day_revisions` row and 1 official `work_segments` row (`start_day_offset=0`, `end_day_offset=1`, `shift_group_key='grp-1'`). `down.sql` refused cleanly on `20261115000000`, `20261114050000`, `20261114040000`, `20261114030000`, and `20261113000000` (`ERROR: ... rollback blocked: ... evidence exists`), leaving `preserved_revisions=1` and `preserved_segments=1`.
- **Not verified / remaining before activation:**
  - `20261117000000_schedule_day_offsets/down.sql` has no DB-level official-evidence guard.
  - Representative populated-data volume migration, M4 legacy backfill rehearsal on non-empty legacy fixtures, mixed-version rolling runtime verification, shadow zero-mismatch sign-off, and production activation/rollback are not proven.
