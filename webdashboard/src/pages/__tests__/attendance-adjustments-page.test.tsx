import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import * as api from "../../lib/api";
import AttendanceAdjustmentsPage from "../attendance-adjustments-page";

(globalThis as any).__mockPermissions = ["attendance:adjustments:manage_behalf"];

vi.mock("../../hooks/use-permissions", () => ({
  useMyPermissions: vi.fn(() => {
    const perms = (globalThis as any).__mockPermissions || [];
    return {
      hasPermission: (perm: string) => perms.includes(perm),
      hasAnyPermission: (...p: string[]) => p.some((x) => perms.includes(x)),
      hasScopedPermission: () => true,
      hasRole: () => true,
      isLoading: false,
      permissions: perms,
      roles: [],
      managedDepartmentIds: null,
    };
  }),
}));

// Mock TanStack Router — stateful + reactive so navigate({ search }) triggers re-render.
vi.mock("@tanstack/react-router", () => {
  let _search: Record<string, unknown> = { tab: "adjustments", status: "all" };
  const _listeners = new Set<() => void>();

  function notify() {
    _listeners.forEach((fn) => fn());
  }

  function useSearchState() {
    const [search, setSearch] = React.useState(_search);
    React.useEffect(() => {
      const listener = () => setSearch({ ..._search });
      _listeners.add(listener);
      return () => {
        _listeners.delete(listener);
      };
    }, []);
    return search;
  }

  return {
    useNavigate: () =>
      (opts: { to?: string; search?: ((prev: unknown) => Record<string, unknown>) | Record<string, unknown>; replace?: boolean }) => {
        if (opts.search) {
          if (typeof opts.search === "function") {
            _search = opts.search(_search);
          } else {
            _search = opts.search as Record<string, unknown>;
          }
          notify();
        }
      },
    useLocation: () => ({ pathname: "/", search: "", hash: "" }),
    useSearch: useSearchState,
    createFileRoute: () => () => ({
      useSearch: useSearchState,
    }),
    Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
  };
});

// Mock useBehalfEmployees
vi.mock("@/hooks/use-behalf-employees", () => ({
  useBehalfEmployees: vi.fn(() => ({
    employees: [
      { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", is_active: true, active_employment_id: "employment-1" }
    ],
    hasAssignments: true,
    isLoading: false,
    isError: false,
  })),
}));

// Mock Combobox
vi.mock("@/components/ui/combobox", () => ({
  Combobox: ({ id, value, onChange, options }: any) => (
    <select
      data-testid="mock-employee-search"
      id={id}
      onChange={(e) => onChange(e.target.value)}
      value={value}
    >
      <option value="">Select...</option>
      {options.map((opt: any) => (
        <option key={opt.value} value={opt.value}>{opt.label}</option>
      ))}
    </select>
  ),
}));

// Mock DropdownMenu for Radix UI simulation in tests
vi.mock("@/components/ui/dropdown-menu", () => ({
  DropdownMenu: ({ children }: any) => <div>{children}</div>,
  DropdownMenuTrigger: ({ children }: any) => <div>{children}</div>,
  DropdownMenuContent: ({ children }: any) => <div data-testid="dropdown-content">{children}</div>,
  DropdownMenuLabel: ({ children }: any) => <div>{children}</div>,
  DropdownMenuSeparator: () => <hr />,
}));

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchWorkspaceAttendanceAdjustments: vi.fn(),
  fetchEmployees: vi.fn(),
  approveAttendanceAdjustment: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  submitAttendanceAdjustment: vi.fn(),
  fetchWorkspace: vi.fn(),
}));

// Override i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("AttendanceAdjustmentsPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
    });

    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("test-ws-id");
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: [
        { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z", tags: [] }
      ],
      total: 1,
      page: 1,
      limit: 50,
    });
    vi.mocked(api.fetchWorkspace).mockResolvedValue({
      id: "test-ws-id",
      name: "Test Workspace",
      timezone: "Asia/Ho_Chi_Minh",
      created_at: "2025-01-01T00:00:00Z",
      date_format: "DD/MM/YYYY",
      time_format: "HH:mm",
      theme_primary_color: "#000000",
      default_holiday_ot_multiplier: "2.0",
      default_language: "vi",
      holiday_overnight_strategy: "split",
    });
    vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue({
      data: [
        { 
          id: "adj-1", 
          employee_id: "emp-1", 
          requested_date: "2026-01-01", 
          requested_punch_in: "08:00", 
          requested_punch_out: "17:00", 
          status: "pending", 
          reason: "Forgot to punch" 
        }
      ],
      total: 1,
      page: 1,
      limit: 20,
    } as any);
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {ui}
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders list of adjustment requests", async () => {
    renderWithProviders(<AttendanceAdjustmentsPage />);
    // ResponsiveTable renders both desktop and mobile views, so use findAllByText
    expect((await screen.findAllByText(/John Doe/i)).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Forgot to punch/i).length).toBeGreaterThan(0);
  });

  it("handles approve action", async () => {
    (globalThis as any).__mockPermissions = [
      "attendance:adjustments:manage_behalf",
      "attendance:adjustments:approve"
    ];
    const user = userEvent.setup();
    vi.mocked(api.approveAttendanceAdjustment).mockResolvedValue(undefined);

    renderWithProviders(<AttendanceAdjustmentsPage />);

    // ResponsiveTable renders both desktop and mobile views, so use findAllByTitle
    const approveButtons = await screen.findAllByTitle(/common.approve/i);
    await user.click(approveButtons[0]);

    await waitFor(() => {
      expect(api.approveAttendanceAdjustment).toHaveBeenCalledWith("test-ws-id", "adj-1", { status: "approved" });
    });
  });

  it("handles request on behalf action", async () => {
    (globalThis as any).__mockPermissions = [
      "attendance:adjustments:manage_behalf",
      "attendance:adjustments:submit_on_behalf"
    ];
    const user = userEvent.setup();
    vi.mocked(api.submitAttendanceAdjustment).mockResolvedValue({} as any);

    renderWithProviders(<AttendanceAdjustmentsPage />);

    // 1. Verify "Request Adjustment" button is visible
    const requestButton = await screen.findByRole("button", { name: /time_tracking.modal.new_title/i });
    expect(requestButton).toBeDefined();

    // 2. Click to open modal
    await user.click(requestButton);

    // 3. Verify modal title is displayed
    expect(await screen.findByText("time_tracking.modal.new_title")).toBeDefined();

    // 4. Fill form
    const selectElement = screen.getAllByTestId("mock-employee-search").find(el => el.id === "adj-emp")!;
    await user.selectOptions(selectElement, "employment-1");

    const dateInput = screen.getByLabelText(/common.work_day/i);
    await user.clear(dateInput);
    await user.type(dateInput, "2026-06-05");

    const reasonInput = screen.getByPlaceholderText("time_tracking.modal.reason_placeholder");
    await user.type(reasonInput, "On behalf test reason");

    // Submit form
    const submitButton = screen.getByRole("button", { name: /common.submit_request/i });
    await user.click(submitButton);

    await waitFor(() => {
      expect(api.submitAttendanceAdjustment).toHaveBeenCalled();
    });
  });

  it("handles status filter changes from table header dropdown", async () => {
    const user = userEvent.setup();
    vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue({
      data: [
        { 
          id: "adj-pending-1", 
          employee_id: "emp-1", 
          requested_date: "2026-01-01", 
          requested_punch_in: "08:00", 
          requested_punch_out: "17:00", 
          status: "pending", 
          reason: "Forgot to punch" 
        }
      ],
      total: 1,
      page: 1,
      limit: 20,
    } as any);

    renderWithProviders(<AttendanceAdjustmentsPage />);

    // Initial fetch should be triggered with status: undefined (since statusFilter defaults to "all")
    await waitFor(() => {
      expect(api.fetchWorkspaceAttendanceAdjustments).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
        status: undefined,
      }));
    });

    // Clear calls to mock to isolate the dropdown filter click trigger
    vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockClear();

    // Click the "Pending" option inside the dropdown (FilterPanel renders Buttons with data-testid)
    const pendingOption = document.querySelector('[data-testid="adjustments-status-filter-option-pending"]');
    expect(pendingOption).not.toBeNull();
    await user.click(pendingOption!);

    // Wait for the query to be triggered with status: "pending"
    await waitFor(() => {
      expect(api.fetchWorkspaceAttendanceAdjustments).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
        status: "pending",
      }));
    });
  });
});
