# Implementation Plan: UI Date/Time Synchronization

Synchronize all date and time displays in the UI with the workspace settings (`date_format` and `time_format`).

## Approach
- **Centralized Hook**: Create `useDateTimeFormatter` hook that fetches the current workspace settings and provides formatting functions.
- **Library**: Use `date-fns` for robust formatting (already used in the project).
- **Graceful Fallbacks**: Provide sensible defaults if workspace settings are loading or unavailable.

## Steps

1. **Create the Hook** (15 min)
   - File: `webdashboard/src/hooks/use-datetime.ts`
   - Implementation:
     ```typescript
     import { format } from "date-fns";
     import { useWorkspace } from "./use-workspace";

     export function useDateTimeFormatter() {
       const { workspace } = useWorkspace(); // Assuming useWorkspace provides current ws
       
       const dateFormat = workspace?.date_format || "DD/MM/YYYY";
       const timeFormat = workspace?.time_format || "24h";

       // Map backend formats to date-fns tokens
       const datePattern = dateFormat
         .replace("DD", "dd")
         .replace("YYYY", "yyyy");
       
       const timePattern = timeFormat === "12h" ? "hh:mm a" : "HH:mm";

       const formatDate = (date: string | Date | null) => {
         if (!date) return "---";
         return format(new Date(date), datePattern);
       };

       const formatTime = (date: string | Date | null) => {
         if (!date) return "---";
         return format(new Date(date), timePattern);
       };

       const formatDateTime = (date: string | Date | null) => {
         if (!date) return "---";
         return format(new Date(date), `${datePattern} ${timePattern}`);
       };

       return { formatDate, formatTime, formatDateTime };
     }
     ```

2. **Update Components** (45 min)
   - Replace manual `toLocaleDateString("vi-VN")` and `formatTime` util calls with the new hook.
   - Targeted files:
     - `ReportsPage.tsx`
     - `TimeTrackingPage.tsx`
     - `LeaveRequestsPage.tsx`
     - `AttendanceAdjustmentsPage.tsx`
     - `EmployeesPage.tsx`
     - `PayrollPage.tsx`
     - `ProfilePage.tsx`
     - `DashboardPage.tsx`
     - `RoleManagementModal.tsx`

3. **Validation** (15 min)
   - Change settings in Workspace Settings.
   - Verify all pages reflect the changes immediately.

## Timeline
| Phase | Duration |
|-------|----------|
| Hook Implementation | 15 min |
| Components Migration | 45 min |
| Testing | 15 min |
| **Total** | **1 hour 15 min** |

## Rollback Plan
- Revert changes to `ReportsPage.tsx` and other UI files.

## Security Checklist
- [x] No sensitive data exposure in formatting.
- [x] Handle invalid date strings gracefully without crashing.
