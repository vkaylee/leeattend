# Phase C — Day offsets, shift groups, DST

Test impact matrix:

- Behavior: interval/API carries nonnegative day offsets and stable `shift_group_key`; official segments retain both.
- Boundaries: first group segment starts offset 0; touching half-open endpoints allowed; overlaps and invalid offsets rejected.
- State: preview, publication, replan, evidence preserve normalized offsets/group identity.
- Failure: nonexistent DST local time rejected; ambiguous local time requires explicit UTC offset; no silent timezone selection.
- Persistence: additive migration keeps legacy rows nullable; official rows require complete offset/group evidence.
- Compatibility: old clients omit new fields and receive defaults (`start_day_offset=0`, end offset inferred from overnight only); old legacy rows remain readable.
- Concurrency/security: reuse existing lifecycle transaction and authorization; no new scope or writer.

Rule IDs: Development Workflow §§2–7; Database Rules §§3–4; API Governance §§1–6; Migration and Rollback §§1–6.
