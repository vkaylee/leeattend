# LeeAttend Feature Specifications

**Version**: 1.0
**Last Updated**: 2026-01-28
**Total Features**: 95+

---

## Feature Organization

LeeAttend features are organized into 8 categories covering the complete attendance-to-payroll lifecycle.

---

## 1. Infrastructure & Tenancy (12 Features)

| Feature ID | Feature Name | Description | Category | Priority | Status |
|---|---|---|---|---|---|
| INF-01 | Multi-tenant workspace management | Separate organization instances with complete data isolation | Infrastructure | P0 | Complete |
| INF-02 | JWT authentication & authorization | Token-based auth with 1h access + 7d refresh tokens | Infrastructure | P0 | Complete |
| INF-03 | Role-based access control (RBAC) | Owner, Admin, Manager, Employee roles with fine-grained permissions | Infrastructure | P0 | Complete |
| INF-04 | Extra roles with temporal expiry | Temporary elevated permissions (e.g., Acting Manager) with auto-revocation | Infrastructure | P1 | Complete |
| INF-05 | User profile management | Personal information, preferences, avatar, language selection | Infrastructure | P1 | Complete |
| INF-06 | Workspace branding & customization | Custom theme color, logo, company name per workspace | Infrastructure | P1 | Complete |
| INF-07 | Timezone & localization settings | Per-workspace timezone and multi-language support | Infrastructure | P0 | Complete |
| INF-08 | Edge agent registration & trust approval | Agents register and await admin approval before log ingestion | Infrastructure | P0 | Complete |
| INF-09 | Device brand management | Support for ZKTeco, Hikvision, Suprema with sidecar pattern | Infrastructure | P0 | Complete |
| INF-10 | WebSocket real-time sync | Persistent agent-to-server connection for log streaming | Infrastructure | P0 | Complete |
| INF-11 | Offline-first edge architecture | Local SQLite cache with 365-day retention and automatic sync | Infrastructure | P0 | Complete |
| INF-12 | Exponential backoff retry logic | Configurable retry strategy with jitter and stability windows | Infrastructure | P0 | Complete |

---

## 2. Attendance Collection & Processing (12 Features)

| Feature ID | Feature Name | Description | Category | Priority | Status |
|---|---|---|---|---|---|
| ATT-01 | Biometric device polling | Fair round-robin polling with semaphore-based concurrency control | Collection | P0 | Complete |
| ATT-02 | Manual punch entry (admin) | HR admins can create attendance records manually | Collection | P1 | Complete |
| ATT-03 | Bulk import (Excel/CSV) | Import historical or batch attendance data from spreadsheets | Collection | P1 | Complete |
| ATT-04 | Raw log immutability | Never delete or modify raw logs (audit trail foundation) | Processing | P0 | Complete |
| ATT-05 | Duplicate detection | Identify and eliminate exact timestamp matches from multiple devices | Processing | P0 | Complete |
| ATT-06 | Pairing algorithm (Anchor-First) | Primary algorithm using first check-in and last check-out | Processing | P0 | Complete |
| ATT-07 | Pairing algorithm (FSM) | Finite state machine for explicit punch status handling | Processing | P1 | Complete |
| ATT-08 | Overnight shift support | Automatic previous-day lookback for night shift workers | Processing | P0 | Complete |
| ATT-09 | 16-hour session limit | Prevent multi-day sessions from incorrect pairing | Processing | P0 | Complete |
| ATT-10 | Automatic recalculation | Reprocess logs when new punch arrives for accurate pairing | Processing | P0 | Complete |
| ATT-11 | Generation metadata tracking | Store source log IDs and algorithm used in time_entry | Processing | P0 | Complete |
| ATT-12 | Real-time monitoring dashboard | Live device status, sync latency, error rates | Processing | P1 | Complete |

---

## 3. Attendance Adjustments & Corrections (4 Features)

| Feature ID | Feature Name | Description | Category | Priority | Status |
|---|---|---|---|---|---|
| ADJ-01 | Adjustment request (employee) | Employee submits punch time correction request | Correction | P1 | In Progress |
| ADJ-02 | Adjustment approval (admin) | Admin reviews and approves/rejects adjustments | Correction | P1 | In Progress |
| ADJ-03 | Time entry override | Replace time_entry with adjusted times (original logs preserved) | Correction | P1 | In Progress |
| ADJ-04 | Reason tracking | Store rationale for each adjustment for audit compliance | Correction | P1 | In Progress |

---

## 4. Shift & Schedule Management (10 Features)

| Feature ID | Feature Name | Description | Category | Priority | Status |
|---|---|---|---|---|---|
| SHIFT-01 | Shift definition | Create shifts with start time, end time, break duration | Scheduling | P0 | Complete |
| SHIFT-02 | Break duration management | Configurable break times (deducted from worked hours) | Scheduling | P0 | Complete |
| SHIFT-03 | Overtime eligibility flags | Mark shifts as overtime-eligible | Scheduling | P1 | Complete |
| SHIFT-04 | Working pattern templates | 7-day recurring schedules (Sun-Sat mapping) | Scheduling | P0 | Complete |
| SHIFT-05 | Direct shift roster | Date-specific shift assignments (overrides pattern) | Scheduling | P0 | Complete |
| SHIFT-06 | Assignment rules | Condition-based auto-assignment (department, tags, employee IDs) | Scheduling | P1 | In Progress |
| SHIFT-07 | Priority-based rule evaluation | Higher priority rules evaluated first (winner-take-all) | Scheduling | P1 | In Progress |
| SHIFT-08 | Bulk assignment execution | Apply rules to 100+ employees in seconds | Scheduling | P1 | In Progress |
| SHIFT-09 | Effective shift resolution | Roster > Pattern > Rule precedence (point-in-time lookup) | Scheduling | P0 | Complete |
| SHIFT-10 | Trace hints for transparency | Show why employee was assigned specific shift | Scheduling | P1 | In Progress |

---

## 5. Leave Management (12 Features)

| Feature ID | Feature Name | Description | Category | Priority | Status |
|---|---|---|---|---|---|
| LEAVE-01 | Leave type catalog | Vacation, sick, maternity, unpaid, etc. (workspace-scoped) | Management | P0 | Complete |
| LEAVE-02 | Leave policy definitions | Templates for entitlement, accrual, carryover rules | Management | P0 | Complete |
| LEAVE-03 | 3-tier policy assignment | Workspace → Department → Employee hierarchy | Management | P0 | Complete |
| LEAVE-04 | Temporal policy tracking | Effective dates for policy version control | Management | P0 | Complete |
| LEAVE-05 | Auto-close previous assignments | Transactional lifecycle (new policy auto-closes old) | Management | P0 | Complete |
| LEAVE-06 | Leave balance calculation | Entitlement + carryover - consumed = available | Management | P0 | Complete |
| LEAVE-07 | Leave accrual logs | Audit trail of all balance changes (additions/deductions) | Management | P0 | Complete |
| LEAVE-08 | Leave request submission (ESS) | Employee submits request through portal | Workflow | P0 | Complete |
| LEAVE-09 | Approval workflow (admin) | Admin reviews, approves, or rejects with notes | Workflow | P0 | Complete |
| LEAVE-10 | Automatic balance deduction | Approved leaves automatically deduct from balance | Workflow | P0 | Complete |
| LEAVE-11 | Overlap validation | Prevent double-booking of approved requests | Workflow | P0 | Complete |
| LEAVE-12 | Paid vs unpaid classification | Leave type determines payroll impact | Management | P0 | Complete |

---

## 6. Payroll Calculation (15 Features)

| Feature ID | Feature Name | Description | Category | Priority | Status |
|---|---|---|---|---|---|
| PAY-01 | Multi-segment calculation | Split period into segments for mid-month changes | Calculation | P0 | Complete |
| PAY-02 | Change point detection | Identify events triggering segmentation (salary, transfer, policy) | Calculation | P0 | Complete |
| PAY-03 | Exact prorating | Calculate pay to the day for partial months | Calculation | P0 | Complete |
| PAY-04 | 3-tier policy resolution | Employee > Department > Workspace hierarchy | Calculation | P0 | Complete |
| PAY-05 | Point-in-time policy lookup | Use policies effective during period (historical accuracy) | Calculation | P0 | Complete |
| PAY-06 | Salary component catalog | Basic, housing, meal, transport, bonuses, etc. | Components | P0 | Complete |
| PAY-07 | Component assignment | Link components to employees with effective dates | Components | P0 | Complete |
| PAY-08 | Base for OT calculation | Mark components used for overtime rate calculation | Components | P1 | Complete |
| PAY-09 | Taxable component flagging | Identify which components are subject to tax | Components | P1 | Complete |
| PAY-10 | Attendance aggregation | Sum standard work, overtime, breaks from time_entries | Calculation | P0 | Complete |
| PAY-11 | Leave impact calculation | Unpaid leave deducts from base pay (prorated) | Calculation | P0 | Complete |
| PAY-12 | Payslip generation | Create draft payslips for review (status: draft → published → paid) | Output | P0 | Complete |
| PAY-13 | Payslip segment persistence | Immutable snapshots of calculations for each segment | Output | P0 | Complete |
| PAY-14 | Salary history tracking | Maintain historical component and config versions | Reporting | P1 | Complete |
| PAY-15 | Department-specific working standards | Departments can override workspace working day counts | Configuration | P0 | Complete |

---

## 7. Payroll Incentives (6 Features)

| Feature ID | Feature Name | Description | Category | Priority | Status |
|---|---|---|---|---|---|
| INCENT-01 | Incentive policy definitions | Define bonus criteria and payout rules | Policies | P1 | In Progress |
| INCENT-02 | Tier-based incentive evaluation | Multiple tiers with thresholds (e.g., attendance, behavior) | Policies | P1 | In Progress |
| INCENT-03 | Priority-based tier matching | First matching tier wins (generosity principle) | Policies | P1 | In Progress |
| INCENT-04 | Threshold criteria | Max late count, unpaid leave days, etc. | Policies | P1 | In Progress |
| INCENT-05 | Payslip incentive recording | Award record with tier reference (audit trail) | Output | P1 | In Progress |
| INCENT-06 | Bonus calculation workflow | Automatic evaluation during payroll generation | Calculation | P1 | In Progress |

---

## 8. Employee Self-Service (ESS) (10 Features)

| Feature ID | Feature Name | Description | Category | Priority | Status |
|---|---|---|---|---|---|
| ESS-01 | Identity-linked employee portal | User account linked to employee record | Portal | P0 | Complete |
| ESS-02 | Personal profile management | Update phone, address, emergency contacts | Profile | P1 | Complete |
| ESS-03 | Attendance history view | Display raw logs and paired time_entries with filtering | History | P0 | Complete |
| ESS-04 | Leave balance display | Real-time balance view with trend indicators | Balances | P0 | Complete |
| ESS-05 | Leave request submission | Submit, track status, view approval history | Requests | P0 | Complete |
| ESS-06 | Payslip access | View historical payslips and download as PDF | Documents | P0 | Complete |
| ESS-07 | Salary component visibility | View current components and historical changes | Documents | P1 | Complete |
| ESS-08 | Adjustment request (employee) | Submit punch time correction with reason | Corrections | P1 | In Progress |
| ESS-09 | Document repository | Access HR documents (contracts, certificates) | Documents | P1 | Complete |
| ESS-10 | Self-clock interface (Phase 1) | Manual punch in/out with session status (future) | Portal | P2 | Future |

---

## 9. Admin Dashboard (14 Features)

| Feature ID | Feature Name | Description | Category | Priority | Status |
|---|---|---|---|---|---|
| DASH-01 | System overview dashboard | HR metrics and system health tabs | Overview | P0 | Complete |
| DASH-02 | Agent status monitoring | Online/offline status, last sync, queue depth | Monitoring | P0 | Complete |
| DASH-03 | Device status monitoring | Enabled/disabled, health indicators, error rates | Monitoring | P0 | Complete |
| DASH-04 | Employee CRUD | Create, read, update, delete employees | Management | P0 | Complete |
| DASH-05 | Department CRUD | Manage organizational structure | Management | P0 | Complete |
| DASH-06 | Shift CRUD | Define work schedule templates | Management | P0 | Complete |
| DASH-07 | Working pattern CRUD | Create and manage 7-day templates | Management | P0 | Complete |
| DASH-08 | Assignment rule CRUD | Create and manage auto-scheduling rules | Management | P1 | In Progress |
| DASH-09 | Leave type CRUD | Define leave categories and policies | Management | P0 | Complete |
| DASH-10 | Leave request approval | Review and approve/reject employee requests | Workflow | P0 | Complete |
| DASH-11 | Attendance adjustment approval | Review and approve correction requests | Workflow | P1 | In Progress |
| DASH-12 | Payroll calculation & review | Generate payroll, preview, and publish | Payroll | P0 | Complete |
| DASH-13 | Bulk import interface | Upload and import attendance data | Data | P1 | Complete |
| DASH-14 | Workspace settings | Configure defaults, policies, branding | Configuration | P1 | Complete |

---

## 10. Reporting & Analytics (8 Features)

| Feature ID | Feature Name | Description | Category | Priority | Status |
|---|---|---|---|---|---|
| REPORT-01 | Attendance summary reports | Daily, weekly, monthly attendance overview | Reports | P1 | Complete |
| REPORT-02 | Leave balance reports | Current balances by employee and leave type | Reports | P1 | Complete |
| REPORT-03 | Payroll summary reports | Total costs, per-employee breakdown | Reports | P1 | Complete |
| REPORT-04 | Overtime reports | OT hours by employee and department | Reports | P1 | Complete |
| REPORT-05 | Late/early violation reports | Punctuality analytics | Reports | P1 | Complete |
| REPORT-06 | Export to Excel | Download reports in standard format | Export | P1 | Complete |
| REPORT-07 | Export to PDF | Payslips and official documents as PDF | Export | P1 | Complete |
| REPORT-08 | Custom report builder | Allow admins to create ad-hoc reports (Phase 3) | Analytics | P2 | Future |

---

## Feature Dependencies

### Critical Path

```
Device Setup
    ↓
Edge Agent Registration
    ↓
Attendance Collection
    ↓
Attendance Pairing
    ↓
Shift Assignment
    ↓
Payroll Calculation
```

### Feature Dependencies Matrix

| Feature | Depends On | Blocking |
|---------|-----------|----------|
| Leave Request | Employee Profile, Leave Type, Leave Policy | Payroll Calculation |
| Payroll Calculation | Time Entries, Leave Requests, Salary Components | Payslip Generation |
| Shift Assignment | Working Patterns, Assignment Rules | Time Entry Matching |
| ESS Portal | Employee Profile, Authentication | Self-Clock Interface |

---

## MVP vs Future Features

### MVP (Phase 1) - COMPLETE

**Core workflow ready for production**:
- Attendance collection and pairing
- Basic payroll calculation
- Leave management
- Admin dashboard
- ESS portal (view-only)

### Phase 2 (In Progress)

**Enhanced HR capabilities**:
- Payroll incentives
- Assignment rule automation
- Attendance adjustments
- Working condition overlays

### Phase 3+ (Future)

**Advanced features**:
- Self-clock interface
- Mobile app
- Geofencing
- HRIS integrations
- Advanced analytics

---

## Feature Acceptance Criteria

### General Criteria (All Features)

- [ ] Code written with 100% test coverage for core logic
- [ ] Integration tests pass with fresh database
- [ ] No regression in existing features
- [ ] Documentation updated with examples
- [ ] API response time <500ms (P95)
- [ ] Audit trail maintained for sensitive operations

### Feature-Specific Criteria

**Attendance Features**:
- [ ] Raw logs never deleted (compliance)
- [ ] Duplicate detection 100% accurate
- [ ] Overnight shifts handled correctly
- [ ] Manual entries bypass pairing algorithm

**Leave Features**:
- [ ] Balance validation prevents overspend
- [ ] Approval workflow enforced
- [ ] Temporal policies applied correctly
- [ ] 3-tier hierarchy respected

**Payroll Features**:
- [ ] Segment calculations accurate to penny
- [ ] Change points detected automatically
- [ ] Historical policy lookup verified
- [ ] Payslips immutable after publication

---

## User Stories (20 Documented)

### Story 1: Edge Agent Setup
**As an** HR Admin
**I want to** register edge agents so that
**I can** collect attendance from biometric devices at remote sites

**Acceptance Criteria**:
- Agent registers with unique ID and token
- Admin approves registration before log ingestion
- Agent status shows online/offline
- Device list visible once agent connected

### Story 2: Automatic Pairing
**As a** System
**I want to** automatically pair punch logs into work sessions so that
**Manual** pairing is eliminated

**Acceptance Criteria**:
- First check-in paired with last check-out within 16 hours
- Overnight sessions detected and previous day lookback applied
- Generation metadata stored for traceability
- Pairing accuracy >95% without manual intervention

### Story 3: Leave Request
**As an** Employee
**I want to** submit leave requests through ESS portal so that
**I** don't need to email HR

**Acceptance Criteria**:
- Request form validates sufficient balance
- Status changes from pending to approved/rejected
- Balance deducted automatically on approval
- Approval notification sent to employee

### Story 4: Payroll Calculation
**As a** Payroll Manager
**I want to** calculate monthly payroll in one click so that
**The** process takes minutes instead of days

**Acceptance Criteria**:
- Calculation completes in <2 minutes per 100 employees
- Change points detected (mid-month salary changes)
- Segments calculated with exact prorating
- Payslip generated in draft status for review

### Story 5: Shift Assignment
**As an** HR Admin
**I want to** bulk assign employees to patterns using rules so that
**I** can schedule 100+ employees in seconds

**Acceptance Criteria**:
- Rules support department and tag-based conditions
- Priority-based evaluation (higher priority wins)
- Assignments traceable via trace hints
- Bulk execution completes in <5 seconds

### Story 6: Adjustment Approval
**As an** HR Admin
**I want to** approve attendance adjustments so that
**Errors** are fixed with proper audit trail

**Acceptance Criteria**:
- Employee submits new punch times with reason
- Admin reviews and approves/rejects
- Approved adjustments replace time_entry
- Original raw logs preserved for compliance

### Story 7: Overtime Calculation
**As a** Payroll Manager
**I want to** flexible overtime rates so that
**Different** departments can have different OT rules

**Acceptance Criteria**:
- OT rate configurable at workspace/dept/employee level
- Base components identified for rate calculation
- OT premium applied correctly
- Rate changes tracked temporally

### Story 8: Leave Balance View
**As an** Employee
**I want to** see real-time leave balance so that
**I** know how many days I can request

**Acceptance Criteria**:
- Balance displayed in ESS portal
- Breakdown by leave type shown
- Accrual logs visible (transparency)
- Carryover shown separately

### Story 9: Multi-Tenant Isolation
**As a** System
**I want to** enforce multi-tenant isolation so that
**Organizations** never see each other's data

**Acceptance Criteria**:
- All queries include workspace_id filter
- Cross-workspace queries impossible
- Integration tests verify isolation
- Zero security leaks in production

### Story 10: Device Monitoring
**As an** HR Admin
**I want to** see device status in real-time so that
**I** know which devices are offline

**Acceptance Criteria**:
- Device shows enabled/disabled, online/offline
- Last sync timestamp displayed
- Error count and recent errors shown
- Alerts trigger on device failure

### Story 11: Department Payroll Config
**As an** HR Manager
**I want to** override payroll settings per department so that
**Different** departments with different work standards are handled

**Acceptance Criteria**:
- Department config created with effective date
- Overrides workspace defaults
- Point-in-time lookup respects temporal versions
- Prorating uses correct department config

### Story 12: Policy Hierarchy
**As a** System
**I want to** apply 3-tier policy hierarchy so that
**Most** specific policy always wins

**Acceptance Criteria**:
- Employee config checked first (highest priority)
- Department config checked second
- Workspace config used as fallback
- Tie-breaking handled via effective_date DESC

### Story 13: Audit Trail Completeness
**As a** Compliance Officer
**I want to** audit all sensitive operations so that
**Regulatory** compliance is demonstrated

**Acceptance Criteria**:
- Raw logs never deleted (immutable)
- All adjustments traceable to user and reason
- Policy changes timestamped
- Payslip history retained indefinitely

### Story 14: Performance Requirement
**As a** System Administrator
**I want to** handle 100+ concurrent agents so that
**System** scales to large deployments

**Acceptance Criteria**:
- Agent uptime >99%
- Log sync latency <10 seconds
- API response time <500ms (P95)
- Horizontal scaling possible

### Story 15: Employee Self-Service
**As an** Employee
**I want to** access payslips online so that
**I** don't need to contact HR

**Acceptance Criteria**:
- Payslips accessible via ESS portal
- Download as PDF
- Search and filter by date
- View salary component breakdown

### Story 16: Configuration Flexibility
**As a** Workspace Owner
**I want to** customize brand color so that
**Dashboard** matches company identity

**Acceptance Criteria**:
- Color picker in workspace settings
- Applied globally across dashboard
- Stored per workspace
- Resets to defaults if removed

### Story 17: Data Import
**As an** HR Admin
**I want to** import historical attendance so that
**Legacy** systems can be migrated

**Acceptance Criteria**:
- Bulk import via Excel/CSV
- Validation prevents duplicate dates
- Import status tracked (pending → processing → complete)
- Errors logged for manual correction

### Story 18: Incentive Policy
**As a** HR Manager
**I want to** define incentive bonuses so that
**Diligent** employees are automatically rewarded

**Acceptance Criteria**:
- Policy defines criteria (max late, unpaid leave)
- Tiers with thresholds created
- First matching tier awarded (generosity)
- Payslip shows awarded incentive

### Story 19: Timezone Support
**As a** Multi-site Organization
**I want to** set timezone per workspace so that
**Attendance** calculations respect local time

**Acceptance Criteria**:
- Workspace timezone configured
- Punch times converted correctly
- Overnight shifts handled per timezone
- Reports show local times

### Story 20: Sidecar Architecture
**As a** System
**I want to** support multiple device brands so that
**Organizations** aren't locked to one vendor

**Acceptance Criteria**:
- ZKTeco worker implemented (complete)
- Hikvision worker pluggable (Phase 2)
- Suprema worker pluggable (Phase 2)
- Brand routing automatic

---

**Document Status**: Final for Phase 1 review
**Next Review**: Phase 2 planning (Q2 2026)
