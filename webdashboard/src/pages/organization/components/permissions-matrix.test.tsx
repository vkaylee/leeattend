import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, within } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { PermissionsMatrix } from "./permissions-matrix";

import type { PermissionCatalogItem, OrganizationRole } from "../../../lib/api/types";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string; count?: number; total?: number }) => {
      if (key === "permissions.mobilePermissionCount") {
        return `${options?.count ?? 0} of ${options?.total ?? 0} enabled`;
      }
      if (key === "permissions.employees.read.desc") {
        return "Can view employees";
      }
      return options?.defaultValue ?? key;
    },
  }),
}));

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

function Wrapper({ children }: { children: React.ReactNode }) {
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

const roles: OrganizationRole[] = [
  {
    id: "role-system",
    name: "admin",
    color: "slate",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    is_system: true,
    scope_type: "global",
  },
  {
    id: "role-custom",
    name: "manager",
    color: "blue",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    is_system: false,
    scope_type: "scoped",
  },
];

const catalog: PermissionCatalogItem[] = [
  { id: "employees:read", group: "employees", label_key: "permissions.employees.read.name" },
  { id: "employees:write", group: "employees", label_key: "permissions.employees.write.name" },
  { id: "reports:read", group: "reports", label_key: "permissions.reports.read.name" },
];

const defaultProps = {
  canScrollLeft: false,
  canScrollRight: true,
  handleScroll: vi.fn(),
  hasManagePermission: true,
  isCatalogError: false,
  isCatalogLoading: false,
  isMatrixLoading: false,
  isRolesLoading: false,
  permissions: {
    "role-system": ["employees:read"],
    "role-custom": ["reports:read"],
  },
  scrollBy: vi.fn(),
  scrollContainerRef: { current: null },
  togglePermission: vi.fn(),
  visibleRoles: roles,
};

describe("PermissionsMatrix mobile view", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders every role and permission in the mobile accordion", () => {
    render(<PermissionsMatrix {...defaultProps} catalog={catalog} />, { wrapper: Wrapper });

    const mobile = screen.getByTestId("permissions-mobile");
    expect(mobile.querySelectorAll('[data-testid^="permissions-mobile-role-"]')).toHaveLength(2);
    expect(mobile.querySelectorAll('[data-testid^="permissions-mobile-permission-"]')).toHaveLength(roles.length * catalog.length);

    for (const role of roles) {
      const section = screen.getByTestId(`permissions-mobile-role-${role.id}`);
      const toggle = within(section).getByRole("button");
      if (toggle.getAttribute("aria-expanded") === "false") fireEvent.click(toggle);
    }

    expect(within(mobile).getAllByRole("checkbox")).toHaveLength(roles.length * catalog.length);
  });

  it("renders translated permission descriptions on mobile", () => {
    render(<PermissionsMatrix {...defaultProps} catalog={catalog} />, { wrapper: Wrapper });

    expect(
      within(screen.getByTestId("permissions-mobile-role-role-system")).getByText("Can view employees"),
    ).toBeInTheDocument();
  });

  it("toggles a permission for the selected mobile role", () => {
    render(<PermissionsMatrix {...defaultProps} catalog={catalog} />, { wrapper: Wrapper });

    const role = roles[1];
    const permission = catalog[0];
    fireEvent.click(screen.getByTestId(`permissions-mobile-toggle-${role.id}-${permission.id}`));

    expect(defaultProps.togglePermission).toHaveBeenCalledWith(role, permission.id);
  });
});
