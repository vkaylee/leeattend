import { describe, it, expect } from "vitest";

import {
  getAllFeatures,
  filterFeatures,
  buildFeatureSearchIndex,
  appToFeature,
  type FeatureItem,
} from "@/lib/feature-registry";
import { APP_REGISTRY } from "@/lib/app-registry";

describe("Feature Registry & Search Indexer", () => {
  it("converts all apps from APP_REGISTRY to feature items", () => {
    const allFeatures = getAllFeatures();
    const appFeatures = allFeatures.filter((f) => f.category === "app");
    expect(appFeatures.length).toBe(APP_REGISTRY.length);
  });

  it("includes action and setting categories", () => {
    const allFeatures = getAllFeatures();
    const actions = allFeatures.filter((f) => f.category === "action");
    const settings = allFeatures.filter((f) => f.category === "setting");

    expect(actions.length).toBeGreaterThan(0);
    expect(settings.length).toBeGreaterThan(0);
  });

  it("applies permission filters correctly", () => {
    const allFeatures = getAllFeatures();

    // User with no permissions and no system roles
    const noPerms = filterFeatures(allFeatures, () => false, []);
    // Dashboard and features with no permissions should still be visible
    expect(noPerms.some((f) => f.id === "app-dashboard")).toBe(true);
    // Protected app should be hidden
    expect(noPerms.some((f) => f.id === "app-employees")).toBe(false);

    // User with employees:read
    const hrPerms = filterFeatures(
      allFeatures,
      (perm) => perm === "employees:read",
      [],
    );
    expect(hrPerms.some((f) => f.id === "app-employees")).toBe(true);
    expect(hrPerms.some((f) => f.id === "act-new-employee")).toBe(true);
    // Setting for roles should still be hidden
    expect(hrPerms.some((f) => f.id === "set-roles")).toBe(false);
  });

  it("applies systemRoles filters correctly", () => {
    const allFeatures = getAllFeatures();

    // Regular employee without admin role
    const regular = filterFeatures(allFeatures, () => true, []);
    expect(regular.some((f) => f.id === "app-infra-monitor")).toBe(false);

    // Admin role
    const admin = filterFeatures(allFeatures, () => true, ["admin"]);
    expect(admin.some((f) => f.id === "app-infra-monitor")).toBe(true);
    expect(admin.some((f) => f.id === "set-infra-monitor")).toBe(true);
  });

  it("builds search index with and without Vietnamese diacritics", () => {
    const testFeature: FeatureItem = {
      id: "test-feature",
      category: "action",
      titleFallback: "Bấm giờ điểm danh",
      descriptionFallback: "Chấm công trực tuyến",
      to: "/ess/clock",
      icon: () => null,
      gradient: "from-cyan-500 to-emerald-600",
      keywords: ["clock in", "check in", "diem danh"],
      testId: "test-feature",
    };

    const index = buildFeatureSearchIndex(testFeature);

    // Diacritic-preserved
    expect(index).toContain("bấm giờ điểm danh");
    expect(index).toContain("chấm công trực tuyến");

    // Diacritic-removed
    expect(index).toContain("bam gio diem danh");
    expect(index).toContain("cham cong truc tuyen");

    // Route and category
    expect(index).toContain("/ess/clock");
    expect(index).toContain("action");
  });

  it("assigns distinct gradients to apps", () => {
    const sampleApp = APP_REGISTRY.find((a) => a.id === "dashboard")!;
    const feature = appToFeature(sampleApp);
    expect(feature.gradient).toContain("from-indigo-500");
  });

  it("covers all settings routes and key system management features", () => {
    const allFeatures = getAllFeatures();
    const settingsRoutes = allFeatures
      .filter((f) => f.category === "setting")
      .map((f) => f.to);

    // All major settings routes must be present
    expect(settingsRoutes).toContain("/workspace/settings/general");
    expect(settingsRoutes).toContain("/workspace/settings/time");
    expect(settingsRoutes).toContain("/workspace/settings/overtime");
    expect(settingsRoutes).toContain("/workspace/settings/leave");
    expect(settingsRoutes).toContain("/workspace/settings/holidays");
    expect(settingsRoutes).toContain("/workspace/settings/workflows");
    expect(settingsRoutes).toContain("/workspace/settings/locations");
    expect(settingsRoutes).toContain("/workspace/settings/roles");
    expect(settingsRoutes).toContain("/workspace/settings/permissions");
    expect(settingsRoutes).toContain("/workspace/settings/organization");
    expect(settingsRoutes).toContain("/workspace/settings/payroll");
    expect(settingsRoutes).toContain("/workspace/settings/billing");
    expect(settingsRoutes).toContain("/workspace/settings/custom-fields");
    expect(settingsRoutes).toContain("/workspace/settings/lifecycle-templates");
    expect(settingsRoutes).toContain("/workspace/settings/condition-types");
    expect(settingsRoutes).toContain("/workspace/settings/storage");
    expect(settingsRoutes).toContain("/workspace/settings/econtracts");
    expect(settingsRoutes).toContain("/workspace/settings/audit-logs");
    expect(settingsRoutes).toContain("/system/smtp");
    expect(settingsRoutes).toContain("/system/tenants");
    expect(settingsRoutes).toContain("/system/roles");
    expect(settingsRoutes).toContain("/infra/monitor");
  });
});
