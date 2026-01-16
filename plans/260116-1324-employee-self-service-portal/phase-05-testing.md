# Phase 5: Testing & Integration

**Priority**: High
**Status**: Pending
**Estimated Effort**: 4-6 hours

---

## Context Links

- Rust tests: `apiserver/tests/`
- TestFactory: `apiserver/tests/common/`
- Playwright: `webdashboard/tests/`
- Vitest: `webdashboard/src/**/__tests__/`

## Overview

Comprehensive testing for ESS features covering API endpoints, UI components, and E2E flows.

## Key Insights

- Follow existing test patterns (TestFactory for Rust)
- Playwright for E2E (existing setup)
- Vitest for component tests
- Real database tests (no mocks for integration)

## Requirements

### Functional
- All ESS endpoints tested
- Profile CRUD operations verified
- Document upload/download tested
- Authorization checks verified
- UI components rendered correctly

### Non-Functional
- 80%+ code coverage for new code
- Tests run in parallel
- Isolated test databases
- Cleanup after each test

## Test Cases

### API Tests (Rust)

#### Profile Tests
```rust
#[tokio::test]
async fn test_get_own_profile() { ... }

#[tokio::test]
async fn test_update_profile_success() { ... }

#[tokio::test]
async fn test_cannot_access_other_profile() { ... }
```

#### Emergency Contacts Tests
```rust
#[tokio::test]
async fn test_add_emergency_contact() { ... }

#[tokio::test]
async fn test_list_emergency_contacts() { ... }

#[tokio::test]
async fn test_delete_emergency_contact() { ... }
```

#### Document Tests
```rust
#[tokio::test]
async fn test_upload_document() { ... }

#[tokio::test]
async fn test_download_own_document() { ... }

#[tokio::test]
async fn test_cannot_download_other_document() { ... }

#[tokio::test]
async fn test_reject_invalid_file_type() { ... }
```

#### Payslip Tests
```rust
#[tokio::test]
async fn test_list_own_payslips() { ... }

#[tokio::test]
async fn test_cannot_access_other_payslips() { ... }
```

### E2E Tests (Playwright)

#### Profile Flow
```typescript
test('employee can view and update profile', async ({ page }) => {
    // Login as employee
    // Navigate to /ess/profile
    // Verify profile data displayed
    // Edit phone number
    // Save and verify update
});
```

#### Document Upload Flow
```typescript
test('employee can upload document', async ({ page }) => {
    // Login
    // Navigate to /ess/documents
    // Upload a PDF file
    // Verify document appears in list
    // Download and verify
});
```

#### Payslip View Flow
```typescript
test('employee can view payslips', async ({ page }) => {
    // Login
    // Navigate to /ess/payslips
    // Verify payslips listed
    // Click to view detail
    // Verify earnings displayed
});
```

### Component Tests (Vitest)

```typescript
// ProfileForm.test.tsx
describe('ProfileForm', () => {
    it('renders all fields', () => { ... });
    it('validates phone format', () => { ... });
    it('submits form data', () => { ... });
});

// DocumentUploader.test.tsx
describe('DocumentUploader', () => {
    it('accepts valid file types', () => { ... });
    it('rejects invalid file types', () => { ... });
    it('shows upload progress', () => { ... });
});
```

## Implementation Steps

1. Create Rust integration test file: `apiserver/tests/ess_tests.rs`
2. Extend TestFactory for ESS data setup
3. Create Playwright spec: `webdashboard/tests/ess.spec.ts`
4. Create component test files
5. Run full test suite
6. Fix any failures
7. Verify coverage

## Todo List

- [ ] Create apiserver/tests/ess_tests.rs
- [ ] Add ESS builders to TestFactory
- [ ] Write profile endpoint tests
- [ ] Write document endpoint tests
- [ ] Write authorization denial tests
- [ ] Create webdashboard/tests/ess.spec.ts
- [ ] Write E2E profile flow test
- [ ] Write E2E document upload test
- [ ] Create component unit tests
- [ ] Run full test suite (./run-tests.sh)
- [ ] Verify all tests pass

## Success Criteria

- All new endpoints have test coverage
- Authorization checks verified with denial tests
- E2E flows complete successfully
- No regressions in existing tests
- Coverage report shows 80%+ for new code

## Security Testing

- Test unauthorized access returns 403
- Test file type validation
- Test file size limits
- Test cross-employee access denied

## Related Code Files

- Create: `apiserver/tests/ess_tests.rs`
- Modify: `apiserver/tests/common/factory.rs`
- Create: `webdashboard/tests/ess.spec.ts`
- Create: `webdashboard/src/pages/ess/__tests__/`
- Create: `webdashboard/src/components/ess/__tests__/`

## Next Steps

After all tests pass, update documentation and deploy.
