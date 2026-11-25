# Phase F — Migration rollout and rollback evidence

Migration: `20261117000000_schedule_day_offsets`

## Forward

- Additive columns on `shift_default_intervals`: day offsets and explicit UTC offsets, defaulting offsets to `0` for old writers.
- Additive nullable evidence columns on `work_segments`: offsets, `shift_group_key`, explicit UTC offsets.
- CHECK constraints enforce nonnegative ordered offsets and valid quarter-hour UTC offsets.
- Own `schema_migrations` marker inserted in same transaction.

## Rollback

- `down.sql` runs transactionally, enforces an in-SQL official-evidence and offset guard (`DO $$ IF EXISTS ... RAISE EXCEPTION 'schedule day offsets rollback blocked: official schedule or offset evidence exists'`), drops only Phase C constraints/columns, and deletes only own marker.
- After official evidence or non-default offset data exists, `down.sql` refuses rollback without mutating schema or rows; roll-forward is required.
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
  - **M3 official-evidence rollback refusal:** re-applied `20261113000000`..`20261117000000`, inserted 1 `published` `schedule_day_revisions` row and 1 official `work_segments` row (`start_day_offset=0`, `end_day_offset=1`, `shift_group_key='grp-1'` / `'grp-m3'`). `down.sql` refused cleanly on `20261117000000` (`ERROR: schedule day offsets rollback blocked: official schedule or offset evidence exists`), `20261115000000`, `20261114050000`, `20261114040000`, `20261114030000`, and `20261113000000`, leaving `preserved_published=1`, `preserved_offset_segments=1`, and `preserved_marker=1`.
  - **M4 populated legacy backfill rehearsal (`20261114040000_schedule_legacy_backfill/up.sql`):** seeded 2 employments with 2 unlinked legacy split `work_segments` (same day) and 2 legacy `shift_rosters` (1 template + 1 `NULL` explicit off) prior to `20261114040000`; verified `legacy_revs=3`, `official_revs=0`, `bad_pub_fields=0`, `bad_provenance=0`, `linked_legacy_segments=0`, and verified pre-official rollback of `20261117000000/down.sql` still succeeds when only `legacy_unverified` rows exist.
- **Not verified / remaining before activation:**
  - Large-scale production-volume data migration timing, mixed-version rolling runtime verification, shadow zero-mismatch sign-off, and production activation/rollback are not proven in local rehearsal.
