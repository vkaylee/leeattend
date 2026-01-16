# Phase 2: Backend API Endpoints

**Priority**: High
**Status**: Pending
**Estimated Effort**: 4-6 hours

---

## Context Links

- Handlers pattern: `apiserver/src/handlers/`
- Existing employees handler: `apiserver/src/handlers/employees.rs`
- Auth middleware: `apiserver/src/handlers/auth.rs`

## Overview

Create ESS-specific API endpoints for employees to access their own data.

## Key Insights

- Need new handler: `apiserver/src/handlers/ess.rs` (Employee Self-Service)
- Authentication: Employee must have linked user_id
- Authorization: Only access own data (validate employee_id ownership)
- Reuse existing patterns (MemberContext, workspace isolation)

## Requirements

### Functional
- GET /api/ess/profile - Get own profile
- PUT /api/ess/profile - Update own profile
- GET/POST/DELETE /api/ess/emergency-contacts - Manage contacts
- GET /api/ess/documents - List own documents
- POST /api/ess/documents - Upload document
- GET /api/ess/documents/:id/download - Download document
- GET /api/ess/payslips - List own payslips
- GET /api/ess/payslips/:id - Get payslip detail
- GET /api/ess/attendance - View own attendance
- GET /api/ess/leave-balance - View own leave balance
- POST /api/ess/leave-requests - Submit leave request (existing, enhance)

### Non-Functional
- Rate limiting on file uploads
- File size limit (10MB default)
- Audit logging for all profile changes

## Architecture

```
ess.rs handler
    ├── get_my_profile()
    ├── update_my_profile()
    ├── get_emergency_contacts()
    ├── add_emergency_contact()
    ├── delete_emergency_contact()
    ├── get_my_documents()
    ├── upload_document()
    ├── download_document()
    ├── get_my_payslips()
    ├── get_my_attendance()
    └── get_my_leave_balance()
```

## Implementation Steps

### 1. Create EmployeeContext extractor
```rust
// Validate current user is linked to an employee
pub struct EmployeeContext {
    pub user_id: Uuid,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
}
```

### 2. Create ESS handler module
- New file: `apiserver/src/handlers/ess.rs`
- Add to `handlers/mod.rs`
- Register routes in main router

### 3. Profile Endpoints
- GET profile: Join employee + employee_profile
- PUT profile: Update profile fields, create if not exists

### 4. Emergency Contacts Endpoints
- CRUD operations with ownership validation

### 5. Documents Endpoints
- Upload: Store file locally (or S3 later), save metadata
- Download: Validate ownership, stream file
- List: Filter by employee_id

### 6. Payslips Endpoints
- List own payslips (filter by employee_id)
- Get single payslip (validate ownership)

### 7. Attendance Endpoints
- Reuse existing attendance query, filter by employee_id

## Todo List

- [ ] Create EmployeeContext extractor
- [ ] Create handlers/ess.rs module
- [ ] Implement profile endpoints
- [ ] Implement emergency contacts endpoints
- [ ] Implement documents endpoints (metadata only first)
- [ ] Implement payslips endpoints
- [ ] Implement attendance view endpoint
- [ ] Add routes to main router
- [ ] Add i18n keys for error messages

## Success Criteria

- All endpoints return correct data for authenticated employee
- Unauthorized access returns 403
- File upload works with size limits
- Audit log records profile changes

## Security Considerations

- Validate employee_id matches current user's linked employee
- File upload: validate MIME types, scan for malware (future)
- Rate limit document uploads (5 per hour per user)

## Related Code Files

- Create: `apiserver/src/handlers/ess.rs`
- Modify: `apiserver/src/handlers/mod.rs`
- Modify: `apiserver/src/main.rs` (add routes)
- Create: `apiserver/src/repositories/ess.rs`

## Next Steps

After API ready, proceed to Phase 3: Frontend ESS Module
