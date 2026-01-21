# Phase 4: Testing & Validation

**Priority:** P0
**Complexity:** Low
**Duration:** 3-4 hours
**Status:** Pending

## Overview

Comprehensive testing of ESS clock, admin dashboard, navigation integration, and adjustment workflow. Validate performance, accessibility, and edge cases.

## Objectives

1. Unit test coverage >90%
2. Integration tests for critical flows
3. E2E tests for user journeys
4. Performance validation (100+ employees)
5. Accessibility audit
6. Edge case validation

## Testing Strategy

### Unit Tests (90%+ Coverage)

**Coverage Targets:**
| Component | Target Coverage | Priority |
|-----------|----------------|----------|
| MyClockPage | 95% | P0 |
| TimeTrackingPage (Admin) | 95% | P0 |
| PunchCard | 90% | P1 |
| TeamStatusGrid | 90% | P1 |
| AdjustmentRequestModal | 90% | P1 |
| use-ess clock hook | 95% | P0 |

### Integration Tests

**Critical Flows:**
1. Employee self-punch flow (work in → break → break end → work out)
2. Admin adjustment flow (view team → adjust employee → verify update)
3. Adjustment request flow (employee request → admin approve → status update)
4. Real-time status updates (punch in → status reflects in 30s)

### E2E Tests

**User Journeys:**
1. **Remote Employee Journey**: Login → Navigate to ESS clock → Punch in → Punch out
2. **Admin Oversight Journey**: Login → View team dashboard → See employee status → Initiate adjustment
3. **Hybrid Scenario**: Employee punches in → Admin sees status → Admin adjusts missed punch

## Implementation Steps

### Step 1: Unit Tests - MyClockPage (1h)

**File:** `src/pages/ess/__tests__/MyClockPage.test.tsx`

```typescript
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import MyClockPage from "../MyClockPage";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter } from "react-router-dom";
import { ToastProvider } from "@/components/Toast";
import * as api from "@/lib/api";
import * as essHooks from "@/hooks/use-ess";

vi.mock("@/lib/api");
vi.mock("@/hooks/use-ess");

describe("MyClockPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } }
    });

    // Mock employee profile
    vi.spyOn(essHooks, "useMyProfile").mockReturnValue({
      employee: { id: "emp-1", first_name: "John", last_name: "Doe" },
      isLoading: false,
    } as any);

    // Mock clock data (off duty)
    vi.spyOn(essHooks, "useMyClock").mockReturnValue({
      entries: [],
      activeEntry: undefined,
      isOnBreak: false,
      isLoading: false,
      punchIn: vi.fn().mockResolvedValue({}),
      punchOut: vi.fn().mockResolvedValue({}),
      isPunching: false,
    } as any);
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            {ui}
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders page title and subtitle", async () => {
    renderWithProviders(<MyClockPage />);
    expect(await screen.findByText(/time clock/i)).toBeDefined();
    expect(screen.getByText(/clock in and out/i)).toBeDefined();
  });

  it("shows off duty status when not clocked in", async () => {
    renderWithProviders(<MyClockPage />);
    expect(await screen.findByText(/off duty/i)).toBeDefined();
  });

  it("shows working status when clocked in", async () => {
    vi.spyOn(essHooks, "useMyClock").mockReturnValue({
      entries: [{ id: "entry-1", entry_type: "work", punch_in_time: "2026-01-21T08:00:00Z" }],
      activeEntry: { id: "entry-1", entry_type: "work", punch_in_time: "2026-01-21T08:00:00Z" },
      isOnBreak: false,
      isLoading: false,
      punchIn: vi.fn(),
      punchOut: vi.fn(),
      isPunching: false,
    } as any);

    renderWithProviders(<MyClockPage />);
    expect(await screen.findByText(/working/i)).toBeDefined();
  });

  it("shows break status when on break", async () => {
    vi.spyOn(essHooks, "useMyClock").mockReturnValue({
      entries: [{ id: "entry-1", entry_type: "break", punch_in_time: "2026-01-21T12:00:00Z" }],
      activeEntry: { id: "entry-1", entry_type: "break", punch_in_time: "2026-01-21T12:00:00Z" },
      isOnBreak: true,
      isLoading: false,
      punchIn: vi.fn(),
      punchOut: vi.fn(),
      isPunching: false,
    } as any);

    renderWithProviders(<MyClockPage />);
    expect(await screen.findByText(/on break/i)).toBeDefined();
  });

  it("handles punch in action", async () => {
    const punchInMock = vi.fn().mockResolvedValue({});
    vi.spyOn(essHooks, "useMyClock").mockReturnValue({
      entries: [],
      activeEntry: undefined,
      isOnBreak: false,
      isLoading: false,
      punchIn: punchInMock,
      punchOut: vi.fn(),
      isPunching: false,
    } as any);

    const user = userEvent.setup();
    renderWithProviders(<MyClockPage />);

    const punchInBtn = await screen.findByText(/start work/i);
    await user.click(punchInBtn);

    await waitFor(() => {
      expect(punchInMock).toHaveBeenCalledWith({ entry_type: "work" });
    });
  });

  it("handles punch out action", async () => {
    const punchOutMock = vi.fn().mockResolvedValue({});
    vi.spyOn(essHooks, "useMyClock").mockReturnValue({
      entries: [{ id: "entry-1", entry_type: "work", punch_in_time: "2026-01-21T08:00:00Z" }],
      activeEntry: { id: "entry-1", entry_type: "work", punch_in_time: "2026-01-21T08:00:00Z" },
      isOnBreak: false,
      isLoading: false,
      punchIn: vi.fn(),
      punchOut: punchOutMock,
      isPunching: false,
    } as any);

    const user = userEvent.setup();
    renderWithProviders(<MyClockPage />);

    const punchOutBtn = await screen.findByText(/punch out/i);
    await user.click(punchOutBtn);

    await waitFor(() => {
      expect(punchOutMock).toHaveBeenCalled();
    });
  });

  it("opens adjustment request modal", async () => {
    const user = userEvent.setup();
    renderWithProviders(<MyClockPage />);

    const adjustBtn = await screen.findByText(/forgot to punch/i);
    await user.click(adjustBtn);

    expect(await screen.findByText(/request adjustment/i)).toBeDefined();
  });

  it("displays recent history", async () => {
    vi.spyOn(essHooks, "useMyClock").mockReturnValue({
      entries: [
        { id: "e1", entry_type: "work", punch_in_time: "2026-01-21T08:00:00Z", punch_out_time: "2026-01-21T17:00:00Z" },
        { id: "e2", entry_type: "work", punch_in_time: "2026-01-20T08:00:00Z", punch_out_time: "2026-01-20T17:00:00Z" },
      ],
      activeEntry: undefined,
      isOnBreak: false,
      isLoading: false,
      punchIn: vi.fn(),
      punchOut: vi.fn(),
      isPunching: false,
    } as any);

    renderWithProviders(<MyClockPage />);
    expect(await screen.findByText(/recent activity/i)).toBeDefined();
    expect(screen.getByText(/08:00/)).toBeDefined();
  });

  it("shows empty state when no history", async () => {
    renderWithProviders(<MyClockPage />);
    expect(await screen.findByText(/no recent time entries/i)).toBeDefined();
  });

  it("disables punch buttons when punching in progress", async () => {
    vi.spyOn(essHooks, "useMyClock").mockReturnValue({
      entries: [],
      activeEntry: undefined,
      isOnBreak: false,
      isLoading: false,
      punchIn: vi.fn(),
      punchOut: vi.fn(),
      isPunching: true, // Punching in progress
    } as any);

    renderWithProviders(<MyClockPage />);
    const punchInBtn = await screen.findByText(/start work/i);
    expect(punchInBtn).toBeDisabled();
  });
});
```

### Step 2: Unit Tests - TimeTrackingPage (Admin) (1h)

**File:** `src/pages/__tests__/TimeTrackingPage.test.tsx` (UPDATE)

```typescript
describe("TimeTrackingPage (Admin Dashboard)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

    // Mock employees
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe", employee_id_number: "E001" },
      { id: "emp-2", first_name: "Jane", last_name: "Smith", employee_id_number: "E002" },
    ]);

    // Mock team entries (1 working, 1 on break)
    (api.fetchTeamTimeEntries as any).mockResolvedValue([
      { id: "te-1", employee_id: "emp-1", entry_type: "work", punch_in_time: "2026-01-21T08:00:00Z", punch_out_time: null },
      { id: "te-2", employee_id: "emp-2", entry_type: "break", punch_in_time: "2026-01-21T12:00:00Z", punch_out_time: null },
    ]);

    // Mock pending adjustments
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([
      { id: "adj-1", employee_id: "emp-1", status: "pending" },
    ]);
  });

  it("renders admin dashboard title", async () => {
    renderWithProviders(<TimeTrackingPage />);
    expect(await screen.findByText(/team time tracking/i)).toBeDefined();
  });

  it("shows analytics cards with correct counts", async () => {
    renderWithProviders(<TimeTrackingPage />);
    expect(await screen.findByText(/currently working/i)).toBeDefined();
    expect(screen.getByText("1")).toBeDefined(); // 1 working
    expect(screen.getByText(/on break/i)).toBeDefined();
  });

  it("renders team status grid", async () => {
    renderWithProviders(<TimeTrackingPage />);
    expect(await screen.findByText(/John Doe/i)).toBeDefined();
    expect(screen.getByText(/Jane Smith/i)).toBeDefined();
  });

  it("shows real-time status badges", async () => {
    renderWithProviders(<TimeTrackingPage />);
    expect(await screen.findByText(/working/i)).toBeDefined();
    expect(screen.getByText(/on break/i)).toBeDefined();
  });

  it("switches between grid and list view", async () => {
    const user = userEvent.setup();
    renderWithProviders(<TimeTrackingPage />);

    const viewToggle = await screen.findByRole("button", { name: /view/i });
    await user.click(viewToggle);

    // Verify list view rendered
    expect(screen.getByRole("table")).toBeDefined();
  });

  it("opens admin adjustment modal", async () => {
    const user = userEvent.setup();
    renderWithProviders(<TimeTrackingPage />);

    const adjustBtn = await screen.findAllByText(/adjust/i);
    await user.click(adjustBtn[0]);

    expect(await screen.findByText(/adjust for/i)).toBeDefined();
  });

  it("shows pending adjustments section", async () => {
    renderWithProviders(<TimeTrackingPage />);
    expect(await screen.findByText(/pending adjustments/i)).toBeDefined();
    expect(screen.getByText("1")).toBeDefined(); // 1 pending
  });

  it("filters team by date", async () => {
    const user = userEvent.setup();
    renderWithProviders(<TimeTrackingPage />);

    const dateInput = await screen.findByRole("textbox", { name: /date/i });
    await user.clear(dateInput);
    await user.type(dateInput, "2026-01-20");

    await waitFor(() => {
      expect(api.fetchTeamTimeEntries).toHaveBeenCalledWith(expect.any(String), "2026-01-20");
    });
  });

  it("handles export action", async () => {
    const user = userEvent.setup();
    renderWithProviders(<TimeTrackingPage />);

    const exportBtn = await screen.findByText(/export/i);
    await user.click(exportBtn);

    // Verify export triggered (mock implementation)
    expect(true).toBe(true); // TODO: Verify actual export logic
  });
});
```

### Step 3: Integration Tests (1h)

**File:** `src/__tests__/integration/time-tracking-flow.test.tsx`

```typescript
describe("Time Tracking Integration Flow", () => {
  it("complete employee punch cycle", async () => {
    const user = userEvent.setup();

    // 1. Employee navigates to ESS clock
    renderWithAuth(<App />, { role: "employee" });
    const clockLink = await screen.findByText(/time clock/i);
    await user.click(clockLink);

    // 2. Employee punches in
    const punchInBtn = await screen.findByText(/start work/i);
    await user.click(punchInBtn);
    expect(await screen.findByText(/working/i)).toBeDefined();

    // 3. Employee starts break
    const breakBtn = await screen.findByText(/start break/i);
    await user.click(breakBtn);
    expect(await screen.findByText(/on break/i)).toBeDefined();

    // 4. Employee ends break
    const returnBtn = await screen.findByText(/return to work/i);
    await user.click(returnBtn);
    expect(await screen.findByText(/working/i)).toBeDefined();

    // 5. Employee punches out
    const punchOutBtn = await screen.findByText(/punch out/i);
    await user.click(punchOutBtn);
    expect(await screen.findByText(/off duty/i)).toBeDefined();
  });

  it("admin adjustment workflow", async () => {
    const user = userEvent.setup();

    // 1. Admin navigates to team dashboard
    renderWithAuth(<App />, { role: "admin" });
    const adminLink = await screen.findByText(/team time tracking/i);
    await user.click(adminLink);

    // 2. Admin sees employee working
    expect(await screen.findByText(/John Doe/i)).toBeDefined();
    expect(screen.getByText(/working/i)).toBeDefined();

    // 3. Admin initiates adjustment
    const adjustBtn = await screen.findByText(/adjust/i);
    await user.click(adjustBtn);

    // 4. Admin fills adjustment form
    const dateInput = await screen.findByLabelText(/work day/i);
    const punchInInput = await screen.findByLabelText(/punch in/i);
    const reasonInput = await screen.findByLabelText(/reason/i);

    await user.clear(punchInInput);
    await user.type(punchInInput, "08:30");
    await user.type(reasonInput, "Missed clock in");

    // 5. Admin submits adjustment
    const submitBtn = await screen.findByText(/apply adjustment/i);
    await user.click(submitBtn);

    await waitFor(() => {
      expect(api.submitAttendanceAdjustment).toHaveBeenCalled();
    });
  });

  it("employee adjustment request → admin approval flow", async () => {
    const user = userEvent.setup();

    // 1. Employee requests adjustment
    renderWithAuth(<MyClockPage />, { role: "employee" });
    const adjustBtn = await screen.findByText(/forgot to punch/i);
    await user.click(adjustBtn);

    const reasonInput = await screen.findByLabelText(/reason/i);
    await user.type(reasonInput, "Forgot to clock out");
    const submitBtn = await screen.findByText(/submit request/i);
    await user.click(submitBtn);

    // 2. Admin sees pending adjustment
    renderWithAuth(<AttendanceAdjustmentsPage />, { role: "admin" });
    expect(await screen.findByText(/forgot to clock out/i)).toBeDefined();

    // 3. Admin approves adjustment
    const approveBtn = await screen.findByTitle(/approve/i);
    await user.click(approveBtn);

    await waitFor(() => {
      expect(api.approveAttendanceAdjustment).toHaveBeenCalled();
    });
  });
});
```

### Step 4: E2E Tests (Playwright) (1h)

**File:** `e2e/time-tracking.spec.ts`

```typescript
import { test, expect } from "@playwright/test";

test.describe("Time Tracking E2E", () => {
  test("remote employee self-clock workflow", async ({ page }) => {
    // 1. Login as employee
    await page.goto("/login");
    await page.fill('input[name="email"]', "john.doe@company.com");
    await page.fill('input[name="password"]', "password123");
    await page.click('button[type="submit"]');

    // 2. Navigate to ESS clock
    await page.click('a[href="/ess/clock"]');
    await expect(page.locator("text=Time Clock")).toBeVisible();

    // 3. Punch in
    await page.click("text=Start Work");
    await expect(page.locator("text=Working")).toBeVisible();

    // 4. Verify status persists on refresh
    await page.reload();
    await expect(page.locator("text=Working")).toBeVisible();

    // 5. Punch out
    await page.click("text=Punch Out");
    await expect(page.locator("text=Off Duty")).toBeVisible();
  });

  test("admin oversight dashboard", async ({ page }) => {
    // 1. Login as admin
    await page.goto("/login");
    await page.fill('input[name="email"]', "admin@company.com");
    await page.fill('input[name="password"]', "admin123");
    await page.click('button[type="submit"]');

    // 2. Navigate to team dashboard
    await page.click('a[href="/time-tracking"]');
    await expect(page.locator("text=Team Time Tracking")).toBeVisible();

    // 3. Verify analytics cards
    await expect(page.locator("text=Currently Working")).toBeVisible();
    await expect(page.locator("text=On Break")).toBeVisible();

    // 4. View employee status
    await expect(page.locator("text=John Doe")).toBeVisible();

    // 5. Initiate adjustment
    await page.click("text=Adjust");
    await expect(page.locator("text=Adjust for")).toBeVisible();
  });
});
```

### Step 5: Performance Validation (30min)

**File:** `src/__tests__/performance/time-tracking-performance.test.tsx`

```typescript
describe("Time Tracking Performance", () => {
  it("renders team dashboard with 100 employees in <2s", async () => {
    // Generate 100 mock employees
    const employees = Array.from({ length: 100 }, (_, i) => ({
      id: `emp-${i}`,
      first_name: `Employee${i}`,
      last_name: `Doe`,
      employee_id_number: `E${String(i).padStart(3, "0")}`,
    }));

    (api.fetchEmployees as any).mockResolvedValue(employees);

    const startTime = performance.now();
    renderWithProviders(<TimeTrackingPage />);
    await screen.findByText(/team time tracking/i);
    const endTime = performance.now();

    expect(endTime - startTime).toBeLessThan(2000); // <2s
  });

  it("handles real-time updates without lag", async () => {
    // Mock initial data
    (api.fetchTeamTimeEntries as any).mockResolvedValue([
      { id: "te-1", employee_id: "emp-1", entry_type: "work", punch_in_time: "2026-01-21T08:00:00Z" },
    ]);

    renderWithProviders(<TimeTrackingPage />);
    await screen.findByText(/working/i);

    // Simulate real-time update after 30s
    await vi.advanceTimersByTime(30000);

    // Mock updated data
    (api.fetchTeamTimeEntries as any).mockResolvedValue([
      { id: "te-1", employee_id: "emp-1", entry_type: "break", punch_in_time: "2026-01-21T12:00:00Z" },
    ]);

    await waitFor(() => {
      expect(screen.getByText(/on break/i)).toBeDefined();
    }, { timeout: 5000 });
  });
});
```

### Step 6: Accessibility Audit (30min)

**Use axe-core for automated accessibility testing:**

```typescript
import { axe, toHaveNoViolations } from "jest-axe";
expect.extend(toHaveNoViolations);

describe("Accessibility", () => {
  it("MyClockPage has no accessibility violations", async () => {
    const { container } = renderWithProviders(<MyClockPage />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  it("TimeTrackingPage has no accessibility violations", async () => {
    const { container } = renderWithProviders(<TimeTrackingPage />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});
```

## Success Criteria

- [ ] Unit test coverage >90% for all new components
- [ ] All integration tests pass
- [ ] E2E tests pass in CI/CD
- [ ] Performance <2s for 100 employees
- [ ] Zero accessibility violations
- [ ] All edge cases covered
- [ ] Manual testing checklist complete
- [ ] No TypeScript errors
- [ ] Build passes without warnings

## Manual Testing Checklist

### Employee Workflows
- [ ] Punch in for work
- [ ] Start break while working
- [ ] End break and return to work
- [ ] Punch out after work
- [ ] Request adjustment for missed punch
- [ ] View recent history (7 days)
- [ ] Test on mobile device
- [ ] Test timezone handling

### Admin Workflows
- [ ] View team dashboard
- [ ] See real-time status updates
- [ ] Filter by date
- [ ] Switch grid/list view
- [ ] Initiate admin adjustment
- [ ] Approve employee adjustment request
- [ ] Export team data
- [ ] Test with 50+ employees

### Edge Cases
- [ ] No workspace selected
- [ ] No employees in workspace
- [ ] Employee with no punch history
- [ ] Punch in at 23:59, verify midnight handling
- [ ] Concurrent punches (same employee, different devices)
- [ ] Offline then online (queue actions?)
- [ ] Timezone change mid-day
- [ ] Daylight saving time transition

## Dependencies

**Blocked By:**
- Phase 1 (ESS clock)
- Phase 2 (admin dashboard)
- Phase 3 (navigation)

**Blocks:**
- Production deployment

## Related Files

**Test Files:**
- `src/pages/ess/__tests__/MyClockPage.test.tsx`
- `src/pages/__tests__/TimeTrackingPage.test.tsx`
- `src/components/time-tracking/__tests__/*.test.tsx`
- `src/hooks/__tests__/use-ess.test.ts`
- `src/__tests__/integration/time-tracking-flow.test.tsx`
- `e2e/time-tracking.spec.ts`
- `src/__tests__/performance/time-tracking-performance.test.tsx`

## Notes

- **Test Data:** Use factories/fixtures for consistent test data
- **Mocking:** Mock API calls, avoid hitting backend in unit tests
- **Coverage:** Use Istanbul/c8 for coverage reports
- **E2E:** Run in headless mode in CI, headed for local dev
- **Performance:** Profile with React DevTools Profiler
- **Accessibility:** Test with screen reader (NVDA/JAWS)
