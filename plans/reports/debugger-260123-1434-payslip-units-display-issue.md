# Investigation Report: Payslip "413 Days" Display Issue

**Date:** 2026-01-23
**Investigator:** debugger agent
**Issue ID:** payslip-units-display
**Severity:** Medium (UI Display Issue)

---

## Executive Summary

**Issue:** Employee 1541's payslip for Dec 2025 displays "413 Days" - wrong unit label.
**Root Cause:** UI displays work **units** but labels them as "Days" - units are granular time measurements (e.g., 15-min blocks).
**Impact:** Confusing display - actual calculation correct, only presentation layer issue.
**Recommended Fix:** Convert units to days for display OR change label to "Units" with explanation.

---

## Technical Analysis

### 1. Database Schema

From `apiserver/migrations/20260117000001_add_units_to_payslips.sql`:
```sql
ALTER TABLE payslips ADD COLUMN standard_units INT NOT NULL DEFAULT 0;
ALTER TABLE payslips ADD COLUMN overtime_units INT NOT NULL DEFAULT 0;
```

**Key Finding:** `standard_units` stores **work units** (integer), not days.

### 2. Calculation Logic

**File:** `apiserver/src/services/payroll_service.rs`

**Lines 115-116:** Accumulators defined as integers
```rust
let mut total_std_units = 0;  // Integer counter for units
let mut total_ot_units = 0;
```

**Lines 161-162:** Units per day calculation
```rust
let seg_units_per_day = (std_config.hours_per_day * Decimal::from(60))
    / Decimal::from(workspace.work_unit_minutes);
```

**Example calculation:**
- If `hours_per_day = 8` and `work_unit_minutes = 15`:
- `seg_units_per_day = (8 * 60) / 15 = 32 units/day`

**Lines 209-217:** Daily unit accumulation
```rust
let (std_u, ot_u) = AttendanceCalculator::calculate_daily_units(
    punch_in, punch_out, Some(start), Some(end),
    s.break_duration_minutes, actual_break_minutes,
    workspace.work_unit_minutes, &workspace.rounding_policy,
    &rules
);
segment_std_units += std_u;  // Accumulating units (not days!)
segment_ot_units += ot_u;
```

**Lines 240-241:** Converting units back to days for salary calculation
```rust
let worked_days_dec = Decimal::from(segment_std_units) / seg_units_per_day;
let daily_rate = if std_config.std_working_days.is_zero() { ... } else { segment_base_salary / std_config.std_working_days };
```

**Lines 245-246:** Total accumulation
```rust
total_std_units += segment_std_units;  // Still in UNITS
total_ot_units += segment_ot_units;
```

**Lines 295:** Stored in database
```rust
standard_units: total_std_units,  // Stored as raw unit count
overtime_units: total_ot_units,
```

### 3. UI Display Logic

**File:** `webdashboard/src/components/payroll/PayrollTable.tsx`

**Line 86:** CORRECT label
```tsx
<span className="text-[10px] font-black text-slate-400 uppercase tracking-widest opacity-70">
  {slip.standard_units} Units • {slip.overtime_units} OT
</span>
```

**Analysis:** This component correctly displays "Units", not "Days".

---

**File:** `webdashboard/src/components/payroll/PayslipDocument.tsx`

**Line 127:** ❌ **BUG FOUND - INCORRECT LABEL**
```tsx
<td className='text-right py-4 font-bold text-slate-600'>
  {payslip.standard_units} Days
</td>
```

**CRITICAL ISSUE:** Displays raw unit count (e.g., 413) with label "Days" instead of converting to actual days.

**Expected behavior:**
```tsx
{(payslip.standard_units / unitsPerDay).toFixed(1)} Days
```

Where `unitsPerDay = (hours_per_day * 60) / work_unit_minutes`

**Example calculation:**
- `payslip.standard_units = 413`
- `hours_per_day = 8`
- `work_unit_minutes = 15`
- `unitsPerDay = (8 * 60) / 15 = 32`
- **Correct display:** `413 / 32 = 12.9 Days`
- **Current (wrong) display:** `413 Days`

---

**Additional Bug - Line 113:** Incorrect prorated amount calculation
```tsx
const proratedAmount = dailyRate * payslip.standard_units;
```

This multiplies daily rate by **units** instead of **days**, producing wrong amount.

**Should be:**
```tsx
const actualDays = payslip.standard_units / unitsPerDay;
const proratedAmount = dailyRate * actualDays;
```

Or better yet, use backend's `sc.standard_amount` (already available, see plan note line 84).

### 4. Evidence Analysis

**Scenario:** Employee worked Dec 2025 with:
- Work unit granularity: 15 minutes
- Hours per day: 8 hours
- Units per day: 32 (480 minutes ÷ 15 minutes)

**If `standard_units = 413`:**
- Actual work days = 413 ÷ 32 = **12.9 days**
- This is plausible if employee only worked partial month

**However, if display shows "413 Days" instead of "413 Units":**
- This is UI label bug, not calculation bug

---

## Root Cause Identification

**Primary Issue:** `PayslipDocument.tsx` line 127 displays raw work units with "Days" label.

**Chain of Events:**
1. Attendance calculation accumulates work in granular units (15-min blocks)
2. Backend stores total units as integer in `payslips.standard_units`
3. UI receives raw unit count (e.g., 413)
4. **`PayslipDocument.tsx` displays 413 with label "Days" instead of converting to actual days (12.9)**

**Verified Display Locations:**
- `PayrollTable.tsx` line 86 ✅ (shows "Units" - correct)
- `PayslipDocument.tsx` line 127 ❌ **BUG CONFIRMED**
- `PayslipDocument.tsx` line 152 ✅ (uses `seg.paid_work_days` - correct)
- `PayslipDocument.tsx` line 173 ✅ (shows "Units" for OT - correct)

---

## Solution Development

### Option 1: Convert Units to Days for Display (Recommended)

**Location:** `webdashboard/src/components/payroll/PayslipDocument.tsx`

**Change line 127 from:**
```tsx
<td className='text-right py-4 font-bold text-slate-600'>
  {payslip.standard_units} Days
</td>
```

**To:**
```tsx
<td className='text-right py-4 font-bold text-slate-600'>
  {payslip.segments?.[0]?.paid_work_days.toFixed(1) || '—'} Days
</td>
```

**Rationale:** Use actual calculated `paid_work_days` from segments (already computed by backend).

**Pros:**
- Uses backend calculation (authoritative source)
- No need to recompute units-to-days conversion
- Consistent with line 152 logic

**Cons:**
- Assumes single segment or uses first segment only
- May not reflect total if multiple segments exist

### Option 2: Keep Units, Add Tooltip (Alternative)

**Change line 86 to:**
```tsx
<Tooltip content="Work units (15-min blocks). 32 units = 1 day">
  {slip.standard_units} Units • {slip.overtime_units} OT
</Tooltip>
```

**Pros:**
- Accurate representation
- Educational for users

**Cons:**
- Still confusing at first glance

### Option 3: Show Both (Best UX)

```tsx
{(slip.standard_units / getUnitsPerDay(workspace)).toFixed(1)} Days
<span className="text-slate-300">({slip.standard_units} units)</span>
```

**Pros:**
- Clear for users
- Transparent calculation
- Debuggable

**Cons:**
- Slightly more UI space

---

## Action Items

### Immediate (Critical - Blocking Issue)
1. **Fix `PayslipDocument.tsx` line 127** ❌
   ```tsx
   // BEFORE (wrong):
   {payslip.standard_units} Days

   // AFTER (correct):
   {payslip.segments?.reduce((sum, seg) => sum + seg.paid_work_days, 0).toFixed(1) || '—'} Days
   ```

2. **Fix `PayslipDocument.tsx` line 113** ❌
   ```tsx
   // BEFORE (wrong):
   const proratedAmount = dailyRate * payslip.standard_units;

   // AFTER (correct):
   const proratedAmount = parseFloat(sc.standard_amount);
   // Backend already provides correct amount
   ```

3. **Add workspace config to PayslipDocument props** (needed for proper calculation if not using segments)
   - Pass `workspace` prop with `work_unit_minutes` and hours per day settings

### Short-term (1-2 days)
1. Add comprehensive tests for `PayslipDocument` component
2. Verify all calculations match backend values
3. Add validation: alert if `standard_units` seems unreasonable (>1000 for monthly payslip)

### Long-term (1 week)
1. Standardize terminology: use "Work Days" consistently across all views
2. Add tooltips explaining units vs days
3. Document work unit system in user-facing help docs

---

## Supporting Evidence

### Code References
- **Calculation logic:** `/home/elt1541/leeattend/apiserver/src/services/payroll_service.rs` (lines 115, 161-162, 209-217, 240-246, 295)
- **Database schema:** `/home/elt1541/leeattend/apiserver/migrations/20260117000001_add_units_to_payslips.sql`
- **UI display:** `/home/elt1541/leeattend/webdashboard/src/components/payroll/PayrollTable.tsx` (line 86)

### Related Commits
- `4bf8f23` - "feat(payroll): implement unified payslip line items with standard_amount"
- `b5f0f9e` - "feat(payroll): add salary component breakdown to payslips"

### Migration Timeline
- 2026-01-17: Added `standard_units` and `overtime_units` columns
- 2026-01-23: Recent payslip UI work (PayslipDocument, PayslipModal)

---

## Preventive Measures

1. **Schema documentation** - Add comments to DB schema explaining unit vs day distinction
2. **Type safety** - Create distinct types: `WorkUnits`, `WorkDays` to prevent confusion
3. **Validation rules** - Add reasonable bounds (e.g., max 32 units/day for 15-min granularity)
4. **Monitoring** - Alert on payslips with `standard_units > expected_max` (e.g., >700 for 30-day month)
5. **UI/UX review** - Standardize terminology across all payroll views

---

## Summary & Recommendations

### Bug Confirmed
**Location:** `webdashboard/src/components/payroll/PayslipDocument.tsx:127`
**Issue:** Displays `{payslip.standard_units} Days` where `standard_units` is raw work units (413), not days.
**Impact:** User sees "413 Days" in official payslip document, should see "~12.9 Days".

### Secondary Bug
**Location:** `webdashboard/src/components/payroll/PayslipDocument.tsx:113`
**Issue:** Calculates prorated amount incorrectly by multiplying daily rate with units instead of days.
**Impact:** Wrong salary component amounts displayed in payslip.

### Recommended Fix Priority
1. **HIGH:** Fix line 127 display label (user-facing, causes confusion)
2. **HIGH:** Fix line 113 calculation (financial accuracy issue)
3. **MEDIUM:** Add tests to prevent regression
4. **LOW:** Standardize terminology across all components

### Implementation Strategy
Use backend-calculated values (`paid_work_days`, `standard_amount`) instead of recalculating in frontend. Backend is authoritative source.

---

## Unresolved Questions

1. Why was line 113-133 calculation added instead of using `sc.standard_amount` from backend?
2. Are there other payslips with this display issue in production?
3. Should we audit all displays of `standard_units` across the application?

---

## Next Steps

**Immediate action:** Implement fixes for lines 127 and 113 in `PayslipDocument.tsx`

**Testing:** Verify with actual payslip data showing correct days calculation

**Implementation priority:** Critical (financial document accuracy)
**Estimated fix time:** 30 minutes (simple display fix)
**Risk level:** Very Low (use existing backend values, no new logic)
