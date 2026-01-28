# LeeAttend Code Standards & Guidelines

## Code Organization & Structure

### File Naming Conventions

**Rust**:
- Use `snake_case.rs` for module files
- One primary entity per file (domain.rs, user.rs)
- Handler/Service/Repository pattern naming:
  - `src/handlers/domain.rs`
  - `src/services/domain_service.rs`
  - `src/repositories/domain_repository.rs`
- Test files: `*_test.rs` or nested in `#[cfg(test)]` modules
- Migration files: `YYYYMMDDHHMMSS_description.sql`

**TypeScript/React**:
- Page components: `PascalCase.tsx` (e.g., EmployeesPage.tsx)
- UI components: `PascalCase.tsx` in domain subdirectories
- Utilities: `camelCase.ts` (e.g., formatDate.ts)
- Hooks: `use-domain.ts` (kebab-case with use- prefix)
- API clients: `camelCase.ts` by domain (employees.ts, payroll.ts)
- Test files: `*.test.ts`, `*.test.tsx`, `*.spec.ts`

**Python**:
- Worker modules: `snake_case.py`
- Main entry: `main.py`
- Test files: `test_*.py`
- Classes: `PascalCase`
- Functions/variables: `snake_case`

### File Size Limits

**Target**: Maximum 200 lines per file

**Application**:
- Rust handlers: Keep focused (single domain per file)
- Rust services: Business logic only, delegate to repos
- Rust repositories: Data access layer
- TypeScript components: One component per file
- TypeScript utils: Group related utilities

**Exceptions**:
- Complex domain modules may exceed 200 lines with justification
- Configuration files have no size limit
- Migration files determined by schema scope

**Refactoring Strategy**:
- When file approaches 150 lines, plan extraction
- Split on logical boundaries (separate concerns)
- Use composition over monolithic structures
- Create focused utility modules

## Rust Code Style

### Language Standards

**Edition**: Rust 2021 (apiserver), Rust 2024 (agent-main)

**Linting**: Cargo Clippy (mandatory before commit)

Run: `cargo clippy --all-targets --all-features -- -D warnings`

**Formatting**: rustfmt (automatic via pre-commit hooks)

Run: `cargo fmt`

### Code Structure

**Import Organization**:
```rust
use std::collections::HashMap;

use axum::response::IntoResponse;
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::models::{Employee, Workspace};
use crate::repositories::EmployeeRepository;
```

Order: `std` → external crates → internal modules

**Async Functions**:
- Use `#[tokio::main]` for entry point
- Mark async functions with `.await`
- Use `tokio::spawn` for concurrent tasks
- Handle `Result` types explicitly

**Error Handling**:
```rust
match operation_result {
    Ok(value) => { /* process */ },
    Err(e) => {
        tracing::error!("Operation failed: {}", e);
        return Err(e.into());
    }
}
```

Use `?` operator for error propagation in Result contexts

**Traits & Implementations**:
```rust
#[async_trait]
pub trait EmployeeRepository {
    async fn get_by_id(&self, id: Uuid) -> Result<Employee>;
}

pub struct PostgresEmployeeRepository {
    pool: PgPool,
}

#[async_trait]
impl EmployeeRepository for PostgresEmployeeRepository {
    async fn get_by_id(&self, id: Uuid) -> Result<Employee> {
        // implementation
    }
}
```

### Naming Conventions

**Variables/Functions**: `snake_case`
- `employee_id`, `get_employee`, `parse_timestamp`

**Types/Structs**: `PascalCase`
- `Employee`, `PayrollConfig`, `TimeEntry`

**Constants**: `SCREAMING_SNAKE_CASE`
- `MAX_POOL_SIZE = 10`, `RETRY_MAX_ATTEMPTS = 10`

**Lifetimes**: `'a`, `'b` (single quotes)

**Generics**: `T`, `K`, `V` (single uppercase letters)

### Function Guidelines

**Max Function Size**: 30 lines (Rule of 30)

Rationale: Functions exceeding 30 lines indicate multiple responsibilities

**Function Signature**:
```rust
pub async fn process_employee(
    repo: &EmployeeRepository,
    id: Uuid,
    workspace_id: Uuid,
) -> Result<Employee> {
    // implementation
}
```

- Explicit parameter types
- Return Result for fallible operations
- Mark async operations explicitly

**Documentation**:
```rust
/// Processes employee attendance for payroll.
///
/// # Arguments
/// * `employee_id` - Employee UUID
/// * `period_start` - Period start date
/// * `period_end` - Period end date
///
/// # Returns
/// Payslip with calculated segments
///
/// # Errors
/// Returns error if employee not found or calculation fails
pub async fn calculate_payslip(
    employee_id: Uuid,
    period_start: Date,
    period_end: Date,
) -> Result<Payslip> {
    // implementation
}
```

### Testing Patterns

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_success() {
        let input = TestData::new();
        let result = operation(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_error() {
        let input = InvalidData::new();
        let result = operation(input);
        assert!(result.is_err());
    }
}
```

**Integration Tests**:
```rust
#[tokio::test]
async fn test_database_operation() {
    let db = TestDatabase::new().await;
    let repo = PostgresEmployeeRepository::new(db.pool());

    let employee = repo.get_by_id(Uuid::new_v4()).await;
    assert!(employee.is_ok());
}
```

Requirements:
- Use `TestDatabase::new()` for unique databases per test
- Auto-apply migrations via `sqlx::migrate!()`
- Signal-safe cleanup on test exit
- Match `--test-threads` to worker pool

### Security Best Practices

**SQL Injection Prevention**:
```rust
// ✓ CORRECT: Parameterized query
sqlx::query_as::<_, Employee>(
    "SELECT * FROM employees WHERE id = $1 AND workspace_id = $2"
)
.bind(id)
.bind(workspace_id)
.fetch_one(&pool)
.await
```

Never use string concatenation for SQL!

**Multi-Tenancy Enforcement**:
```rust
// ✓ CORRECT: Always filter by workspace_id
let employees = sqlx::query_as::<_, Employee>(
    "SELECT * FROM employees WHERE workspace_id = $1"
)
.bind(workspace_id)
.fetch_all(&pool)
.await?;
```

No workspace_id filter = security bug!

**Password Handling**:
```rust
use bcrypt::{hash, verify};

// Hash on create
let hashed = hash(password, bcrypt::DEFAULT_COST)?;

// Verify on login
let matches = verify(provided_password, &stored_hash)?;
```

Never store plaintext passwords!

## TypeScript/React Code Style

### Language Standards

**Version**: TypeScript 5, React 19

**Linting**: ESLint 9 (mandatory before commit)

Run: `npm run lint` (or `npm run lint:fix` for auto-fixes)

**Type Checking**: Strict mode enabled

```json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true
  }
}
```

### Component Structure

**Functional Components**:
```typescript
interface EmployeeListProps {
  workspaceId: string;
  onSelect?: (employee: Employee) => void;
}

export function EmployeeList({ workspaceId, onSelect }: EmployeeListProps) {
  const { employees, isLoading } = useEmployees(workspaceId);

  if (isLoading) return <LoadingSpinner />;

  return (
    <div className="space-y-2">
      {employees.map((emp) => (
        <EmployeeCard key={emp.id} employee={emp} onSelect={onSelect} />
      ))}
    </div>
  );
}
```

Requirements:
- Use `interface` for component props
- Destructure props in function signature
- Return JSX/TSX

**Hooks**:
```typescript
export function useEmployees(workspaceId: string) {
  const query = useQuery({
    queryKey: ["employees", workspaceId],
    queryFn: () => apiClient.employees.list(workspaceId),
  });

  return {
    employees: query.data ?? [],
    isLoading: query.isLoading,
    error: query.error,
  };
}
```

Naming: `use-domain.ts` file with `useCapitalCase` function

**API Clients**:
```typescript
// lib/api/employees.ts
export const employeesApi = {
  list: async (workspaceId: string) => {
    const response = await apiFetch(
      `/workspaces/${workspaceId}/employees`,
      { method: "GET" }
    );
    return response.json() as Promise<Employee[]>;
  },

  create: async (workspaceId: string, data: CreateEmployeeDTO) => {
    return apiFetch(
      `/workspaces/${workspaceId}/employees`,
      { method: "POST", body: JSON.stringify(data) }
    ).then((res) => res.json());
  },
};
```

Organization: Domain-specific modules (employees.ts, payroll.ts, etc.)

### Naming Conventions

**Components**: `PascalCase`
- `EmployeeList`, `PayrollTable`, `LeaveRequestForm`

**Functions/Variables**: `camelCase`
- `getEmployee`, `calculateTotal`, `formatDate`

**Constants**: `SCREAMING_SNAKE_CASE`
- `API_TIMEOUT`, `DEFAULT_PAGE_SIZE`

**CSS Classes** (Tailwind): `kebab-case`
- `bg-blue-500`, `flex`, `gap-4`

**Props Interface**: `ComponentNameProps`
- `EmployeeListProps`, `PayslipViewerProps`

### Testing Patterns

**Unit Tests** (Vitest):
```typescript
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { EmployeeList } from "./EmployeeList";

describe("EmployeeList", () => {
  it("renders employee names", () => {
    const employees = [
      { id: "1", firstName: "John", lastName: "Doe" },
      { id: "2", firstName: "Jane", lastName: "Smith" },
    ];

    render(<EmployeeList employees={employees} />);

    expect(screen.getByText("John Doe")).toBeInTheDocument();
    expect(screen.getByText("Jane Smith")).toBeInTheDocument();
  });

  it("calls onSelect when employee clicked", async () => {
    const onSelect = vi.fn();
    const employees = [{ id: "1", firstName: "John", lastName: "Doe" }];

    const { user } = render(
      <EmployeeList employees={employees} onSelect={onSelect} />
    );

    await user.click(screen.getByText("John Doe"));
    expect(onSelect).toHaveBeenCalledWith(employees[0]);
  });
});
```

Requirements:
- Use `vitest` for unit tests
- Mock external dependencies (API calls, contexts)
- Timeout: 10s

**E2E Tests** (Playwright):
```typescript
import { test, expect } from "@playwright/test";
import { LoginPage } from "./pom/login-page";
import { EmployeesPage } from "./pom/employees-page";

test.describe("Employee Management", () => {
  let loginPage: LoginPage;
  let employeesPage: EmployeesPage;

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page);
    await loginPage.goto();
    await loginPage.login("user@example.com", "password");

    employeesPage = new EmployeesPage(page);
    await employeesPage.goto();
  });

  test("should list all employees", async () => {
    const employees = await employeesPage.getEmployees();
    expect(employees.length).toBeGreaterThan(0);
  });

  test("should create new employee", async () => {
    await employeesPage.clickAddButton();
    await employeesPage.fillForm({
      firstName: "Test",
      lastName: "User",
      email: "test@example.com",
    });
    await employeesPage.clickSaveButton();

    const success = await employeesPage.isSuccessMessageVisible();
    expect(success).toBe(true);
  });
});
```

Requirements:
- Use Page Object Model (POM) pattern
- Locator priority: `data-testid` > semantic selectors > roles
- Custom fixtures for auth (auto-login)
- Timeout: 15s global, 10s API

### React Patterns

**State Management**:
- Context API: Workspace, Auth state
- React Query: Server state (employees, attendance, etc.)
- Local state: Component UI (forms, modals)

**Query Hook Pattern**:
```typescript
const query = useQuery({
  queryKey: ["employees", workspaceId],
  queryFn: () => api.employees.list(workspaceId),
  staleTime: 5 * 60 * 1000, // 5 minutes
  retry: 1,
});
```

**Mutation Hook Pattern**:
```typescript
const mutation = useMutation({
  mutationFn: (data: CreateEmployeeDTO) =>
    api.employees.create(workspaceId, data),
  onSuccess: () => {
    queryClient.invalidateQueries({
      queryKey: ["employees", workspaceId],
    });
  },
});
```

**Custom Hook Pattern**:
```typescript
export function useEmployees(workspaceId: string) {
  const query = useQuery({
    queryKey: ["employees", workspaceId],
    queryFn: () => api.employees.list(workspaceId),
  });

  const createMutation = useMutation({
    mutationFn: (data: CreateEmployeeDTO) =>
      api.employees.create(workspaceId, data),
  });

  return {
    employees: query.data ?? [],
    isLoading: query.isLoading,
    create: createMutation.mutate,
    isCreating: createMutation.isPending,
  };
}
```

### Form Validation

**Runtime Validation**:
```typescript
interface CreateEmployeeDTO {
  firstName: string;
  lastName: string;
  email: string;
  departmentId: string;
}

function validateEmployee(data: unknown): data is CreateEmployeeDTO {
  if (typeof data !== "object" || data === null) return false;
  const obj = data as Record<string, unknown>;
  return (
    typeof obj.firstName === "string" &&
    typeof obj.lastName === "string" &&
    typeof obj.email === "string" &&
    typeof obj.departmentId === "string"
  );
}
```

## Python Code Style

### Language Standards

**Version**: Python 3.9+

**Linting**: No enforced linter (prefer readability)

**Testing**: pytest with asyncio support

### File Structure

**Entry Point**:
```python
# main.py
import asyncio
import os
from zk_worker import ZKWorker

async def main():
    port = int(os.environ.get("PORT", 5555))
    worker = ZKWorker(port)
    await worker.start()

if __name__ == "__main__":
    asyncio.run(main())
```

**Worker Implementation**:
```python
# zk_worker.py
import json
from typing import Any, Dict

class ZKWorker:
    def __init__(self, port: int):
        self.port = port
        self.device = None

    async def handle_ping(self) -> bool:
        """Heartbeat check"""
        return True

    async def handle_device_connect(self, ip: str, port: int) -> Dict[str, Any]:
        """Connect to device"""
        try:
            # Device connection logic
            return {"status": "connected", "device_info": {...}}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    async def handle_device_get_logs(self) -> list:
        """Fetch logs from device"""
        # Implementation
        return logs
```

### Naming Conventions

**Classes**: `PascalCase`
```python
class ZKWorker:
    pass

class DeviceConnectionPool:
    pass
```

**Functions/Methods**: `snake_case`
```python
def connect_device(ip: str, port: int):
    pass

async def fetch_attendance_logs():
    pass
```

**Constants**: `SCREAMING_SNAKE_CASE`
```python
DEFAULT_TIMEOUT = 300
MAX_RETRY_ATTEMPTS = 10
DEVICE_PORT = 5005
```

### Testing Patterns

**Unit Tests**:
```python
import pytest
from zk_worker import ZKWorker

@pytest.fixture
def worker():
    return ZKWorker(port=5555)

@pytest.mark.asyncio
async def test_ping_success(worker):
    result = await worker.handle_ping()
    assert result is True

@pytest.mark.asyncio
async def test_device_connect_error(worker):
    result = await worker.handle_device_connect("invalid.ip", 9999)
    assert result["status"] == "error"
```

Requirements:
- Use `pytest` with `asyncio` plugin
- Mark async tests with `@pytest.mark.asyncio`
- Use fixtures for test setup
- Timeout: per test configuration

## Commit Message Guidelines

### Format (Conventional Commits)

```
type(scope): subject

body

footer
```

**Type**: One of
- `feat` - New feature
- `fix` - Bug fix
- `refactor` - Code refactoring (no feature/fix)
- `test` - Test additions or modifications
- `docs` - Documentation changes
- `style` - Formatting, not code logic
- `chore` - Build, dependencies, tooling

**Scope**: Component affected
- `api` - API server changes
- `agent` - Edge agent changes
- `dashboard` - Web dashboard changes
- `db` - Database migrations
- `ci` - CI/CD changes

**Subject**: Imperative mood, lowercase
- "add employee pairing logic" ✓
- "Added employee pairing logic" ✗
- "Fixes bug in pairing" ✗

**Body**: Detailed explanation (optional)
- Why change was made
- What problem it solves
- Alternative approaches considered

**Footer**: Issue references (optional)
```
Fixes #123
Related-to #456
```

### Examples

```
feat(api): add overtime multiplier override at employee level

Allows managers to customize overtime rates per employee.
Previously only supported at department/workspace level.

Implements 3-tier policy resolution hierarchy.

Fixes #42
```

```
fix(agent): prevent concurrent device polling

Semaphore-based scheduling ensures fair resource allocation.
Prevents device starvation under high load.
```

```
test(dashboard): add E2E tests for employee creation

Page Object Model pattern for maintainability.
Covers happy path and validation errors.
```

## Pre-Commit Checklist

Before committing any code:

**Rust Components**:
- [ ] Run `cargo fmt` (formatting)
- [ ] Run `cargo clippy` (linting)
- [ ] Run `cargo test` (unit tests)
- [ ] Run integration tests via `./run-tests.sh --api`

**TypeScript/React**:
- [ ] Run `npm run lint` (linting)
- [ ] Run `npm run typecheck` (type checking)
- [ ] Run `npm run test` (unit tests)
- [ ] Run `npm run test:e2e` (E2E tests, optional)

**Python**:
- [ ] Run `pytest` (unit tests)
- [ ] Check syntax with `python -m py_compile`

**All**:
- [ ] No confidential data (.env, secrets, credentials)
- [ ] Commit message follows Conventional Commits format
- [ ] No console.log, dbg!, print statements
- [ ] Changes are focused (one feature/fix per commit)

## Pull Request Guidelines

### PR Title Format

Same as commit subject (conventional commits):
```
feat(api): add overtime multiplier override
fix(dashboard): correct date formatting in payslip
refactor(agent): simplify retry logic
```

### PR Description Template

```markdown
## Summary
Brief description of changes (2-3 sentences).

## Changes Made
- Bullet point per change
- Reference related components
- Note any breaking changes

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] E2E tests pass (if applicable)
- [ ] Manual testing completed

## Screenshots (if UI changes)
Include before/after screenshots for UI changes.

## Breaking Changes
None / Describe any breaking changes and migration path.

## Related Issues
Fixes #123, Related to #456
```

## Documentation Standards

### Code Comments

**What to Document**:
- Non-obvious logic (why, not what)
- Algorithm choices and trade-offs
- Public API contracts
- Complex business rules

**What NOT to Document**:
- Self-evident code (good naming sufficient)
- Obvious side effects
- Language-specific details

**Comment Style**:
```rust
// Single-line comment for brief notes
// (prefer code clarity over comments)

/* Multi-line comment for complex explanations
 * spanning multiple lines
 */

/// Doc comment for public API (Rust)
/// Explains what function does, not how
```

### README for Components

Each component should have minimal README:
- Purpose and scope
- Quick start (build, run, test)
- Key files to understand
- Integration points

## Standards Compliance

### Testing Coverage

Minimum requirements:
- Critical business logic: 80%+
- API handlers: 70%+
- Utilities: 60%+

Run coverage: `cargo tarpaulin --out Html` (Rust)

### Performance Benchmarks

Key metrics (from architecture docs):
- Batch writer: 100 records or 200ms (agent)
- Device polling: Fair scheduling via semaphore
- API response time: < 500ms (median)
- WebSocket latency: < 100ms (agent to server)

### Security Checklist

Before production deploy:
- [ ] Multi-tenancy: All queries filtered by workspace_id
- [ ] Authentication: JWT validation on protected routes
- [ ] Authorization: Role-based checks enforced
- [ ] SQL Injection: Parameterized queries only
- [ ] Secrets: No hardcoded keys or credentials
- [ ] Debug Features: Disabled in production

## Useful Commands Reference

**Rust**:
```bash
cargo fmt                          # Format code
cargo clippy                       # Lint with suggestions
cargo test                         # Run tests
cargo build --release             # Optimized build
cargo sqlx prepare               # Update SQLx metadata
```

**TypeScript**:
```bash
npm run lint                       # Check linting
npm run lint:fix                   # Auto-fix linting
npm run typecheck                  # Type checking
npm run test                       # Run unit tests
npm run test:e2e                   # Run E2E tests
npm run build                      # Production build
```

**Python**:
```bash
pytest                             # Run tests
pytest -v                          # Verbose output
python -m py_compile script.py     # Syntax check
```

**Project**:
```bash
./run-tests.sh --all              # Full test suite
./setup-test-env.sh               # Test database
```

## Standards Evolution

Documentation is living. When updating standards:
1. Update this file
2. Document rationale (why change)
3. Communicate to team
4. Update existing code in phases
5. Enforce on new code immediately

Current version: 1.0 (as of 2026-01-28)
