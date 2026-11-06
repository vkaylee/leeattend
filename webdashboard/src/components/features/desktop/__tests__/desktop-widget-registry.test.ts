import { describe, it, expect } from "vitest";

import {
  filterDesktopWidgets,
  getDefaultEnabledWidgetIds,
  getDefaultWidgetOrder,
  DESKTOP_WIDGET_REGISTRY,
} from "../desktop-widget-registry";

describe("Desktop Widget Registry", () => {
  it("provides non-empty list of core registered widgets", () => {
    const widgets = DESKTOP_WIDGET_REGISTRY;
    expect(widgets.length).toBeGreaterThanOrEqual(5);

    const ids = widgets.map((w) => w.id);
    expect(ids).toContain("quick-punch");
    expect(ids).toContain("system-health");
    expect(ids).toContain("today-overview");
    expect(ids).toContain("pending-tasks");
    expect(ids).toContain("quick-shortcuts");
  });

  it("returns default enabled widget IDs", () => {
    const enabledIds = getDefaultEnabledWidgetIds();
    expect(enabledIds).toContain("quick-punch");
    expect(enabledIds).toContain("system-health");
  });

  it("returns default ordered widget IDs", () => {
    const order = getDefaultWidgetOrder();
    expect(order.length).toBe(DESKTOP_WIDGET_REGISTRY.length);
    expect(order[0]).toBe("quick-punch");
  });

  it("filters widgets by permissions correctly", () => {
    const mockHasPermission = (perm: string) => perm !== "attendance:logs:read";
    const filtered = filterDesktopWidgets(
      DESKTOP_WIDGET_REGISTRY,
      mockHasPermission,
      [],
    );

    const ids = filtered.map((w) => w.id);
    expect(ids).not.toContain("today-overview");
    expect(ids).toContain("quick-punch");
  });

  it("filters widgets by system roles correctly", () => {
    const customWidget = {
      ...DESKTOP_WIDGET_REGISTRY[0],
      id: "admin-only",
      systemRoles: ["admin"],
    };

    const widgets = [customWidget];
    const nonAdmin = filterDesktopWidgets(widgets, () => true, ["member"]);
    expect(nonAdmin).toHaveLength(0);

    const admin = filterDesktopWidgets(widgets, () => true, ["admin"]);
    expect(admin).toHaveLength(1);
  });
});
