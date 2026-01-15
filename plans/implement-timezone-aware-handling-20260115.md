# Implementation Plan: Timezone-Aware Handling

Standardize date and time handling across the backend to use Workspace-specific timezones instead of the server's local time (`Local`). This ensures accuracy for reporting, pairing, and payroll calculations regardless of where the server is hosted.

## Approach
- **Universal UTC Storage:** Continue storing all timestamps in UTC in the database.
- **Workspace-Contextual Processing:** Every time-sensitive operation must first fetch the workspace's configured IANA timezone (e.g., `Asia/Ho_Chi_Minh`).
- **Timezone-Generic Logic:** Refactor `AttendanceCalculator` and services to use `chrono-tz` and avoid `chrono::Local`.

## Steps

### 1. Update Dependencies & Models
- Ensure `chrono-tz` is available in `Cargo.toml`.
- Verify `Workspace` model in `apiserver/src/models.rs` includes `timezone: String`.

### 2. Refactor `AttendanceCalculator`
- Modify `AttendanceCalculator` methods to accept `DateTime<Tz>` or perform arithmetic in a way that is independent of `Local`.
- Update unit tests in `attendance_calculator.rs` to use specific timezones instead of `Local`.

### 3. Refactor `ReportService`
- Fetch workspace timezone at the start of `get_daily_attendance_report`.
- Replace `Local.from_local_datetime` with `tz.from_local_datetime` (where `tz` is the workspace `Tz`).
- Ensure `start_dt` and `end_dt` for database queries are correctly calculated based on the workspace's start-of-day and end-of-day in UTC.
- Use the workspace timezone for `date_naive()` checks and `time()` comparisons.

### 4. Refactor `PairingService` & `PayrollService`
- Audit `PairingService` to ensure `daily_reprocess` uses the workspace timezone for defining "the day" (00:00 to 23:59).
- Update `PayrollService` to use workspace-local time when determining which day a punch belongs to.

### 5. Verification & Testing
- **Unit Tests:** Update existing tests in `attendance_calculator.rs`.
- **Integration Tests:** Create `apiserver/tests/timezone_consistency_tests.rs` that:
    - Creates a workspace in a non-UTC timezone (e.g., `Asia/Ho_Chi_Minh` UTC+7).
    - Inserts logs that would cross UTC date boundaries (e.g., 01:00 AM local = 06:00 PM previous day UTC).
    - Verifies that the daily report correctly groups these logs under the local date.

## Timeline
| Phase | Duration |
|-------|----------|
| Models & Utilities | 15 min |
| AttendanceCalculator Refactor | 30 min |
| Report & Pairing Service Refactor | 45 min |
| Integration Testing | 30 min |
| **Total** | **2 hours** |

## Rollback Plan
- Revert changes to `ReportService` and `AttendanceCalculator`.
- Since database schema already has `timezone`, no DB rollback is needed unless the column is removed (not planned).

## Security Checklist
- [x] Input validation (validate IANA timezone string).
- [x] Auth checks (ensure users only access their own workspace settings).
- [x] Error handling (fallback to UTC if timezone parsing fails).
