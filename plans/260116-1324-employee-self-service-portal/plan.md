# Employee Self-Service (ESS) Portal - Implementation Plan

**Project**: LeeAttend HRM System
**Date**: 2026-01-16
**Status**: In Progress

---

## Overview

Add Employee Self-Service portal to LeeAttend, enabling employees to:
- View/edit personal profile (contact info, emergency contacts, bank details)
- Upload/download documents (ID, contracts, certificates)
- Access payslips
- View own attendance records
- Submit/track leave requests (enhance existing)

## Phases

| Phase | Description | Status | File |
|-------|-------------|--------|------|
| 1 | Database Schema Extensions | Pending | [phase-01-database-schema.md](./phase-01-database-schema.md) |
| 2 | Backend API Endpoints | Pending | [phase-02-backend-api.md](./phase-02-backend-api.md) |
| 3 | Frontend ESS Module | Pending | [phase-03-frontend-ess.md](./phase-03-frontend-ess.md) |
| 4 | Document Management | Pending | [phase-04-document-management.md](./phase-04-document-management.md) |
| 5 | Testing & Integration | Pending | [phase-05-testing.md](./phase-05-testing.md) |

## Key Dependencies

- Existing Employee model (apiserver/src/models.rs:246)
- User-Employee linkage (user_id field)
- Existing RBAC system (workspace members)
- Payslip model (already exists)
- Leave management system (already exists)

## Architecture Decision

**Employee Portal Access Pattern:**
- Employees access via existing login (User account)
- User linked to Employee via `employees.user_id`
- New "employee" role in RBAC for self-service access
- Endpoints validate employee owns requested data

## Success Criteria

1. Employees can view/edit own profile with audit trail
2. Document upload/download with proper access control
3. Payslip viewing restricted to own payslips
4. Attendance view for own records only
5. Leave request submission with balance display
6. Mobile-responsive UI following existing design system

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Data privacy (bank details) | Encrypt at-rest, audit all access |
| Unauthorized access | Strict ownership validation per request |
| File upload security | Validate types, size limits, virus scanning |

---

**Reports**: [./reports/](./reports/)
