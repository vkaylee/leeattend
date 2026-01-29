# Requirements Clarification: Admin-Initiated OT Claims

**Date:** 2026-01-28
**System:** LeeAttend - Attendance & Overtime Management
**Feature:** Allow administrators to proactively create OT claims for employees
**Current Location:** `/approvals/ot` page (administrator interface)

---

## 1. Problem Statement

### Current State
- **Employee-driven workflow**: System detects OT → Employee must manually claim via ESS portal → Manager approves/rejects
- **Status flow**: `None` → `PendingEmployee` → `PendingManager` → `Approved/Rejected`
- **Pain point**: If employee forgets to claim, legitimate OT goes unpaid despite system detection

### Proposed Enhancement
Allow administrators to bypass the employee claim step by directly creating OT claims on behalf of employees from the `/approvals/ot` page.

### Business Scenarios Requiring Admin Intervention
1. **Forgotten claims**: Employee worked OT but forgot to submit claim before payroll cutoff
2. **Bulk approvals**: Manager wants to pre-approve a team's OT hours after emergency deployment
3. **Retroactive corrections**: HR discovers unclaimed OT during audit review
4. **Onboarding/training gaps**: New employees unfamiliar with claim process
5. **System integration**: Automated workflows where external systems trigger OT claim creation

---

## 2. Key Constraints Identified

### A. Audit & Compliance Requirements
**Critical Question:** Who initiated the claim?

Current data model (`attendance_daily_summaries`):
- `claimed_ot_minutes` - amount claimed
- `ot_reason` - employee's justification
- `manager_note` - manager's approval note

**Missing traceability:**
- No `claimed_by_user_id` field (could be employee OR admin)
- No `claim_created_at` timestamp separate from `updated_at`
- No distinction between employee self-claim vs admin-initiated

**Compliance risk:** Labor audits require proof that employee consented to OT claim. Admin-initiated claims without employee visibility may violate labor laws in some jurisdictions.

### B. Employee Consent & Notification
**Question:** Should employees be notified when admin creates claim on their behalf?

Scenarios:
1. **Proactive approval** (admin creates → auto-approved): Employee may not be aware until payslip
2. **Admin create → employee review**: Requires new status `PendingEmployeeReview`
3. **Admin create → manager approve**: Skips employee involvement entirely

**Recommendation:** Implement email/in-app notification + ESS audit log showing admin-initiated claims.

### C. Existing Workflow States
Current `OtStatus` enum (Rust):
```rust
pub enum OtStatus {
    None,
    PendingEmployee,
    PendingManager,
    Approved,
    Rejected
}
```

**New state needed?**
- Option 1: Reuse `PendingManager` (admin claim goes directly to manager approval queue)
- Option 2: Add `AdminInitiated` state for traceability
- Option 3: Skip manager approval (admin has authority to approve immediately)

**Decision required:** Should admin-initiated claims auto-approve or require secondary manager review?

### D. Data Model Implications
Required schema changes:
```sql
ALTER TABLE attendance_daily_summaries
  ADD COLUMN claimed_by_user_id UUID REFERENCES users(id),
  ADD COLUMN claim_source TEXT CHECK (claim_source IN ('employee', 'admin', 'system')),
  ADD COLUMN claim_created_at TIMESTAMP WITH TIME ZONE;
```

**Migration risk:** Existing claims have no `claimed_by_user_id`. Backfill strategy needed.

### E. Permission & Authorization
**Question:** Who can create admin-initiated claims?

Current role hierarchy:
- `Owner` - full access
- `Admin` - workspace management
- `Member` - read-only or limited scope

**Recommendation:**
- Allow `Owner` and `Admin` roles
- Add audit log entry: "Admin [Name] created OT claim for Employee [Name] on [Date]"

---

## 3. Success Criteria

### Must-Have (Definition of Done)
1. **UI affordance**: Admin sees "Create Claim" button on `/approvals/ot` page
2. **Claim creation flow**: Modal to select employee, date, minutes, reason
3. **Traceability**: Database records who created the claim (employee vs admin)
4. **Validation**: Cannot claim more than `system_detected_ot_minutes`
5. **Status handling**: Admin-initiated claims move to `PendingManager` or auto-approve based on policy
6. **Employee visibility**: Claim appears in employee's ESS portal attendance history
7. **Audit trail**: Admin action logged in system audit log

### Should-Have (Enhancements)
1. **Bulk claim creation**: Upload CSV to create claims for multiple employees
2. **Notification**: Email/push notification to employee when admin creates claim
3. **Employee consent**: Add opt-out mechanism or require employee acknowledgment
4. **Policy-based auto-approval**: If admin creates claim, auto-approve if within policy limits

### Could-Have (Future Iteration)
1. **Delegation**: Allow managers to create claims for their direct reports only
2. **Templates**: Save common claim reasons for faster creation
3. **Integration API**: REST endpoint for external systems to create claims

---

## 4. Edge Cases to Handle

1. **Double claiming**: What if admin creates claim and employee also submits one for same date?
   - **Mitigation**: Check for existing claim with status `!= None` before creation
   - **UI**: Show warning "Claim already exists for this date"

2. **Retroactive claims after payroll finalized**: Can admin create claim for past payroll periods?
   - **Decision needed**: Lock claims after payroll publish date or allow with approval workflow?

3. **No system-detected OT**: Can admin create claim when `system_detected_ot_minutes = 0`?
   - **Scenario**: Manual time entry correction after device malfunction
   - **Recommendation**: Allow but require justification + flag for audit review

4. **Partial claims**: Admin creates 60min claim, employee later claims remaining 30min
   - **Current logic**: Second claim would overwrite first
   - **Fix needed**: Support additive claims or prevent duplicate claims

5. **Rejected claims**: If admin-initiated claim rejected, should employee be able to re-claim?
   - **Workflow**: Rejection should reset status to `PendingEmployee` for employee to revise

---

## 5. Open Questions for Stakeholder Confirmation

### Business Logic
1. **Q1:** Should admin-initiated claims auto-approve or require manager review?
   - **Impact:** Determines if we add new status state or reuse existing workflow

2. **Q2:** Should employees receive notification when admin creates claim on their behalf?
   - **Impact:** Compliance requirement in some jurisdictions

3. **Q3:** Can admin create claims when `system_detected_ot_minutes = 0`?
   - **Impact:** Enables manual corrections but increases fraud risk

### Technical Architecture
4. **Q4:** Store claim initiator in new column `claimed_by_user_id` or use audit log only?
   - **Impact:** Query performance vs data normalization

5. **Q5:** Should admin claims bypass employee consent workflow entirely?
   - **Impact:** Legal compliance risk vs operational efficiency

### User Experience
6. **Q6:** Where should admin initiate claim creation?
   - **Option A:** Existing `/approvals/ot` page (add "Create Claim" button)
   - **Option B:** Employee detail page under attendance tab
   - **Option C:** Both locations for flexibility

7. **Q7:** Should admin see all system-detected OT (including unclaimed) or only pending claims?
   - **Impact:** If showing unclaimed OT, need new data fetch endpoint

---

## 6. Recommended Solution Architecture

### Phase 1: Minimal Viable Feature
1. **UI change**: Add "Create Claim" button to `/approvals/ot` page
2. **Modal form**:
   - Employee selector (dropdown or search)
   - Date picker
   - Minutes input (max = `system_detected_ot_minutes`)
   - Reason textarea
   - "Submit" action
3. **Backend**:
   - New endpoint: `POST /api/v1/workspaces/:id/attendance/create-claim-as-admin`
   - Reuse existing `AttendanceSummaryService::claim_ot()` logic
   - Add `claimed_by_user_id` field to track admin initiator
   - Set status to `PendingManager` (requires manager approval)
4. **Database**: Add migration for new column

### Phase 2: Enhanced Workflow
1. Add notification to employee (email + in-app)
2. Employee can view admin-initiated claims in ESS portal with badge "Created by Admin"
3. Audit log integration
4. Bulk claim creation via CSV upload

### Phase 3: Policy Automation
1. Auto-approve admin-initiated claims if within policy threshold
2. Delegation: Managers create claims for direct reports only
3. Integration API for external systems

---

## 7. Risks & Mitigation

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Labor law violation** (admin claims without employee consent) | High | Add employee notification + consent workflow |
| **Audit trail gaps** (cannot prove who initiated claim) | High | Add `claimed_by_user_id` + `claim_source` fields |
| **Double claiming** (admin + employee claim same date) | Medium | Add uniqueness check + UI warning |
| **Fraud risk** (admins creating fake claims) | Medium | Audit log + policy limits + manager review |
| **Data migration failure** (backfilling `claimed_by_user_id` for existing claims) | Low | Mark old claims as `claim_source='legacy'` |

---

## 8. Next Steps

1. **Stakeholder review**: Present this document to product owner and legal team
2. **Answer open questions**: Schedule decision meeting for Q1-Q7
3. **Technical spike**: Estimate effort for DB migration and API changes (2-3 days)
4. **Design mockups**: Create UI wireframes for claim creation modal
5. **Implementation plan**: Break down into tasks with dependencies

**Estimated effort:** 5-8 days (1 dev, including testing and documentation)

---

## Appendix: Current System Behavior

### Existing Endpoints
```
POST /api/v1/workspaces/:id/attendance/claim-ot/:employee_id
  - Payload: { date, claimed_minutes, reason }
  - Auth: Employee (self-claim) or Admin (on behalf)
  - Status transition: PendingEmployee → PendingManager

POST /api/v1/workspaces/:id/attendance/approve-ot/:employee_id
  - Payload: { date, approved_minutes, status, manager_note }
  - Auth: Admin/Manager only
  - Status transition: PendingManager → Approved/Rejected

GET /api/v1/workspaces/:id/attendance/summaries
  - Query: start_date, end_date, employee_id (optional)
  - Returns: Array of AttendanceDailySummary
```

### Data Model
```sql
CREATE TABLE attendance_daily_summaries (
  id UUID PRIMARY KEY,
  workspace_id UUID NOT NULL,
  employee_id UUID NOT NULL,
  date DATE NOT NULL,
  actual_work_minutes INTEGER NOT NULL DEFAULT 0,
  system_detected_ot_minutes INTEGER NOT NULL DEFAULT 0,
  claimed_ot_minutes INTEGER NOT NULL DEFAULT 0,
  approved_ot_minutes INTEGER NOT NULL DEFAULT 0,
  ot_status ot_status_type NOT NULL DEFAULT 'none',
  ot_reason TEXT,
  manager_note TEXT,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  UNIQUE(workspace_id, employee_id, date)
);
```

### Current UI (OtApprovalsPage.tsx)
- Shows only `ot_status = 'pending_manager'` claims
- Manager can approve/reject with single click
- No visibility into unclaimed system-detected OT
- No ability to create claims
