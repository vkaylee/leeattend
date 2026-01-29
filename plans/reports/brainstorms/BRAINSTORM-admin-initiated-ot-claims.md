# BRAINSTORM: Admin-Initiated OT Claims Implementation

**Date:** 2026-01-28
**Context:** LeeAttend attendance system - enabling admins to create OT claims on behalf of employees
**Constraints:**
- Audit trail required (who initiated)
- Employee notification/consent needed
- Cannot exceed system_detected_ot_minutes
- Integration with existing /approvals/ot page
- Maintain compliance and transparency

---

## 1. Implementation Approaches

### Approach A: Draft-and-Notify (Two-Step Process)

**Description:** Admin creates OT claim in "draft" status, employee must explicitly accept before manager review.

**Key Features:**
- Admin initiates claim → `draft_by_admin` status
- Employee receives notification with OT details
- Employee can accept/reject via notification link
- Upon acceptance → standard `pending_approval` workflow
- Rejection returns to admin with reason

**Workflow:**
```
draft_by_admin → (employee accepts) → pending_approval → approved/rejected
                ↓ (employee rejects)
              returned_to_admin
```

**Pros:**
- Maximum transparency and employee control
- Clear consent mechanism
- Audit trail built-in (who drafted, who accepted)
- Low compliance risk
- Respects employee autonomy

**Cons:**
- Adds friction (extra approval step)
- Slower processing time
- Employee may not respond promptly
- Requires notification infrastructure
- More complex state machine

**Complexity:** Medium
**Risk Level:** Low
**Dev Effort:** 4-5 days

---

### Approach B: Pre-Approved Fast Track

**Description:** Admin-initiated claims bypass manager approval if within thresholds, directly marked approved.

**Key Features:**
- Admin creates claim → immediate `approved` status
- Employee notified post-facto
- Special flag: `admin_initiated: true, requires_employee_ack: true`
- Employee can dispute within 7 days
- No manager review needed (admin is trusted authority)

**Workflow:**
```
(admin creates) → approved (with admin_initiated flag) → [employee notified]
                                                         ↓ (dispute window)
                                                       [dispute if needed]
```

**Pros:**
- Fastest processing time
- Minimal workflow complexity
- Reduces manager workload
- Good for corrections/fixes
- Simple state machine

**Cons:**
- High compliance risk (no employee consent upfront)
- Potential abuse by admins
- Employee may feel powerless
- Dispute mechanism adds complexity
- May violate labor laws in some jurisdictions

**Complexity:** Low
**Risk Level:** High
**Dev Effort:** 2-3 days

---

### Approach C: Proxy Submission (Admin Acts As Employee)

**Description:** Admin creates claim as if employee submitted it, enters standard workflow with admin audit flag.

**Key Features:**
- Claim goes directly to `pending_approval` (standard workflow)
- Special metadata: `submitted_by_admin_id`, `submitted_on_behalf_of`
- Employee notified: "OT claim submitted on your behalf"
- Manager reviews as normal
- Employee can withdraw before approval

**Workflow:**
```
pending_approval (submitted_by_admin flag) → approved/rejected
     ↓ (employee can withdraw)
   withdrawn_by_employee
```

**Pros:**
- Uses existing approval workflow (minimal code change)
- Manager retains oversight
- Moderate employee control (can withdraw)
- Balanced transparency
- Clear audit trail

**Cons:**
- Employee notification is informational only (no consent)
- Withdrawal process may be unclear
- Manager may question legitimacy
- Moderate compliance risk
- Requires UI changes for admin submission

**Complexity:** Medium
**Risk Level:** Medium
**Dev Effort:** 3-4 days

---

### Approach D: Consent-with-Context (Informed Approval)

**Description:** Admin initiates claim with justification, employee sees context and explicitly approves, then manager review.

**Key Features:**
- Admin provides reason/context when creating claim
- Employee receives notification with admin's justification
- Employee must approve with full visibility
- Upon approval → standard manager workflow
- Employee can request modifications before accepting

**Workflow:**
```
admin_initiated (with context) → (employee approves) → pending_approval → approved/rejected
                                ↓ (employee requests change)
                           modification_requested → admin_revises → (loop)
```

**Pros:**
- High transparency with context
- Employee understands "why"
- Manager sees full history
- Excellent audit trail
- Supports collaborative corrections
- Low compliance risk

**Cons:**
- Most complex workflow
- Potential back-and-forth delays
- Requires rich notification system
- More UI development needed
- Higher testing overhead

**Complexity:** High
**Risk Level:** Low
**Dev Effort:** 6-7 days

---

### Approach E: Threshold-Based Automation

**Description:** Workflow changes based on OT amount - small claims auto-approved, large claims require review.

**Key Features:**
- Admin creates claim with auto-calculation
- If OT ≤ 30 minutes: auto-approved (admin authority)
- If OT > 30 minutes: requires employee + manager approval
- Configurable threshold in system settings
- Different notification templates per path

**Workflow:**
```
(admin creates) → [threshold check]
                       ↓ (≤30 min)              ↓ (>30 min)
                  auto_approved          pending_employee_consent → pending_approval
```

**Pros:**
- Flexible based on impact
- Fast for minor corrections
- Proper oversight for significant OT
- Configurable business rules
- Balances speed and control

**Cons:**
- Arbitrary threshold selection
- Complex branching logic
- Threshold debates (why 30 min?)
- Different UX paths confusing
- Requires admin configuration UI

**Complexity:** High
**Risk Level:** Medium
**Dev Effort:** 5-6 days

---

### Approach F: Manager Pre-Authorization

**Description:** Admin creates "pending" claim, manager reviews first, then employee sees already-approved claim.

**Key Features:**
- Admin submits → manager reviews immediately
- Manager approves/rejects before employee notification
- If approved by manager → employee notified of approved OT
- Employee can still dispute but claim already in system
- Inverted workflow order

**Workflow:**
```
pending_manager_review → (manager approves) → approved_pending_employee_ack
                        ↓ (manager rejects)
                       rejected_by_manager
```

**Pros:**
- Manager oversight before employee involvement
- Reduces employee decision burden
- Admin-manager collaboration
- Prevents invalid claims early
- Clear authority structure

**Cons:**
- Employee sees fait accompli
- Low employee agency
- Manager may rubber-stamp
- Compliance concerns (no upfront consent)
- Confusing "already approved" notification

**Complexity:** Medium
**Risk Level:** High
**Dev Effort:** 4-5 days

---

## 2. Categorization

### Conservative (Minimize Risk)
- **Approach A: Draft-and-Notify** - Employee consent required upfront
- **Approach D: Consent-with-Context** - Full transparency with justification

### Balanced (Moderate Features & Risk)
- **Approach C: Proxy Submission** - Uses existing workflow with audit flags
- **Approach E: Threshold-Based Automation** - Risk-based routing

### Aggressive (Speed Over Caution)
- **Approach B: Pre-Approved Fast Track** - Skip approvals entirely
- **Approach F: Manager Pre-Authorization** - Employee last in chain

---

## 3. Technical Considerations

### Database Schema Changes

**All Approaches Need:**
```sql
-- New columns in attendance_ot_claims table
ALTER TABLE attendance_ot_claims ADD COLUMN submitted_by_admin_id UUID REFERENCES users(id);
ALTER TABLE attendance_ot_claims ADD COLUMN admin_submission_reason TEXT;
ALTER TABLE attendance_ot_claims ADD COLUMN employee_consent_at TIMESTAMP;
ALTER TABLE attendance_ot_claims ADD COLUMN employee_consent_status VARCHAR(20); -- 'pending', 'accepted', 'rejected'
```

**Approach-Specific:**
- **A, D:** Need `draft_by_admin`, `returned_to_admin` statuses
- **B:** Need `admin_initiated` flag, dispute mechanism table
- **E:** Need `system_ot_threshold_minutes` config table
- **F:** Need `approved_pending_employee_ack` status

### API Endpoints Required

**Common:**
- `POST /api/admin/attendance/ot-claims/create-for-employee`
- `GET /api/ess/ot-claims/pending-consent` (employee view)
- `POST /api/ess/ot-claims/{id}/consent` (accept/reject)

**Approach-Specific:**
- **D:** `POST /api/ess/ot-claims/{id}/request-modification`
- **B:** `POST /api/ess/ot-claims/{id}/dispute`
- **E:** `GET /api/admin/config/ot-thresholds`

### UI Components Affected

**Admin Interface:**
- New "Create OT Claim for Employee" form in admin panel
- Employee selector, date/time picker, reason field
- Validation against system_detected_ot_minutes

**Employee Interface:**
- OT consent notification card (similar to leave approvals)
- Detailed view showing admin-provided context
- Accept/Reject buttons with optional comment

**Manager Interface:**
- Existing `/approvals/ot` page - add "submitted by admin" badge
- Filter option: "Admin-initiated claims"

### Migration Complexity

**Phase 1:** Schema changes (1 day)
- Add columns with NULL defaults
- Backfill existing claims with `submitted_by_admin_id = NULL`

**Phase 2:** Status enum updates (0.5 day)
- Extend status type if using approaches A, D, or F

**Phase 3:** Notification infrastructure (1-2 days)
- Email/in-app templates for employee consent
- Action links with JWT tokens

**Phase 4:** Rollback plan (critical)
- Feature flag: `enable_admin_initiated_ot_claims`
- Graceful degradation if disabled

---

## 4. Recommendation Matrix

| Approach | Dev Effort | Compliance Risk | UX Improvement | Maintainability | Priority Score |
|----------|-----------|----------------|----------------|-----------------|---------------|
| **A: Draft-and-Notify** | 4-5 days | ⭐ Low | ⭐⭐⭐ High | ⭐⭐⭐ Good | **8.5/10** |
| **B: Pre-Approved** | 2-3 days | ⚠️ High | ⭐⭐ Moderate | ⭐⭐ Fair | 5.0/10 |
| **C: Proxy Submission** | 3-4 days | ⚠️ Medium | ⭐⭐ Moderate | ⭐⭐⭐ Good | **7.5/10** |
| **D: Consent-with-Context** | 6-7 days | ⭐ Low | ⭐⭐⭐⭐ Excellent | ⭐⭐ Fair | 7.0/10 |
| **E: Threshold-Based** | 5-6 days | ⚠️ Medium | ⭐⭐⭐ High | ⭐ Poor | 6.0/10 |
| **F: Manager Pre-Auth** | 4-5 days | ⚠️ High | ⭐ Low | ⭐⭐ Fair | 4.5/10 |

### Scoring Criteria
- **Compliance Risk:** Lower is better
- **UX Improvement:** Higher is better (admin + employee experience)
- **Maintainability:** Long-term code health
- **Priority Score:** Weighted average (compliance 40%, UX 30%, effort 20%, maintainability 10%)

---

## 5. Recommended Approach

### 🥇 **Primary Recommendation: Approach A (Draft-and-Notify)**

**Rationale:**
1. **Compliance-First:** Explicit employee consent before OT claim enters payroll
2. **Balanced Effort:** 4-5 days is reasonable for the value delivered
3. **Clear Audit Trail:** Every action logged (admin create, employee accept/reject, manager approve)
4. **Extensible:** Can add features like bulk notifications, auto-reminders
5. **User Trust:** Employees feel respected, admins have tool for corrections

**Implementation Priorities:**
- Phase 1: Schema + basic create endpoint (2 days)
- Phase 2: Employee consent UI + notifications (2 days)
- Phase 3: Testing + integration with existing approvals (1 day)

### 🥈 **Alternative: Approach C (Proxy Submission)**

**Use If:**
- Need faster delivery (3-4 days vs 4-5)
- Organization has high trust culture
- Existing approval workflow is already robust
- Compliance requirements less strict

**Trade-off:**
- Less explicit consent mechanism
- Employee can only withdraw (not accept/reject)
- Faster integration with existing code

---

## 6. Hybrid Option (Best of Both)

### Approach A + Threshold Fast-Track

**Concept:** Combine Draft-and-Notify with smart automation
- OT ≤ 15 minutes: Admin creates → employee auto-notified → approved (assumed consent for trivial amounts)
- OT > 15 minutes: Full Draft-and-Notify workflow

**Benefits:**
- Fast corrections for clock-in/out mistakes
- Full consent for significant OT
- Best UX for both paths

**Additional Effort:** +1 day (threshold logic)

---

## 7. Unresolved Questions

### Policy Questions
1. What is the max time window for admin to create retroactive OT claims? (e.g., 30 days back)
2. Should employees have deadline to respond to consent requests? (e.g., auto-reject after 7 days)
3. Can admin modify/delete claim after employee consent but before manager approval?
4. What happens if employee is on leave when consent notification sent?

### Technical Questions
1. Should consent notifications support batch operations (multiple claims at once)?
2. Do we need API rate limiting for admin OT creation (prevent abuse)?
3. Should system send reminder notifications if employee hasn't responded in X days?
4. How to handle timezone differences for claims spanning DST transitions?

### Workflow Questions
1. If manager rejects admin-initiated claim, does it return to admin or just close?
2. Can employee add comments when accepting/rejecting consent?
3. Should manager see different approval UI for admin-initiated vs employee-submitted?
4. Do admin-initiated claims affect employee OT quotas/limits differently?

---

## 8. Next Steps

### If Proceeding with Approach A:
1. **Stakeholder Validation:** Present workflow to HR/compliance team
2. **Design Phase:** Create UI mockups for admin creation form and employee consent page
3. **Technical Spike:** Test notification delivery infrastructure (email + in-app)
4. **Plan Creation:** Run `/plan:fast` with this brainstorm context

### If Need More Clarity:
1. Schedule working session with product owner
2. Review labor law requirements with legal
3. Test workflow with sample users (admin + employee roles)
4. Prototype consent notification template

---

**Document Status:** ✅ Complete
**Ready for:** Decision + planning phase
**Estimated Total Effort (Approach A):** 4-5 development days + 2 days testing = **1.5 weeks**
