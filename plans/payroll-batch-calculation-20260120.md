# # Implementation Plan: Batch Payroll Calculation & Selection UI

### ## Approach
- **Hybrid Selection**: Enhance the `PayrollPage` sidebar to support multi-employee selection via checkboxes. This maintains the current "Detail View" (click to view history) while enabling "Batch Operations" (check to calculate).
- **Parallel Processing**: Utilize Rust's async capabilities (`join_all`) in the `PayrollService` to process multiple calculations concurrently, improving performance for administrative batch runs.
- **Transactional Safety**: Each calculation in the batch remains independent (errors in one employee don't block others), but the UI provides a unified progress summary.

### ## Steps

#### 1. Backend: Batch Logic (apiserver)
- **Modify `apiserver/src/models.rs`**:
  ```rust
  #[derive(Debug, Deserialize)]
  pub struct BatchCalculatePayrollRequest {
      pub employee_ids: Vec<Uuid>,
      pub year: i32,
      pub month: u32,
  }
  ```
- **Modify `apiserver/src/services/payroll_service.rs`**: Add `calculate_batch_payslips`.
- **Modify `apiserver/src/handlers/payroll.rs`**: Add `calculate_batch_payroll` handler and register route `POST /api/workspaces/:ws_id/payroll/calculate-batch`.

#### 2. Frontend: API & Hooks (webdashboard)
- **Modify `webdashboard/src/lib/api/payroll.ts`**: Add `calculateBatchPayslips`.
- **Modify `webdashboard/src/lib/api/types.ts`**: Add `BatchCalculatePayrollRequest` type.

#### 3. Frontend: Selection UI (webdashboard)
- **Modify `webdashboard/src/pages/PayrollPage.tsx`**:
  - Add `selectedIds: Set<string>` state.
  - Implement "Select All" toggle for the sidebar.
  - Add checkboxes to employee list items in the sidebar.
  - Add a "Batch Calculate" button that appears when `selectedIds.size > 0`.
- **Create `webdashboard/src/components/payroll/BatchCalculationModal.tsx`**:
  - Year/Month selection.
  - Progress tracker showing "Processing X of Y".
  - Success/Error breakdown after completion.

#### 4. Testing & Localization
- **Create `apiserver/tests/payroll_batch_tests.rs`**: Integration test for the batch endpoint.
- **Update `en.json`, `vi.json`, `zh.json`**: Add keys for batch calculation (Selection, Progress, etc.).

### ## Timeline
| Phase | Duration |
|-------|----------|
| Backend (Service/Handler) | 30 min |
| Frontend (API/Selection) | 20 min |
| Batch Progress UI | 30 min |
| i18n & Tests | 20 min |
| **Total** | **~1 hour 40 min** |

### ## Rollback Plan
- Revert changes to `PayrollPage.tsx` and `payroll.rs`.
- The new database records (`payslips`) created during batch calculation can be deleted manually if needed, but they don't corrupt existing state.

### ## Security Checklist
- [x] RBAC: Ensure only `Admin` or `Owner` can trigger batch calculation.
- [x] Input Validation: Validate `year` and `month` ranges.
- [x] Workspace Isolation: Verify all `employee_ids` belong to the same `workspace_id`.
