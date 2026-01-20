# Implementation Plan: Payroll Standard Days Traceability

### ## Approach
- **Generic Visualizer**: Refactor the existing `PriorityLegend` (from Shift scheduling) into a generic `HierarchyLegend` component. This ensures a consistent "Metro Map" visual language across the entire application for all hierarchical decision-making systems.
- **Backend Traceability**: Enhance the `payslip_segments` table and the `PolicyResolver` to track exactly which level (Individual, Department, or Workspace) provided the "Standard Days" configuration.

### ## Steps

1. **Backend: Enhance Payslip Segment Storage** (15 min)
   - Create a migration to add `std_days_source` (VARCHAR) to `payslip_segments` table.
   - Update `StandardMonthConfig` struct to include a `source` field.
   - Update `PolicyResolver::resolve_standard_month` to return the source (e.g., "individual", "department", "workspace").
   - Update `PayrollService` to store this source in the database during payslip generation.

2. **Frontend: Generic Hierarchy Visualizer** (20 min)
   - Refactor `webdashboard/src/components/shifts/PriorityLegend.tsx` into a reusable `HierarchyLegend.tsx` in `webdashboard/src/components/common/`.
   - Prop types:
     ```typescript
     interface HierarchyStep {
       label: string;
       priority: number;
       color: string;
       isActive?: boolean; // Highlight the "winner"
     }
     interface HierarchyLegendProps {
       steps: HierarchyStep[];
       title: string;
       explanation?: string;
       icon?: React.ReactNode;
     }
     ```

3. **Frontend: Payroll Integration** (15 min)
   - Update `webdashboard/src/lib/api/types.ts` to include `std_days_source` in `PayrollSegment`.
   - Create `webdashboard/src/components/payroll/PayrollPriorityLegend.tsx` that wraps `HierarchyLegend` with the 3-tier payroll hierarchy.
   - Integrate into `PayrollTable.tsx`: Add a popover or inline legend next to the "Std Days" column in the segment detail table.

4. **Testing** (20 min)
   - Update `apiserver/tests/payroll_service_tests.rs` to verify source tracking.
   - Add a Vitest test for the generic `HierarchyLegend`.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Backend Changes | 15 min |
| Generic Refactor | 20 min |
| UI Integration | 15 min |
| Testing & Verification | 20 min |
| **Total** | **~1 hour 10 min** |

### ## Rollback Plan
- Revert the migration adding the `std_days_source` column.
- Restore the original `PriorityLegend.tsx` from git history if refactoring breaks Shift UI.

### ## Security Checklist
- [ ] Ensure `std_days_source` is validated as one of the allowed enum values in the backend.
- [ ] No sensitive data exposed in the traceability metadata.
