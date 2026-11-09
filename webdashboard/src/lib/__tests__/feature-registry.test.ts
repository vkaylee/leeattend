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

  it("includes app and action categories without child Settings Hub entries", () => {
    const allFeatures = getAllFeatures();
    const actions = allFeatures.filter((f) => f.category === "action");

    expect(actions.length).toBeGreaterThan(0);
    expect(allFeatures.some((f) => f.id === "set-general")).toBe(false);
  });

  it("applies permission filters correctly", () => {
    const allFeatures = getAllFeatures();

    const noPerms = filterFeatures(allFeatures, () => false);
    expect(noPerms.some((f) => f.id === "app-dashboard")).toBe(true);
    expect(noPerms.some((f) => f.id === "app-employees")).toBe(false);
    expect(noPerms.some((f) => f.id === "app-infra-monitor")).toBe(false);

    const hrPerms = filterFeatures(allFeatures, (perm) => perm === "employees:read");
    expect(hrPerms.some((f) => f.id === "app-employees")).toBe(true);
    expect(hrPerms.some((f) => f.id === "act-new-employee")).toBe(true);

    const adminPerms = filterFeatures(
      allFeatures,
      (perm) => perm === "infra:monitor" || perm === "organization:settings:read" || perm === "members:roles:manage" || perm === "audit_logs:read",
    );
    expect(adminPerms.some((f) => f.id === "app-infra-monitor")).toBe(true);
    expect(adminPerms.some((f) => f.id === "app-settings")).toBe(true);
    expect(adminPerms.some((f) => f.id === "app-audit-logs")).toBe(true);
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

});
