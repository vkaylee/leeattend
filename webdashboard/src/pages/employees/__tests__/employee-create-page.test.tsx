import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api";
import { ApiError } from "@/lib/api/errors";
import { clearRuntimeFeatureFlags, setRuntimeFeatureFlags } from "@/lib/feature-flags";

import EmployeeCreatePage from "../employee-create-page";

const navigate = vi.fn();
let mockSearch: { enrollment_id?: string; staging_log_id?: string } = {};
let mockPermissions: string[] = [];

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => navigate,
  useSearch: () => mockSearch,
  useBlocker: () => ({ status: "unblocked" }),
  Link: ({ children, ...props }: { children: React.ReactNode; [key: string]: unknown }) => <a {...props}>{children}</a>,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string }) => options?.defaultValue ?? key,
  }),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    permissions: mockPermissions,
    roles: [],
    managedDepartmentIds: null,
    isLoading: false,
    hasPermission: (perm: string) => mockPermissions.includes(perm),
    hasAnyPermission: (...perms: string[]) => perms.some((p) => mockPermissions.includes(p)),
    hasRole: () => false,
    hasScopedPermission: (perm: string) => mockPermissions.includes(perm),
  }),
}));

vi.mock("@/lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/api")>()),
  createEmployee: vi.fn(),
  createEmployeeTenure: vi.fn(),
  matchEmployeeIdentity: vi.fn(),
  resolveStagingLog: vi.fn(),
  fetchEmployers: vi.fn().mockResolvedValue([
    { id: "11111111-1111-1111-1111-111111111111", name: "Acme Corp", tax_code: "0101234567" },
  ]),
  fetchLocations: vi.fn().mockResolvedValue([]),
}));

vi.mock("@/pages/settings/hooks/use-locations", () => ({
  useLocations: () => ({ data: [], isError: false }),
}));

function renderPage() {
  return render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <ToastProvider><EmployeeCreatePage /></ToastProvider>
    </QueryClientProvider>,
  );
}

describe("EmployeeCreatePage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    clearRuntimeFeatureFlags();
    mockSearch = {};
    mockPermissions = [];
  });

  it("renders HR fields without account linking controls and gates identity fields by flag and role permission", () => {
    mockPermissions = ["employees:create", "employees:identity:match"];
    const { rerender } = renderPage();

    expect(screen.getByTestId("employees-form-name-input")).toBeDefined();
    expect(screen.getByTestId("employees-form-id-input")).toBeDefined();
    expect(screen.getByTestId("employees-form-hire-date-input")).toBeDefined();
    expect(screen.queryByTestId("employees-form-user-combobox")).toBeNull();
    // Flag defaults OFF -> identity inputs stay hidden even for admin/hr_manager.
    expect(screen.queryByTestId("employees-form-identity-fields")).toBeNull();

    // Flag ON without employees:identity:match -> non-pilot roles remain hidden.
    mockPermissions = ["employees:create"];
    setRuntimeFeatureFlags([{ key: "ff_employee_identity_match", enabled: true }]);
    rerender(
      <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
        <ToastProvider><EmployeeCreatePage /></ToastProvider>
      </QueryClientProvider>,
    );
    expect(screen.queryByTestId("employees-form-identity-fields")).toBeNull();
  });

  it("creates employee without sending user_id and blocks duplicate submit while pending", async () => {
    const user = userEvent.setup();
    let resolveCreate: ((value: unknown) => void) | undefined;
    vi.mocked(api.createEmployee).mockImplementation(
      () => new Promise((resolve) => { resolveCreate = resolve; }) as ReturnType<typeof api.createEmployee>,
    );
    renderPage();

    await user.type(screen.getByTestId("employees-form-name-input"), "New Employee");
    await user.type(screen.getByTestId("employees-form-id-input"), "EMP-100");
    fireEvent.change(screen.getByTestId("employees-form-hire-date-input"), { target: { value: "2026-11-19" } });

    const form = document.getElementById("employee-create-form")!;
    fireEvent.submit(form);
    fireEvent.submit(form);

    await waitFor(() => {
      expect(api.createEmployee).toHaveBeenCalledTimes(1);
    });
    const [payload] = vi.mocked(api.createEmployee).mock.calls[0];
    expect(payload).not.toHaveProperty("user_id");

    resolveCreate?.({ id: "employee-1" });
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith({ to: "/employees/$employeeId", params: { employeeId: "employee-1" } });
    });
  });

  it("runs debounced preflight for none, stale, and hard terminated reuse flow without silent reuse", async () => {
    const user = userEvent.setup();
    setRuntimeFeatureFlags([{ key: "ff_employee_identity_match", enabled: true }]);
    mockPermissions = ["employees:create", "employees:identity:match", "employees:rehire"];
    vi.mocked(api.matchEmployeeIdentity)
      .mockResolvedValueOnce({ tier: "none", candidates: [] })
      .mockResolvedValueOnce({
        tier: "hard",
        candidates: [
          {
            employee_id: "22222222-2222-2222-2222-222222222222",
            display_name: "Nguyễn Văn Cũ",
            employee_code: "EMP-OLD-01",
            status: "terminated",
            tier: "hard",
            reasons: ["national_id_exact"],
          },
        ],
      });
    vi.mocked(api.createEmployeeTenure).mockResolvedValue({
      tenure_id: "33333333-3333-3333-3333-333333333333",
      employee_id: "22222222-2222-2222-2222-222222222222",
      employee_code: "EMP-OLD-01",
      hire_type: "rehire",
      prior_service_credit_days: 0,
      start_date: "2026-12-01",
      relationship_id: "44444444-4444-4444-4444-444444444444",
      employment_id: "55555555-5555-5555-5555-555555555555",
      employer_id: "11111111-1111-1111-1111-111111111111",
      status: "active",
    });

    renderPage();
    expect(screen.getByTestId("employees-form-identity-fields")).toBeDefined();

    fireEvent.change(screen.getByTestId("employees-form-national-id-input"), { target: { value: "001111222" } });
    await waitFor(() => {
      expect(screen.getByTestId("identity-preflight-none")).toBeDefined();
    }, { timeout: 2000 });

    // Changing input immediately transitions to stale before next debounced match completes.
    fireEvent.change(screen.getByTestId("employees-form-national-id-input"), { target: { value: "001234567" } });
    expect(screen.getByTestId("identity-preflight-stale")).toBeDefined();

    await waitFor(() => {
      expect(screen.getByTestId("identity-preflight-hard")).toBeDefined();
    }, { timeout: 2000 });

    // Hard match blocks create-new until explicit reuse is chosen.
    expect((screen.getByTestId("save-employee-btn") as HTMLButtonElement).disabled).toBe(true);

    // Explicitly choose Reuse existing employee.
    await user.click(screen.getByTestId("reuse-candidate-22222222-2222-2222-2222-222222222222"));
    expect(screen.getByTestId("identity-preflight-reuse-selected")).toBeDefined();

    fireEvent.change(screen.getByTestId("employees-form-hire-date-input"), { target: { value: "2026-12-01" } });

    // Submitting reuse without employer shows inline validation error.
    fireEvent.submit(document.getElementById("employee-create-form")!);
    expect(screen.getByTestId("employees-form-employer-error")).toBeDefined();
    expect(api.createEmployeeTenure).not.toHaveBeenCalled();

    // Select employer via combobox and submit reuse-tenure endpoint (never createEmployee).
    await screen.findByTestId("employees-form-employer-combobox-option-11111111-1111-1111-1111-111111111111");
    await user.selectOptions(screen.getByTestId("employees-form-employer-combobox"), "11111111-1111-1111-1111-111111111111");
    fireEvent.submit(document.getElementById("employee-create-form")!);

    await waitFor(() => {
      expect(api.createEmployeeTenure).toHaveBeenCalledWith("22222222-2222-2222-2222-222222222222", {
        start_date: "2026-12-01",
        employer_id: "11111111-1111-1111-1111-111111111111",
        primary_location_id: undefined,
        expected_employee_code: "EMP-OLD-01",
        expected_status: "terminated",
      });
    });
    expect(api.createEmployee).not.toHaveBeenCalled();
  });

  it("never auto-reuses soft candidates and requires explicit confirmation before creating new employee", async () => {
    const user = userEvent.setup();
    setRuntimeFeatureFlags([{ key: "ff_employee_identity_match", enabled: true }]);
    mockPermissions = ["employees:create", "employees:identity:match", "employees:rehire"];
    vi.mocked(api.matchEmployeeIdentity).mockResolvedValue({
      tier: "soft",
      candidates: [
        {
          employee_id: "66666666-6666-6666-6666-666666666666",
          display_name: "Trần Thị B",
          employee_code: "EMP-SOFT-01",
          status: "active",
          tier: "soft",
          reasons: ["name_dob_review"],
        },
      ],
    });
    vi.mocked(api.createEmployee).mockResolvedValue({ id: "employee-new" } as never);

    renderPage();
    fireEvent.change(screen.getByTestId("employees-form-name-input"), { target: { value: "Trần Thị B" } });
    fireEvent.change(screen.getByTestId("employees-form-id-input"), { target: { value: "EMP-NEW-02" } });
    fireEvent.change(screen.getByTestId("employees-form-hire-date-input"), { target: { value: "2026-11-25" } });
    fireEvent.change(screen.getByTestId("employees-form-dob-input"), { target: { value: "1993-04-10" } });

    await waitFor(() => {
      expect(screen.getByTestId("identity-preflight-soft")).toBeDefined();
    }, { timeout: 2000 });

    const saveBtn = screen.getByTestId("save-employee-btn") as HTMLButtonElement;
    expect(saveBtn.disabled).toBe(true);

    const ackBtn = screen.getByTestId("soft-match-ack-btn");
    expect(ackBtn.getAttribute("aria-pressed")).toBe("false");
    await user.click(ackBtn);
    expect(ackBtn.getAttribute("aria-pressed")).toBe("true");
    expect(saveBtn.disabled).toBe(false);

    fireEvent.submit(document.getElementById("employee-create-form")!);
    await waitFor(() => {
      expect(api.createEmployee).toHaveBeenCalledTimes(1);
    });
    expect(api.createEmployeeTenure).not.toHaveBeenCalled();
  });

  it("handles preflight API error + retry and 409 conflict without losing form state", async () => {
    const user = userEvent.setup();
    setRuntimeFeatureFlags([{ key: "ff_employee_identity_match", enabled: true }]);
    mockPermissions = ["employees:create", "employees:identity:match"];
    vi.mocked(api.matchEmployeeIdentity)
      .mockRejectedValueOnce(new ApiError(429, "Too Many Requests", "RATE_LIMITED"))
      .mockResolvedValue({ tier: "none", candidates: [] });
    vi.mocked(api.createEmployee).mockRejectedValueOnce(new ApiError(409, "Conflict", "CONFLICT"));

    renderPage();
    fireEvent.change(screen.getByTestId("employees-form-name-input"), { target: { value: "Lê Văn C" } });
    fireEvent.change(screen.getByTestId("employees-form-id-input"), { target: { value: "EMP-CONF-01" } });
    fireEvent.change(screen.getByTestId("employees-form-hire-date-input"), { target: { value: "2026-11-25" } });
    fireEvent.change(screen.getByTestId("employees-form-phone-input"), { target: { value: "+84912345678" } });

    await waitFor(() => {
      expect(screen.getByTestId("identity-preflight-error")).toBeDefined();
    }, { timeout: 2000 });

    await user.click(screen.getByTestId("identity-preflight-retry-btn"));
    await waitFor(() => {
      expect(screen.getByTestId("identity-preflight-none")).toBeDefined();
    }, { timeout: 2000 });

    fireEvent.submit(document.getElementById("employee-create-form")!);
    await waitFor(() => {
      expect(screen.getByTestId("identity-preflight-conflict")).toBeDefined();
    });
    expect((screen.getByTestId("employees-form-name-input") as HTMLInputElement).value).toBe("Lê Văn C");
  });

  it("routes attendance staging prefill through preflight without treating raw_user_id as identity proof", async () => {
    const user = userEvent.setup();
    setRuntimeFeatureFlags([{ key: "ff_employee_identity_match", enabled: true }]);
    mockPermissions = ["employees:create", "employees:identity:match"];
    mockSearch = {
      enrollment_id: "BADGE-9001",
      staging_log_id: "77777777-7777-7777-7777-777777777777",
    };
    vi.mocked(api.matchEmployeeIdentity).mockResolvedValue({
      tier: "hard",
      candidates: [
        {
          employee_id: "88888888-8888-8888-8888-888888888888",
          display_name: "Phạm Thị D",
          employee_code: "EMP-EXIST-09",
          status: "active",
          tier: "hard",
          reasons: ["phone_exact"],
        },
      ],
    });
    vi.mocked(api.resolveStagingLog).mockResolvedValue(undefined);

    renderPage();
    expect(screen.getByTestId("staging-prefill-notice")).toBeDefined();
    expect((screen.getByTestId("employees-form-id-input") as HTMLInputElement).value).toBe("BADGE-9001");
    // raw enrollment_id must NEVER trigger a match lookup by itself.
    expect(api.matchEmployeeIdentity).not.toHaveBeenCalled();

    fireEvent.change(screen.getByTestId("employees-form-phone-input"), { target: { value: "+84919998888" } });
    await waitFor(() => {
      expect(api.matchEmployeeIdentity).toHaveBeenCalledWith({ phone: "+84919998888" });
    }, { timeout: 2000 });
    expect(screen.getByTestId("identity-preflight-hard")).toBeDefined();

    await user.click(screen.getByTestId("assign-staging-candidate-88888888-8888-8888-8888-888888888888"));
    await waitFor(() => {
      expect(api.resolveStagingLog).toHaveBeenCalledWith(
        "77777777-7777-7777-7777-777777777777",
        "88888888-8888-8888-8888-888888888888",
      );
    });
  });
});
