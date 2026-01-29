---
title: "OT Paper Form Digitization"
description: "Enable department heads/assistants to enter paper OT forms directly into system, eliminating HR bottleneck"
status: pending
priority: P2
effort: 3-4d
branch: main
tags: [ot, workflow, paper-forms, approvals]
created: 2026-01-28
---

# OT Paper Form Digitization

## Overview

Replace manual paper-to-HR workflow with direct digital entry by department heads/assistants. Paper form signature = legal consent. No worker notification needed (no SMS/email).

## Business Context

**Current Flow**: Worker fills paper OT form → Dept head signs → HR enters into system
**Target Flow**: Worker fills paper OT form → Dept head/assistant enters directly → Same approval workflow

## Phases

| # | Phase | Status | Effort | File |
|---|-------|--------|--------|------|
| 1 | Database Schema | [ ] pending | 1h | [phase-01](./phase-01-database-schema.md) |
| 2 | Backend API | [ ] pending | 4h | [phase-02](./phase-02-backend-api.md) |
| 3 | Frontend Entry Form | [ ] pending | 4h | [phase-03](./phase-03-frontend-entry-form.md) |
| 4 | Approval Integration | [ ] pending | 3h | [phase-04](./phase-04-approval-integration.md) |
| 5 | Testing | [ ] pending | 4h | [phase-05](./phase-05-testing.md) |

## Key Design Decisions

1. **Reuse `claim_ot` endpoint** with role-based behavior (skip `pending_employee` → direct to `pending_manager`)
2. **New status: `pending_manager`** (entered by admin, not by employee)
3. **Photo upload optional** (local filesystem initially, S3 later if needed)
4. **Department scoping** via existing employee-department relationship
5. **No duplicate prevention** (allow multiple entries, HR can reject duplicates)

## Success Criteria

- [ ] Dept head can enter OT claim for any employee in their department
- [ ] Entered claims appear in OT Approvals page with "Paper Form" badge
- [ ] Approval workflow unchanged (approve/reject works same as self-service)
- [ ] Audit trail shows who entered the claim and when
- [ ] Photo attachment optional but functional

## Dependencies

- Existing `attendance_daily_summaries` table
- Existing `AttendanceSummaryService` service
- Existing `OtApprovalsPage` component
- Existing `EmployeeSearch` component

## Unresolved Questions

1. Should assistant be able to approve claims they entered, or only dept head?
2. Photo upload: required or optional? → **Decided: Optional**
3. File storage: AWS S3 or local filesystem? → **Start with local, migrate later**
4. Should system prevent duplicate entries for same employee/date?
5. Paper form retention policy after digitization?
6. Should HR have read-only access to digitized forms?
7. Can department heads delegate entry permission to specific assistants?
