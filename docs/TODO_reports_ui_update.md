# TODO: UI Updates for Split Work Units (Std/OT)

## Backend Changes (Prerequisite)
- [ ] In `DailyAttendanceReport` struct (apiserver/src/models.rs), split `work_units` into `std_work_units` and `ot_work_units`.
- [ ] Add `ot_claim_status` field (Pending/Approved/None) to `DailyAttendanceReport` to track claim status.

## Frontend Updates (/workspace/reports)
- [ ] Update `DailyAttendanceTable` columns:
    - Replace single "Hours" column with "Std Hours" and "OT Hours".
- [ ] Visual Indicator for Unclaimed OT:
    - If `ot_work_units > 0` AND `ot_claim_status == None/Pending`, show a warning icon or highlight the OT cell.
    - Tooltip: "OT hours not yet claimed/approved".
- [ ] Add "Claim OT" action button in the row actions if uncliamed OT exists.
