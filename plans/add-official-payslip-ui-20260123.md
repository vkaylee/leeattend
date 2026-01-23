# Implementation Plan: Official Payslip Document UI

Create a professional, "Official Document" style payslip view that provides a clear breakdown of earnings, deductions, and incentives, suitable for printing and record-keeping.

## Approach
- **Why this solution**: Users (especially employees in the ESS portal) need a document-like view that feels official and trustworthy. The current expandable card view is good for quick checks but lacks the "official document" structure.
- **Alternatives considered**: 
  - Improving the inline card (rejected as it doesn't solve the "official document" feel).
  - Generating PDF on the server (rejected for MVP to keep implementation simple and fast, client-side printing/viewing is sufficient for now).

## Steps

1. **Create PayslipDocument Component** (25 min)
   - Create `webdashboard/src/components/payroll/PayslipDocument.tsx`.
   - This component will render the payslip in a classic A4-style layout.
   - It will include:
     - Header: Company Name, Logo placeholder, Payslip Period.
     - Employee Info: Name, ID, Department.
     - Earnings Table: Base Pay (segmented), Overtime.
     - Incentives Section: List of earned incentives/bonuses.
     - Totals: Net Pay, Gross Pay, Deductions.

2. **Create PayslipModal Component** (15 min)
   - Create `webdashboard/src/components/payroll/PayslipModal.tsx`.
   - Use Radix UI Dialog to wrap `PayslipDocument`.
   - Add a "Print" button that triggers `window.print()`.

3. **Integrate into PayrollTable** (10 min)
   - Modify `webdashboard/src/components/payroll/PayrollTable.tsx`.
   - Add a "View Official Payslip" button to each month's card.
   - Wire up the state to open the `PayslipModal`.

4. **Add Print-Specific Styles** (10 min)
   - Use Tailwind's `print:` modifiers or a dedicated `@media print` block to hide UI elements (buttons, sidebar, navbar) when printing.

## Implementation Details

### PayslipDocument Structure
```tsx
interface PayslipDocumentProps {
  employee: Employee;
  payslip: Payslip;
  workspace: Workspace;
}
// ... layout with table and totals ...
```

## Timeline
| Phase | Duration |
|-------|----------|
| Component Implementation | 25 min |
| Modal Wrapper | 15 min |
| Integration & State | 10 min |
| Print Optimization | 10 min |
| **Total** | **1 hour** |

## Rollback Plan
- Revert changes in `PayrollTable.tsx`.
- Delete the new files: `PayslipDocument.tsx`, `PayslipModal.tsx`.

## Security Checklist
- [x] Auth checks: Ensured by existing backend API guards.
- [x] Input validation: Prop types and TypeScript interfaces.
- [x] Error handling: Graceful loading states if data is missing.

---

## Implementation Status (2026-01-23)

### Completed Tasks
- [x] Created `PayslipDocument.tsx` (229 lines) - Professional A4-style layout
- [x] Created `PayslipModal.tsx` (76 lines) - Radix Dialog wrapper with print functionality
- [x] Integrated into `PayrollTable.tsx` - Added "Official Payslip" button
- [x] Added print-specific styles using Tailwind `print:` modifiers
- [x] **BONUS:** Backend salary component breakdown integration (unplanned)
  - Updated `apiserver/src/models.rs` - Added `SalaryComponentWithDefinition` struct
  - Updated `apiserver/src/handlers/payroll.rs` - Fetch salary components in payslip endpoints
  - Added TypeScript types in `webdashboard/src/lib/api/types.ts`

### Outstanding Issues (CRITICAL - Blocks Deployment)
1. **TypeScript build failure:** `PayrollTable.test.tsx` needs `employee`/`workspace` props added to test cases
2. **ESLint failure:** Warning in `PayrollSummaryTable.tsx:59` exceeds `max-warnings 0` policy
3. **Missing tests:** No test coverage for `PayslipDocument` and `PayslipModal` components
4. **Calculation issue:** Line 127 in `PayslipDocument.tsx` recalculates amount instead of using `standard_amount` from backend

### Code Review Report
See: `/home/elt1541/leeattend/webdashboard/plans/reports/code-reviewer-260123-1346-payslip-salary-components.md`

**Score:** 6.5/10
**Status:** 🔴 **NOT READY FOR DEPLOYMENT** (build failures must be fixed first)

### Next Actions
1. Fix TypeScript compilation errors in test file
2. Fix ESLint warning in PayrollSummaryTable
3. Add unit tests for new components
4. Fix amount calculation logic in PayslipDocument
