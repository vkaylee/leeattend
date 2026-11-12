import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const permissions = new Set<string>();

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({ hasAnyPermission: (...items: string[]) => items.some((item) => permissions.has(item)), isLoading: false }),
}));

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string, options?: { defaultValue?: string }) => options?.defaultValue ?? key }) }));

import { getActiveNavGroup, useFilteredMenuItems } from "@/hooks/use-nav-menu-items";

describe("salary-management navigation", () => {
  it("shows salary management only with payroll read permission", () => {
    permissions.clear();
    const hidden = renderHook(() => useFilteredMenuItems()).result.current;
    expect(hidden.groups.flatMap((group) => group.items).some((item) => item.to === "/salary-management")).toBe(false);

    permissions.add("payroll:read");
    const visible = renderHook(() => useFilteredMenuItems()).result.current;
    expect(visible.groups.flatMap((group) => group.items).find((item) => item.to === "/salary-management")?.testId).toBe("sidebar-link-salary-management");
  });

  it("marks salary management as payroll navigation", () => {
    expect(getActiveNavGroup("/salary-management")).toBe("payroll");
  });
});
