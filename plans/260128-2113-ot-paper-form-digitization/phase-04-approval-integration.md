# Phase 04: Approval Integration

## Context Links
- [Phase 03 - Frontend Entry Form](/home/elt1541/leeattend/plans/260128-2113-ot-paper-form-digitization/phase-03-frontend-entry-form.md)
- [OtApprovalsPage](/home/elt1541/leeattend/webdashboard/src/pages/approvals/OtApprovalsPage.tsx)

## Overview
- **Priority**: P2
- **Status**: Pending
- **Effort**: 3 hours

Enhance OtApprovalsPage to show paper form entries with visual distinction and add entry button.

## Key Insights

From existing code:
- `OtApprovalsPage` uses `useAttendanceSummaries` hook
- Displays table with date, employee, system, claimed, reason, actions
- Currently shows `employee_id` UUID (not name) - opportunity to improve
- Filter is by month only, no status filter

Key enhancements:
- Add "Paper Form" badge for `entered_via === 'paper_form'`
- Add "Enter Paper Form" button in header
- Add filter toggle: All / Paper Forms / Self-Service
- Show employee name instead of UUID

## Requirements

### Functional
- "Enter Paper Form" button opens dialog
- Paper form entries show visual badge
- Filter by entry method (optional)
- Show "Entered by" user name on hover/click
- Approval workflow unchanged

### Non-Functional
- Badge styling consistent with existing UI
- Mobile responsive
- No performance impact (minimal DOM changes)

## UI Design

### Header Enhancement

```
+----------------------------------------------------------+
|  OT Approvals                     [Enter Paper Form] btn |
|  Review and approve overtime claims                      |
|                                                          |
|  [ < ] [  January 2026  ] [ > ]   [All v] filter dropdown|
+----------------------------------------------------------+
```

### Table Enhancement

```
| Date       | Employee      | Source      | System | Claimed | Reason           | Actions |
|------------|---------------|-------------|--------|---------|------------------|---------|
| 2026-01-28 | Nguyen Van A  | PAPER FORM  | 90m    | 120m    | "Production..." | [✓] [✗] |
| 2026-01-27 | Tran Thi B    | Self-service| 60m    | 60m     | "Urgent..."     | [✓] [✗] |
```

### Badge Component

```tsx
function SourceBadge({ enteredVia }: { enteredVia?: string }) {
  if (enteredVia === "paper_form") {
    return (
      <span className="px-2 py-0.5 rounded-full text-[8px] font-bold uppercase tracking-wider bg-amber-100 text-amber-700 border border-amber-200">
        Paper Form
      </span>
    );
  }
  return (
    <span className="px-2 py-0.5 rounded-full text-[8px] font-bold uppercase tracking-wider bg-slate-100 text-slate-500">
        Self-service
    </span>
  );
}
```

## Implementation

### Modified OtApprovalsPage

```tsx
// webdashboard/src/pages/approvals/OtApprovalsPage.tsx

import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAttendanceSummaries, useApproveOt } from "@/hooks/use-attendance";
import { EnterPaperOtDialog } from "@/components/approvals/EnterPaperOtDialog";
import { startOfMonth, endOfMonth, format, subMonths, addMonths } from "date-fns";
import {
  CheckCircle, XCircle, ChevronLeft, ChevronRight,
  FileText, Filter
} from "lucide-react";
import * as Popover from "@radix-ui/react-popover";
import { MonthYearPicker } from "@/components/common/MonthYearPicker";

type SourceFilter = "all" | "paper_form" | "self_service";

export default function OtApprovalsPage() {
  const { t } = useTranslation();
  const [currentMonth, setCurrentMonth] = useState(new Date());
  const [sourceFilter, setSourceFilter] = useState<SourceFilter>("all");
  const [dialogOpen, setDialogOpen] = useState(false);

  const startDate = format(startOfMonth(currentMonth), "yyyy-MM-dd");
  const endDate = format(endOfMonth(currentMonth), "yyyy-MM-dd");

  const { summaries, isLoading, refetch } = useAttendanceSummaries(startDate, endDate);
  const { approveOt, isProcessing } = useApproveOt();

  // Filter by status and source
  const pendingSummaries = summaries
    .filter(s => s.ot_status === "pending_manager")
    .filter(s => {
      if (sourceFilter === "all") return true;
      return s.entered_via === sourceFilter;
    });

  const handleApprove = async (summary: any, approved: boolean) => {
    try {
      await approveOt({
        employeeId: summary.employee_id,
        date: summary.date,
        approved_minutes: approved ? summary.claimed_ot_minutes : 0,
        status: approved ? "approved" : "rejected",
        manager_note: approved ? "Approved" : "Rejected"
      });
      refetch();
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="space-y-10 p-8">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8">
        <div className="space-y-1">
          <h1 className="text-4xl font-bold tracking-tighter text-slate-700 uppercase">
            OT Approvals
          </h1>
          <p className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em]">
            Review and approve overtime claims
          </p>
        </div>

        <div className="flex items-center gap-4">
          {/* Enter Paper Form Button */}
          <button
            onClick={() => setDialogOpen(true)}
            className="flex items-center gap-2 px-5 py-3 rounded-2xl bg-amber-500 text-white text-xs font-bold uppercase tracking-wider shadow-lg hover:bg-amber-600 transition-all"
          >
            <FileText size={16} />
            {t("approvals.paper_ot.btn_enter") || "Enter Paper Form"}
          </button>

          {/* Month Picker - existing code */}
          {/* ... */}
        </div>
      </div>

      {/* Filter Bar */}
      <div className="flex items-center gap-4">
        <Filter size={16} className="text-slate-400" />
        <div className="flex gap-2">
          {(["all", "paper_form", "self_service"] as SourceFilter[]).map((filter) => (
            <button
              key={filter}
              onClick={() => setSourceFilter(filter)}
              className={`px-3 py-1.5 rounded-lg text-[10px] font-bold uppercase tracking-wider transition-all ${
                sourceFilter === filter
                  ? "bg-primary-600 text-white"
                  : "bg-slate-100 text-slate-600 hover:bg-slate-200"
              }`}
            >
              {filter === "all" ? "All" : filter === "paper_form" ? "Paper Forms" : "Self-Service"}
            </button>
          ))}
        </div>
      </div>

      {/* Table */}
      <div className="bg-white border border-slate-100 rounded-[3rem] overflow-hidden shadow-sm">
        {isLoading ? (
          <div className="p-32 text-center">Loading...</div>
        ) : pendingSummaries.length === 0 ? (
          <div className="py-40 text-center">
            <CheckCircle size={40} className="text-emerald-200 mx-auto mb-4" />
            <h3 className="text-xl font-bold text-slate-700">All Caught Up!</h3>
          </div>
        ) : (
          <table className="w-full text-left">
            <thead>
              <tr className="bg-slate-50/50 border-b border-slate-100">
                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase">Date</th>
                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase">Employee</th>
                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase">Source</th>
                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase text-center">System</th>
                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase text-center">Claimed</th>
                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase">Reason</th>
                <th className="px-8 py-6 text-[10px] font-bold text-slate-500 uppercase text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-50">
              {pendingSummaries.map((summary) => (
                <tr key={summary.id} className="hover:bg-slate-50/50">
                  <td className="px-8 py-6 font-bold text-slate-700">{summary.date}</td>
                  <td className="px-8 py-6 text-sm text-slate-600">
                    {/* TODO: Join with employee name */}
                    {summary.employee_id.substring(0, 8)}...
                  </td>
                  <td className="px-8 py-6">
                    <SourceBadge enteredVia={summary.entered_via} />
                  </td>
                  <td className="px-8 py-6 text-center font-mono text-slate-500">
                    {summary.system_detected_ot_minutes}m
                  </td>
                  <td className="px-8 py-6 text-center font-mono font-bold text-amber-600">
                    {summary.claimed_ot_minutes}m
                  </td>
                  <td className="px-8 py-6 text-sm text-slate-600 italic max-w-xs truncate">
                    "{summary.ot_reason}"
                  </td>
                  <td className="px-8 py-6 text-right">
                    <div className="flex items-center justify-end gap-2">
                      <button
                        onClick={() => handleApprove(summary, true)}
                        disabled={isProcessing}
                        className="p-2 rounded-full hover:bg-emerald-50 text-slate-400 hover:text-emerald-600"
                      >
                        <CheckCircle className="w-5 h-5" />
                      </button>
                      <button
                        onClick={() => handleApprove(summary, false)}
                        disabled={isProcessing}
                        className="p-2 rounded-full hover:bg-rose-50 text-slate-400 hover:text-rose-600"
                      >
                        <XCircle className="w-5 h-5" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Dialog */}
      <EnterPaperOtDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onSuccess={() => refetch()}
      />
    </div>
  );
}

// Badge Component
function SourceBadge({ enteredVia }: { enteredVia?: string }) {
  if (enteredVia === "paper_form") {
    return (
      <span className="px-2 py-0.5 rounded-full text-[8px] font-bold uppercase tracking-wider bg-amber-100 text-amber-700 border border-amber-200">
        Paper
      </span>
    );
  }
  return (
    <span className="px-2 py-0.5 rounded-full text-[8px] font-bold uppercase tracking-wider bg-slate-100 text-slate-500">
      Self
    </span>
  );
}
```

## Employee Name Enhancement

To show employee name instead of UUID, two options:

### Option A: Join in Backend Query (Recommended)

Modify `get_attendance_summaries` handler to join with employees table:

```rust
// In attendance.rs
sqlx::query!(
    r#"
    SELECT
        s.*,
        e.full_name as employee_name,
        u.email as entered_by_email
    FROM attendance_daily_summaries s
    JOIN employees e ON s.employee_id = e.id
    LEFT JOIN users u ON s.claim_entered_by_user_id = u.id
    WHERE s.workspace_id = $1 AND s.date >= $2 AND s.date <= $3
    ORDER BY s.date ASC
    "#,
    ...
)
```

### Option B: Frontend Lookup (Simple but N+1)

Fetch employee list separately and join in React:

```typescript
const { employees } = useEmployees();
const employeeMap = useMemo(() =>
  new Map(employees.map(e => [e.id, e])),
  [employees]
);

// In render
{employeeMap.get(summary.employee_id)?.full_name || summary.employee_id}
```

**Decision**: Start with Option B (simpler), refactor to Option A if performance issues.

## Related Code Files

### To Modify
- `/home/elt1541/leeattend/webdashboard/src/pages/approvals/OtApprovalsPage.tsx`

### Optional Backend Enhancement
- `/home/elt1541/leeattend/apiserver/src/handlers/attendance.rs` - Add employee name join

## Implementation Steps

1. [ ] Add "Enter Paper Form" button to header
2. [ ] Import and integrate `EnterPaperOtDialog`
3. [ ] Add `SourceBadge` component
4. [ ] Add "Source" column to table
5. [ ] Add filter buttons (All / Paper / Self)
6. [ ] Add employee name lookup (Option B)
7. [ ] Test filter behavior
8. [ ] Test dialog integration

## Todo List

- [ ] Modify OtApprovalsPage header
- [ ] Add SourceBadge component
- [ ] Add filter state and UI
- [ ] Integrate dialog
- [ ] Test full workflow

## Success Criteria

- [ ] "Enter Paper Form" button visible in header
- [ ] Paper form entries show amber "Paper" badge
- [ ] Self-service entries show slate "Self" badge
- [ ] Filter toggles work correctly
- [ ] Dialog opens and closes properly
- [ ] New entries appear after dialog submit

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Filter breaks existing flow | Medium | Default to "all" |
| Employee lookup N+1 | Low | Option B acceptable for small lists |

## Security Considerations

- Button only visible to Admin/Owner (same page access)
- Filter is client-side, doesn't expose hidden data

## Next Steps

After this phase:
- Phase 05 tests complete workflow
- Consider batch approval feature (future enhancement)
