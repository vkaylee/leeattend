# Phase F — Migration rollout and rollback evidence

Migration: `20261117000000_schedule_day_offsets`

## Forward

- Additive columns on `shift_default_intervals`: day offsets and explicit UTC offsets, defaulting offsets to `0` for old writers.
- Additive nullable evidence columns on `work_segments`: offsets, `shift_group_key`, explicit UTC offsets.
- CHECK constraints enforce nonnegative ordered offsets and valid quarter-hour UTC offsets.
- Own `schema_migrations` marker inserted in same transaction.

## Rollback

- `down.sql` runs transactionally, drops only Phase C constraints/columns, and deletes only own marker.
- Safe only before expanded evidence is written. Once official evidence exists, roll-forward is required; no destructive rollback claim.
- Existing legacy rows remain readable because new work-segment fields are nullable.

## Mixed-version boundary

- Old readers ignore additive fields.
- Old writers remain compatible for defaulted interval offsets and nullable segment evidence until lifecycle activation requires complete official evidence.
- New readers require complete offset/group evidence only for newly published official segments.

## Verification

- Forward migration executed through repository migration wrapper during Phase C integration gate.
- Phase C focused repository integration passed after migration application.
- Down migration SQL reviewed for transaction and marker behavior; disposable empty-schema down execution remains required before release activation.
- No production rollback evidence claimed from source inspection alone.
