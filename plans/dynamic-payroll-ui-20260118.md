# Implementation Plan: Smart Master-Detail Payroll UI

## Approach
We will implement a **Smart Master-Detail Table** (Approach C from brainstorming) to handle the complexity of Dynamic Payroll V2. This approach balances the density of information required for payroll processing with the need to highlight complex multi-segment scenarios and pending adjustments.

**Key Features:**
1.  **Master Row:** Shows high-level payroll totals (Net Pay, Gross Pay) and status.
2.  **Smart Indicators:**
    *   **Multi-Segment Badge:** Visual indicator if an employee has split salary segments (e.g., mid-month raise).
    *   **Pending Adjustments Warning:** Actionable badge if unprocessed attendance adjustments exist.
3.  **Detail Panel (Expanded Row):**
    *   **Segment Visualizer:** A simplified timeline bar showing segment proportions.
    *   **Segment Table:** Detailed breakdown of each segment's calculation.
4.  **Adjustment Drawer:** A side-panel workflow to resolve attendance adjustments (Approve/Reject) without leaving the payroll context, triggering an automatic single-row recalculation.

## Steps

### 1. Analysis & Preparation (10 min)
- Review existing `PayrollPage.tsx` and `api/payroll.ts`.
- Ensure `shadcn/ui` components (`Table`, `Sheet`, `Badge`, `Button`, `HoverCard`) are available.

### 2. Backend Verification (Already Done)
- Endpoint `POST /workspaces/:ws_id/employees/:id/calculate-payroll` exists and is ready for "Single Row Recalculation".

### 3. Frontend Implementation

#### 3.1. Type Definitions (10 min)
- Update `webdashboard/src/types/payroll.ts` (or equivalent) to include `segments` in the payroll model if missing.
- Define types for `AttendanceAdjustment`.

#### 3.2. `SegmentVisualizer` Component (20 min)
- **File:** `webdashboard/src/components/payroll/SegmentVisualizer.tsx`
- **Function:** Render a horizontal bar chart representing the payroll period (e.g., Jan 1 - Jan 31).
- **Logic:** Calculate width % based on segment days / total days.
- **Style:** Use distinct colors for different segment types (e.g., "Standard", "Probation", "Promotion").

#### 3.3. `AdjustmentDrawer` Component (30 min)
- **File:** `webdashboard/src/components/payroll/AdjustmentDrawer.tsx`
- **UI:** usage of `Sheet` component.
- **Content:** List of pending adjustments for the specific employee and month.
- **Actions:** "Approve" / "Reject" buttons.
- **Integration:** Call `api.approve_adjustment` then callback to trigger payroll recalculation.

#### 3.4. `PayrollTable` Refactor (40 min)
- **File:** `webdashboard/src/components/payroll/PayrollTable.tsx` (Extract from Page if needed).
- **Change:** Convert to `tanstack/react-table` or enhance existing custom table.
- **Row Expansion:** Implement expandable rows to show `SegmentVisualizer` and segment details.
- **Status Column:** Add logic to show "⚠️ 3 Pending Adjustments" button which opens `AdjustmentDrawer`.

#### 3.5. Page Integration (20 min)
- **File:** `webdashboard/src/pages/PayrollPage.tsx`
- **Logic:**
    - Fetch adjustments alongside payroll (or on demand).
    - Handle "Recalculate Single" action: Call API -> Optimistically update row / Refetch single row.

### 4. Testing (20 min)
- **Unit Tests:**
    - `SegmentVisualizer` renders correct widths.
    - `PayrollTable` expands/collapses.
- **Manual Verification:**
    - Create an employee with a mid-month salary change (Multi-segment).
    - Create a pending attendance adjustment.
    - Verify UI shows both indicators.
    - Resolve adjustment -> Verify payroll recalculates.

## Timeline
| Phase | Duration |
|-------|----------|
| Types & Utils | 10 min |
| SegmentVisualizer | 20 min |
| AdjustmentDrawer | 30 min |
| Table Refactor | 40 min |
| Integration | 20 min |
| Testing | 20 min |
| **Total** | **2.5 Hours** |

## Security Checklist
- [ ] **RBAC:** Ensure only `admin` or `payroll_manager` can approve adjustments.
- [ ] **Validation:** Ensure adjustments belong to the correct workspace and employee.
- [ ] **State Consistency:** Ensure payroll totals match the sum of segments.
