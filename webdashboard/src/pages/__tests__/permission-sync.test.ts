import { describe, expect, it } from "vitest";

import en from "../../i18n/locales/en/permissions.json";
import vi from "../../i18n/locales/vi/permissions.json";
import zh from "../../i18n/locales/zh/permissions.json";
import { permissionGroupIcons } from "../../lib/permission-groups";

/**
 * Permission IDs are intentionally not mirrored in the frontend.
 * The backend catalog endpoint is the runtime authority, and its generated
 * OpenAPI contract is consumed by the matrix and permission pickers.
 */
describe("Permission catalog presentation", () => {
  it("provides an icon for every backend permission group", () => {
    expect(Object.keys(permissionGroupIcons).sort()).toEqual([
      "attendance",
      "departments",
      "documents",
      "employees",
      "employment_phases",
      "ess",
      "holidays",
      "infra",
      "leave",
      "members",
      "organization",
      "overtime",
      "payroll",
      "recruitment",
      "reports",
      "security",
      "shifts",
    ]);
  });

  it("keeps the production-blocking permission translated in every locale", () => {
    const permissionId = "overtime:request:create_on_behalf";
    for (const locale of [en, vi, zh]) {
      expect(locale.items[permissionId].name).toBeTruthy();
      expect(locale.items[permissionId].desc).toBeTruthy();
    }
  });

  it("does not maintain a duplicate frontend permission ID catalog", () => {
    expect(Object.values(permissionGroupIcons)).toHaveLength(17);
  });
});
