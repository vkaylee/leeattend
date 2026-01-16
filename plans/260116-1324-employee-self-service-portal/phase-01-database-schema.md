# Phase 1: Database Schema Extensions

**Priority**: High
**Status**: Pending
**Estimated Effort**: 2-3 hours

---

## Context Links

- Current models: `apiserver/src/models.rs`
- Migrations dir: `apiserver/migrations/`
- Employee model: lines 246-258

## Overview

Extend database to store employee personal details, emergency contacts, bank info, and documents.

## Key Insights

- Employee model currently minimal (name, ID number, dates)
- User model has profile fields but is separate from employee data
- Need separate tables for sensitive data (bank, emergency contacts)
- Documents need file storage + metadata

## Requirements

### Functional
- Store personal details (phone, address, DOB, gender, national ID)
- Emergency contacts (multiple per employee)
- Bank account details (for payroll)
- Document metadata (type, path, expiry)

### Non-Functional
- Encrypt sensitive fields (bank account, national ID)
- Audit all changes to personal data
- Soft-delete for documents

## Architecture

```
employees (existing)
    └── employee_profiles (1:1)
            ├── phone, address, dob, gender, national_id
            └── bank_name, bank_account, bank_branch
    └── employee_emergency_contacts (1:N)
            ├── name, relationship, phone
            └── is_primary
    └── employee_documents (1:N)
            ├── document_type, file_path, file_name
            ├── uploaded_at, expires_at
            └── is_active
```

## Implementation Steps

### Migration: employee_profiles
```sql
CREATE TABLE employee_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL UNIQUE REFERENCES employees(id) ON DELETE CASCADE,
    phone VARCHAR(20),
    address TEXT,
    date_of_birth DATE,
    gender VARCHAR(10),
    national_id VARCHAR(50),
    bank_name VARCHAR(100),
    bank_account_number VARCHAR(50),
    bank_branch VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Migration: employee_emergency_contacts
```sql
CREATE TABLE employee_emergency_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    contact_name VARCHAR(100) NOT NULL,
    relationship VARCHAR(50) NOT NULL,
    phone VARCHAR(20) NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Migration: employee_documents
```sql
CREATE TABLE employee_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES workspaces(id),
    document_type VARCHAR(50) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    expires_at DATE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    uploaded_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_employee_documents_employee ON employee_documents(employee_id);
```

## Todo List

- [ ] Create migration file for employee_profiles
- [ ] Create migration file for employee_emergency_contacts
- [ ] Create migration file for employee_documents
- [ ] Add Rust models in models.rs
- [ ] Run migrations and verify schema

## Success Criteria

- All tables created with proper constraints
- Foreign keys working correctly
- Indexes on frequently queried columns
- Models compile without errors

## Security Considerations

- Bank account number should be encrypted (consider pgcrypto)
- National ID should be encrypted
- Audit triggers for profile changes (future enhancement)

## Next Steps

After schema ready, proceed to Phase 2: Backend API Endpoints
