# Test Plan: RBAC Integration Tests

## Scope
Comprehensive integration testing of the new Role-Based Access Control (RBAC) features in LeeAttend, focusing on:
1. Scope-based permissions (workspaces vs global)
2. Role hierarchy and inheritance
3. Temporary extra roles
4. Individual permission overrides
5. Audit log generation for permission changes

## Objectives
- Ensure all 60+ permissions are correctly enforced across the API.
- Verify that users can only access resources within their authorized scopes.
- Validate that role inheritance grants appropriate permissions.
- Confirm temporary roles grant access and expire correctly.
- Test that individual permission overrides (grant/deny) work as expected and override role-based permissions.
- Ensure the audit log correctly records all modifications to roles, permissions, and overrides.

## Test Approach
- **Integration Tests**: Focus on API endpoints that require specific permissions.
- **Data Scenarios**: Create test fixtures with various user roles, scopes, temporary assignments, and overrides.
- **Asserting Access**: Verify API responses (200 OK vs 403 Forbidden) based on the configured permissions.
- **Database Verification**: Check the database state (e.g., audit log entries) after permission-modifying actions.

## Acceptance Criteria
- [ ] All API endpoints enforce the correct permissions based on the new RBAC model.
- [ ] Users cannot access resources outside their assigned scopes.
- [ ] Role inheritance correctly grants higher-level permissions.
- [ ] Temporary roles function as expected within their valid timeframe.
- [ ] Individual permission overrides override role-based permissions.
- [ ] All permission changes generate correct audit log entries.
- [ ] The full integration test suite passes without any failures.

## Checkpoints
- CP1: Setup Test Environment and Fixtures
- CP2: Test Global vs Workspace Scopes
- CP3: Test Role Inheritance
- CP4: Test Temporary Extra Roles
- CP5: Test Individual Permission Overrides
- CP6: Test Audit Log Generation
