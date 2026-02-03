# Scout Report: Device CRUD Testing Setup

**Objective:** Scout webdashboard testing setup, device-related tests, and UI components needing data-testid attributes for E2E device CRUD testing.

**Date:** 2026-02-01 15:48

---

## 1. Testing Setup for webdashboard

### Playwright Configuration
- **Location:** `/home/elt1541/leeattend/webdashboard/playwright.config.ts`
- **Status:** Fully configured with worker orchestration
- **Test Directory:** `/home/elt1541/leeattend/webdashboard/tests/`
- **Framework:** Playwright E2E testing
- **Timeout:** 60s per test, 5s expect timeout
- **Workers:** Parallel execution (1 worker on CI, multiple on local)
- **Reporter:** HTML + line

### Worker Orchestration System
- **Location:** `/home/elt1541/leeattend/webdashboard/tests/worker-orchestrator.ts`
- **Feature:** Advanced isolated environment setup per worker
- **Database Isolation:** Each worker gets unique DB (test_database_e2e_{workerIndex})
- **Port Management:** Dynamic port allocation via port files
- **Backend Spawn:** Rust apiserver binary in debug mode
- **Frontend Spawn:** npm dev server with Vite

### Page Object Model (POM)
**Existing POM files:**
- `/home/elt1541/leeattend/webdashboard/tests/pom/BasePage.ts` - Base class with goto, verifyTitle
- `/home/elt1541/leeattend/webdashboard/tests/pom/LoginPage.ts` - Login flows with data-testid usage
- `/home/elt1541/leeattend/webdashboard/tests/pom/Sidebar.ts` - Navigation
- `/home/elt1541/leeattend/webdashboard/tests/pom/EmployeesPage.ts` - Employee management

**Pattern:** All POMs extend BasePage, use data-testid for element selection

### Existing E2E Tests
Located in `/home/elt1541/leeattend/webdashboard/tests/`:
- login-success.spec.ts
- members-ui.spec.ts
- attendance-logs.spec.ts
- employee-pagination.spec.ts
- sidebar-navigation.spec.ts
- leave-policies.spec.ts
- register-flow.spec.ts
- 12+ other spec files

**Test Pattern Example (login-success.spec.ts):**
```typescript
import { expect } from '@playwright/test';
import { test } from './worker-orchestrator';

test.describe('Login Success Flow', () => {
  test('should successfully login', async ({ loginPage }) => {
    await loginPage.goto();
    await loginPage.login('admin@leeattend.com', 'admin123');
    await expect(loginPage['page']).toHaveURL(/\/$/);
    await loginPage.verifyTitle('dashboard-title');
  });
});
```

### package.json Test Scripts
- **Vitest:** Unit testing (not Playwright)
- **Playwright:** E2E testing framework installed
- **Test Scripts:**
  - `test` - Vitest run
  - `test:coverage` - Vitest with coverage
  - No explicit Playwright script (run via `npx playwright test`)

---

## 2. Existing Device-Related Tests

### Backend API Tests
**Location:** `/home/elt1541/leeattend/apiserver/tests/device_management.rs`

**Test Coverage:**
- `test_create_and_get_device` - POST /api/devices, GET /api/devices
- `test_delete_device` - DELETE /api/devices/{id}
- `test_update_device` - PUT /api/devices/{id}
- `test_delete_device_not_found` - 404 handling

**API Endpoints:**
- POST `/api/devices` - Create device (201)
- GET `/api/devices` - List all devices (200)
- PUT `/api/devices/{id}` - Update device (200)
- DELETE `/api/devices/{id}` - Delete device (204/404)

**Handler Implementation:** `/home/elt1541/leeattend/apiserver/src/handlers/agents.rs`
- `create_device` (line 247)
- `update_device` (line 276)
- `delete_device` (line 321)

**Device Model Fields:**
- id (UUID)
- agent_id (String)
- name (String)
- ip (String)
- port (i32)
- workspace_id (UUID)
- brand (String: "zkteco" | "mock")
- is_enabled (bool)

### Frontend API Client
**Location:** `/home/elt1541/leeattend/webdashboard/src/lib/api/agents.ts`

**Functions:**
- `fetchDevices()` - GET /api/devices
- `createDevice(device)` - POST /api/devices
- `updateDevice(id, device)` - PUT /api/devices/{id}
- `deleteDevice(id)` - DELETE /api/devices/{id}

**React Hook:** `/home/elt1541/leeattend/webdashboard/src/hooks/use-devices.ts`
- React Query integration
- Mutations with automatic cache invalidation
- Returns: devices[], isLoading, createDevice, updateDevice, deleteDevice

---

## 3. UI Components That Need data-testid

### ConnectivityTopology.tsx
**Location:** `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/ConnectivityTopology.tsx`

**Missing data-testid attributes:**

**Header Actions:**
- Line 155-160: Expand All button
- Line 162-166: Collapse All button
- Line 170-180: Include Inactive toggle
- Line 181-186: Add Agent button (has Plus icon)

**Add Agent Form (line 191-216):**
- Line 200: Agent ID input
- Line 204: Agent Name input
- Line 208: Agent Token input
- Line 212: Cancel button
- Line 213: Save button (submit)

**Agent List:**
- Line 245-260: AgentAccordionCard instances (need container data-testid)

**Unassigned Devices Section (line 264-285):**
- Line 272: Unassigned devices container
- Line 273-282: DeviceListItem instances

**Edit Device Modal (line 297-382):**
- Line 313: Device name input
- Line 321: Agent combobox
- Line 331: Brand combobox
- Line 348: IP input
- Line 358: Port input
- Line 367: Cancel button
- Line 372: Update button

### AgentAccordionCard.tsx
**Location:** `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/AgentAccordionCard.tsx`

**Missing data-testid attributes:**

**Header Row (line 74-149):**
- Line 74-149: Accordion header (clickable)
- Line 79-81: Chevron expand/collapse
- Line 84-91: Agent icon
- Line 95-97: Agent name text
- Line 98-100: Agent ID text
- Line 105-112: Trust status badge
- Line 113-117: Inactive badge
- Line 121-124: Device count badge
- Line 127-130: Last seen timestamp

**Action Buttons (line 133-148):**
- Line 134-136: Edit button
- Line 137-144: Trust/Revoke button
- Line 145-147: Delete button

**Expandable Content (line 151-176):**
- Line 157-174: Device list container
- Line 159-161: Empty state message
- Line 163-173: DeviceListItem instances

### DeviceListItem.tsx
**Location:** `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/DeviceListItem.tsx`

**Missing data-testid attributes:**

**Device Row (line 70-164):**
- Line 70-74: Device container row
- Line 76-82: Device icon
- Line 87-93: Device name + brand badge
- Line 90-92: Brand badge

**Connection Info (line 97-105):**
- Line 98-101: IP address display
- Line 102-104: Port display

**Status & Actions (line 107-163):**
- Line 108-115: Status badge (active/disabled)
- Line 118-136: Enable/Disable toggle button
- Line 141-143: More actions dropdown trigger
- Line 147-154: Edit menu item
- Line 155-161: Delete menu item

---

## 4. Device CRUD Flow Architecture

### Current Implementation
1. **Backend:** Rust handlers in agents.rs
2. **Frontend API:** TypeScript client in lib/api/agents.ts
3. **State Management:** React Query in use-devices.ts
4. **UI Layer:** ConnectivityTopology → AgentAccordionCard → DeviceListItem

### User Flow for Device CRUD
1. **Create:** Modal in ConnectivityTopology (Edit Device Modal)
2. **Read:** Displayed in AgentAccordionCard expansion
3. **Update:** Edit via DeviceListItem dropdown → Modal
4. **Delete:** Delete via DeviceListItem dropdown

### Data Flow
```
User Action
  ↓
UI Component (ConnectivityTopology/DeviceListItem)
  ↓
React Hook (useDevices)
  ↓
API Client (agents.ts)
  ↓
Backend Handler (agents.rs)
  ↓
Database (devices table)
```

---

## 5. Test Coverage Gaps

### Backend Coverage
✅ Full CRUD coverage in device_management.rs
✅ Error scenarios (404 handling)
✅ Workspace isolation
✅ Agent notification on config update

### Frontend Coverage
❌ No E2E tests for device CRUD
❌ No unit tests for device components
❌ No POM for DevicesPage/ConnectivityTopology
❌ No data-testid attributes in device UI

### Recommended Test Files to Create
1. `/home/elt1541/leeattend/webdashboard/tests/device-crud.spec.ts` - E2E tests
2. `/home/elt1541/leeattend/webdashboard/tests/pom/DevicesPage.ts` - POM
3. `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/__tests__/DeviceListItem.test.tsx` - Unit tests
4. `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/__tests__/AgentAccordionCard.test.tsx` - Unit tests

---

## 6. data-testid Naming Convention

**Observed Pattern from LoginPage.ts:**
- Format: `{feature}-{element}-{type}`
- Examples: `login-email-input`, `login-password-input`, `login-submit-btn`, `dashboard-title`

**Recommended Naming for Devices:**
- `connectivity-add-agent-btn`
- `connectivity-expand-all-btn`
- `connectivity-include-inactive-toggle`
- `device-create-name-input`
- `device-create-ip-input`
- `device-create-port-input`
- `device-create-brand-select`
- `device-create-agent-select`
- `device-create-submit-btn`
- `device-create-cancel-btn`
- `device-item-{id}-row`
- `device-item-{id}-name`
- `device-item-{id}-status-badge`
- `device-item-{id}-enable-toggle`
- `device-item-{id}-edit-btn`
- `device-item-{id}-delete-btn`
- `agent-card-{id}-header`
- `agent-card-{id}-expand-btn`
- `agent-card-{id}-devices-list`

---

## 7. Summary Findings

### Strengths
- Mature Playwright setup with worker orchestration
- Established POM pattern
- Full backend CRUD API coverage
- React Query integration for state management
- Existing E2E test examples to follow

### Gaps
- No data-testid attributes in device UI components
- No E2E tests for device CRUD flows
- No POM for devices/connectivity pages
- Missing unit tests for device components

### Effort Estimate
- Add data-testid attributes: ~30 minutes (3 files)
- Create DevicesPage POM: ~45 minutes
- Write E2E tests (CRUD flow): ~2 hours
- Unit tests (optional): ~1.5 hours
- **Total:** ~4-5 hours

---

## 8. Files to Modify

### Add data-testid attributes:
1. `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/ConnectivityTopology.tsx`
2. `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/AgentAccordionCard.tsx`
3. `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/DeviceListItem.tsx`

### Create new test files:
4. `/home/elt1541/leeattend/webdashboard/tests/pom/DevicesPage.ts`
5. `/home/elt1541/leeattend/webdashboard/tests/device-crud.spec.ts`

---

## Unresolved Questions

1. Should we test device CRUD in isolation or as part of agent management flow?
2. Do we need to test brand selection (zkteco vs mock) separately?
3. Should we test device enable/disable toggle in E2E or just unit tests?
4. Is there a visual regression testing requirement for device UI?
5. Should we test unassigned devices section separately?

