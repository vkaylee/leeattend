import { beforeEach, describe, expect, it, vi } from "vitest";

const { redirect, fetchMyPermissions } = vi.hoisted(() => ({
  redirect: vi.fn((options: { to: string }) => ({ ...options, isRedirect: true })),
  fetchMyPermissions: vi.fn(),
}));

vi.mock("@tanstack/react-router", () => ({
  createFileRoute: () => (options: unknown) => ({ options }),
  redirect,
  Outlet: () => null,
}));

vi.mock("@/lib/api", () => ({ fetchMyPermissions }));
vi.mock("@/components/layout/app-shell", () => ({ default: () => null }));
vi.mock("@/components/common/error-boundary", () => ({
  ErrorBoundary: ({ children }: { children: unknown }) => children,
}));
vi.mock("@/pages/settings/components/settings-sidebar", () => ({
  SettingsSidebar: () => null,
}));
vi.mock("@/components/ui/button", () => ({ default: () => null }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

import { Route as InfraRoute } from "../_authenticated/_infra";
import { Route as AssignmentsRoute } from "../_authenticated/_organization/assignments";
import { Route as ContractsRoute } from "../_authenticated/_organization/contracts";
import { Route as EmployeeDetailRoute } from "../_authenticated/_organization/employees_.$employeeId";
import { Route as LifecycleRoute } from "../_authenticated/_organization/lifecycle";
import { Route as PayrollRoute } from "../_authenticated/_organization/payroll";
import { Route as PayrollSettingsRoute } from "../_authenticated/_organization/settings/payroll";
import { Route as SalaryManagementRoute } from "../_authenticated/_organization/salary-management";
import { Route as SalaryManagementSetupRoute } from "../_authenticated/_organization/salary-management.setup";
import { Route as SettingsRoute } from "../_authenticated/_organization/settings";
import { Route as WorkingConditionsRoute } from "../_authenticated/_organization/working-conditions";

interface RouteOptions {
  beforeLoad?: (args?: { params?: { employeeId?: string }; search?: unknown }) => Promise<void> | void;
}

const settingsBeforeLoad = (SettingsRoute.options as RouteOptions).beforeLoad!;
const infraBeforeLoad = (InfraRoute.options as RouteOptions).beforeLoad!;
const contractsBeforeLoad = (ContractsRoute.options as RouteOptions).beforeLoad!;
const personnelHubBeforeLoads = [
  (AssignmentsRoute.options as RouteOptions).beforeLoad!,
  (WorkingConditionsRoute.options as RouteOptions).beforeLoad!,
  (LifecycleRoute.options as RouteOptions).beforeLoad!,
];
const payrollBeforeLoads = [
  (PayrollRoute.options as RouteOptions).beforeLoad!,
  (PayrollSettingsRoute.options as RouteOptions).beforeLoad!,
];
const salaryManagementBeforeLoad = (SalaryManagementRoute.options as RouteOptions).beforeLoad!;
const salaryManagementSetupBeforeLoad = (SalaryManagementSetupRoute.options as RouteOptions).beforeLoad!;

describe("permission route guards", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("allows settings users with any settings permission", async () => {
    fetchMyPermissions.mockResolvedValue({
      permissions: ["organization:settings:read"],
    });

    await expect(settingsBeforeLoad()).resolves.toBeUndefined();
    expect(redirect).not.toHaveBeenCalled();
  });

  it("redirects users without settings permission", async () => {
    fetchMyPermissions.mockResolvedValue({ permissions: ["ess:attendance"] });

    await expect(settingsBeforeLoad()).rejects.toMatchObject({
      isRedirect: true,
      to: "/",
    });
  });

  it("redirects users without infrastructure permission", async () => {
    fetchMyPermissions.mockResolvedValue({ permissions: ["employees:read"] });

    await expect(infraBeforeLoad()).rejects.toMatchObject({
      isRedirect: true,
      to: "/",
    });
  });

  it("allows contracts users with contract read permission", async () => {
    fetchMyPermissions.mockResolvedValue({ permissions: ["documents:contract:read"] });

    await expect(contractsBeforeLoad()).resolves.toBeUndefined();
    expect(redirect).not.toHaveBeenCalled();
  });

  it("redirects users without contract read permission", async () => {
    fetchMyPermissions.mockResolvedValue({ permissions: ["employees:read"] });

    await expect(contractsBeforeLoad()).rejects.toMatchObject({
      isRedirect: true,
      to: "/",
    });
  });

  it("fails closed when contract permission lookup fails", async () => {
    fetchMyPermissions.mockRejectedValue(new Error("permission service unavailable"));

    await expect(contractsBeforeLoad()).rejects.toMatchObject({
      isRedirect: true,
      to: "/",
    });
  });

  it("redirects personnel hubs without employee read permission", async () => {
    fetchMyPermissions.mockResolvedValue({ permissions: ["ess:attendance"] });

    for (const beforeLoad of personnelHubBeforeLoads) {
      await expect(beforeLoad()).rejects.toMatchObject({ isRedirect: true, to: "/" });
    }
  });

  it("allows personnel hubs with employee read permission", async () => {
    fetchMyPermissions.mockResolvedValue({ permissions: ["employees:read"] });

    for (const beforeLoad of personnelHubBeforeLoads) {
      await expect(beforeLoad()).resolves.toBeUndefined();
    }
  });

  it("allows payroll routes only with payroll read permission", async () => {
    fetchMyPermissions.mockResolvedValue({ permissions: ["payroll:read"] });
    for (const beforeLoad of payrollBeforeLoads) await expect(beforeLoad()).resolves.toBeUndefined();

    fetchMyPermissions.mockResolvedValue({ permissions: ["employees:read"] });
    for (const beforeLoad of payrollBeforeLoads) {
      await expect(beforeLoad()).rejects.toMatchObject({ isRedirect: true, to: "/" });
    }
  });

  it("fails closed for payroll routes when permission lookup fails", async () => {
    fetchMyPermissions.mockRejectedValue(new Error("permission service unavailable"));
    for (const beforeLoad of payrollBeforeLoads) {
      await expect(beforeLoad()).rejects.toMatchObject({ isRedirect: true, to: "/" });
    }
  });

  it("allows salary management only with payroll read permission", async () => {
    fetchMyPermissions.mockResolvedValue({ permissions: ["payroll:read"] });
    await expect(salaryManagementBeforeLoad()).resolves.toBeUndefined();

    fetchMyPermissions.mockResolvedValue({ permissions: ["employees:read"] });
    await expect(salaryManagementBeforeLoad()).rejects.toMatchObject({ isRedirect: true, to: "/" });
  });

  it("allows salary setup with payroll manage permission", async () => {
    fetchMyPermissions.mockResolvedValue({ permissions: ["payroll:manage"] });
    await expect(salaryManagementSetupBeforeLoad()).resolves.toBeUndefined();

    fetchMyPermissions.mockResolvedValue({ permissions: ["payroll:read"] });
    await expect(salaryManagementSetupBeforeLoad()).rejects.toMatchObject({ isRedirect: true, to: "/" });
  });

  it("fails closed when permission lookup fails", async () => {
    fetchMyPermissions.mockRejectedValue(new Error("permission service unavailable"));

    await expect(infraBeforeLoad()).rejects.toMatchObject({
      isRedirect: true,
      to: "/",
    });
  });

  it.each([
    ["assignments", "/assignments"],
    ["working-conditions", "/working-conditions"],
    ["lifecycle", "/lifecycle"],
  ])("redirects legacy %s bookmark to filtered centralized page", (tab, to) => {
    expect(() =>
      (EmployeeDetailRoute.options as RouteOptions).beforeLoad!({
        params: { employeeId: "emp-1" },
        search: { tab },
      }),
    ).toThrow();
    expect(redirect).toHaveBeenCalledWith({
      to,
      search: { employeeId: "emp-1" },
    });
  });

  it("keeps non-centralized profile tabs on employee detail", () => {
    expect(() =>
      (EmployeeDetailRoute.options as RouteOptions).beforeLoad!({
        params: { employeeId: "emp-1" },
        search: { tab: "contracts" },
      }),
    ).not.toThrow();
    expect(redirect).not.toHaveBeenCalled();
  });
});
