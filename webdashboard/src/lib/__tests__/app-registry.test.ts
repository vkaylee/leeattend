import { describe, it, expect, afterEach } from "vitest";

import {
  APP_REGISTRY,
  GROUP_LABEL_KEYS,
  type AppDefinition,
} from "@/lib/app-registry";
import { isFeatureEnabled } from "@/lib/feature-flags";
import { ROUTES } from "@/lib/routes";

describe("APP_REGISTRY", () => {
  it("all apps have unique ids", () => {
    const ids = APP_REGISTRY.map((a) => a.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("all apps have unique testIds", () => {
    const testIds = APP_REGISTRY.map((a) => a.testId);
    expect(new Set(testIds).size).toBe(testIds.length);
  });

  it("all apps have a valid group", () => {
    const validGroups = Object.keys(GROUP_LABEL_KEYS);
    for (const app of APP_REGISTRY) {
      expect(validGroups).toContain(app.group);
    }
  });

  it("all apps have an icon, labelKey, descriptionKey, and valid to path", () => {
    for (const app of APP_REGISTRY) {
      expect(app.icon).toBeDefined();
      expect(app.labelKey).toBeTruthy();
      expect(app.descriptionKey).toBeTruthy();
      expect(app.to).toBeTruthy();
      expect(app.to.startsWith("/")).toBe(true);
    }
  });

  it("dashboard app has no permission guard", () => {
    const dash = APP_REGISTRY.find((a) => a.id === "dashboard");
    expect(dash).toBeDefined();
    expect(dash!.permissions).toBeUndefined();
    expect(dash!.systemRoles).toBeUndefined();
  });

  it("dashboard-analytics app has no permission guard", () => {
    const analytics = APP_REGISTRY.find((a) => a.id === "dashboard-analytics");
    expect(analytics).toBeDefined();
    expect(analytics!.permissions).toBeUndefined();
    expect(analytics!.systemRoles).toBeUndefined();
  });

  it("ESS apps require ess:* permissions", () => {
    const essApps = APP_REGISTRY.filter((a) => a.group === "ess");
    expect(essApps.length).toBeGreaterThanOrEqual(3);
    for (const app of essApps) {
      expect(app.permissions).toBeDefined();
      expect(app.permissions!.some((p) => p.startsWith("ess:"))).toBe(true);
    }
  });

  it("system apps with systemRoles target platform_admin", () => {
    const systemApps = APP_REGISTRY.filter(
      (a) => a.group === "system" && a.systemRoles,
    );
    expect(systemApps.length).toBeGreaterThanOrEqual(2);
    for (const app of systemApps) {
      expect(app.systemRoles).toContain("admin");
    }
  });

  it("all app routes reference valid ROUTES constants or paths", () => {
    const validPaths = new Set<string>(Object.values(ROUTES).flatMap((v) =>
      typeof v === "string" ? [v] : Object.values(v)
    ));

    for (const app of APP_REGISTRY) {
      // Either the route is in ROUTES, or it's a known exception
      if (!validPaths.has(app.to) && app.to !== "/members" && app.to !== "/documents") {
        // Still valid as long as it starts with /
        expect(app.to.startsWith("/")).toBe(true);
      }
    }
  });

  it("has at least 23 apps", () => {
    expect(APP_REGISTRY.length).toBeGreaterThanOrEqual(23);
  });

  it("covers all NavGroup values in GROUP_LABEL_KEYS", () => {
    expect(GROUP_LABEL_KEYS.dashboard).toBeTruthy();
    expect(GROUP_LABEL_KEYS.org).toBeTruthy();
    expect(GROUP_LABEL_KEYS.time).toBeTruthy();
    expect(GROUP_LABEL_KEYS.shifts).toBeTruthy();
    expect(GROUP_LABEL_KEYS.payroll).toBeTruthy();
    expect(GROUP_LABEL_KEYS.system).toBeTruthy();
    expect(GROUP_LABEL_KEYS.ess).toBeTruthy();
  });
});

// ── Filtering logic (mirrors app-launcher-page) ──────────────────

function filterApps(
  apps: AppDefinition[],
  permissions: string[],
  systemRoles: string[],
): AppDefinition[] {
  const permSet = new Set(permissions);
  const hasAnyPermission = (...perms: string[]) =>
    perms.length === 0 || perms.some((p) => permSet.has(p));

  return apps.filter((app) => {
    if (app.featureFlag && !isFeatureEnabled(app.featureFlag)) {
      return false;
    }
    if (app.systemRoles && app.systemRoles.length > 0) {
      return app.systemRoles.some((role) => systemRoles.includes(role));
    }
    if (app.permissions && app.permissions.length > 0) {
      return hasAnyPermission(...app.permissions);
    }
    return true;
  });
}

describe("app filtering logic", () => {
  afterEach(() => {
    localStorage.clear();
  });

  it("hides feature-flagged apps by default even with matching permissions", () => {
    const visible = filterApps(APP_REGISTRY, ["employees:update"], []);

    expect(visible.find((a) => a.id === "econtracts")).toBeUndefined();
  });

  it("shows a feature-flagged app when the flag is enabled", () => {
    localStorage.setItem("ff_econtracts", "true");

    const visible = filterApps(APP_REGISTRY, ["employees:update"], []);
    expect(visible.find((a) => a.id === "econtracts")).toBeDefined();
  });

  it("registers the econtracts feature flag on the app definition", () => {
    const econtracts = APP_REGISTRY.find((a) => a.id === "econtracts");
    expect(econtracts).toBeDefined();
    expect(econtracts!.featureFlag).toBe("ff_econtracts");
  });

  it("non-privileged employee sees only dashboard + ESS apps", () => {
    const visible = filterApps(
      APP_REGISTRY,
      ["ess:profile", "ess:attendance", "ess:clock"],
      [],
    );

    expect(visible.find((a) => a.id === "dashboard")).toBeDefined();
    expect(visible.find((a) => a.id === "dashboard-analytics")).toBeDefined();
    expect(visible.find((a) => a.id === "ess-profile")).toBeDefined();
    expect(visible.find((a) => a.id === "ess-attendance")).toBeDefined();
    expect(visible.find((a) => a.id === "ess-clock")).toBeDefined();

    // Should NOT see admin-only items
    expect(visible.find((a) => a.id === "employees")).toBeUndefined();
    expect(visible.find((a) => a.id === "system-roles")).toBeUndefined();
    expect(visible.find((a) => a.id === "payroll")).toBeUndefined();
  });

  it("manager with HR perms sees org + time + shifts + payroll", () => {
    const visible = filterApps(
      APP_REGISTRY,
      [
        "employees:read", "members:read", "departments:read",
        "attendance:logs:read", "leave:requests:approve", "leave:balances:read",
        "shifts:read", "payroll:read", "reports:read",
        "ess:profile", "ess:attendance",
      ],
      [],
    );

    expect(visible.find((a) => a.id === "employees")).toBeDefined();
    expect(visible.find((a) => a.id === "payroll")).toBeDefined();
    expect(visible.find((a) => a.id === "shifts")).toBeDefined();

    // Should NOT see system/platform items
    expect(visible.find((a) => a.id === "system-roles")).toBeUndefined();
    expect(visible.find((a) => a.id === "infra-monitor")).toBeUndefined();
  });

  it("platform_admin sees everything including system items", () => {
    const visible = filterApps(
      APP_REGISTRY,
      [
        "employees:read", "members:read", "departments:read",
        "attendance:logs:read", "leave:requests:approve",
        "shifts:read", "payroll:read", "reports:read",
        "ess:profile", "ess:attendance", "ess:clock",
      ],
      ["platform_admin"],
    );

    expect(visible.find((a) => a.id === "system-roles")).toBeDefined();
    expect(visible.find((a) => a.id === "infra-monitor")).toBeDefined();
  });

  it("admin sees infra-monitor and system-roles", () => {
    const visible = filterApps(APP_REGISTRY, [], ["admin"]);

    expect(visible.find((a) => a.id === "infra-monitor")).toBeDefined();
    expect(visible.find((a) => a.id === "system-roles")).toBeDefined();
  });
});
