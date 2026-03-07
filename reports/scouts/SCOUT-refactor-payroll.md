# Scout Report: Refactor Payroll Service

## Exploration Scope
- Target: `calculate_monthly_payslip` in `payroll_service.rs`.
- Boundaries: `apiserver/src/services/payroll_service.rs`.

## Patterns Discovered
### Pattern: Massive Function / SRP Violation
- **Location**: `apiserver/src/services/payroll_service.rs:265-737`
- **Usage**: Calculating context resolution, shift resolution, exact attendance, daily financials, and incentive tracking all inside a single 470-line block with deep nested loops.
- **Must Follow**: No, this violates Single Responsibility Principle.

## Integration Points
| Point  | File | Function | New Code Location |
| ------ | ---- | -------- | ----------------- |
| Payroll Calc | `payroll_service.rs` | `calculate_monthly_payslip` | Same file, as helper methods (`calculate_daily_attendance`, `process_segment`, etc.) |

## Warnings
- ⚠️ Ensure that loop contexts (mutable accumulators like `segment_std_units`, `segment_ot_pay`, etc.) are preserved when breaking the function down.
