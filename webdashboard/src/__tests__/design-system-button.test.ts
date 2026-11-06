import fs from "fs";
import path from "path";
import { describe, it, expect } from "vitest";

// Whitelist of legacy files that still contain native <button> elements.
// No new files should be added to this list. All new UI work must use the project's custom <Button> component.
const LEGACY_VIOLATIONS = new Set([
  "components/approvals/create-ot-request-dialog.tsx",
  "components/common/employee-search.tsx",
  "components/common/error-boundary.tsx",
  "components/common/hierarchy-legend.tsx",
  "components/common/month-year-picker.tsx",
  "components/common/new-version-banner.tsx",
  "components/common/pagination-controls.tsx",
  "components/common/search-box.tsx",
  "components/common/temporal-config-table.tsx",
  "components/debug/debug-toolbar.tsx",
  "components/employees/working-condition-config.tsx",
  "components/features/settings/permission-management-modal.tsx",
  "components/features/settings/role-form-modal.tsx",
  "components/features/settings/role-management-modal.tsx",
  "components/features/dashboard/chart-catalog-sidebar.tsx",
  "components/layout/__tests__/navigation.test.tsx",
  "components/layout/app-shell.tsx",
  "components/layout/auth-language-switcher.tsx",
  "components/layout/navbar.tsx",
  "components/layout/notifications-popover.tsx",
  "components/layout/sidebar.tsx",
  "components/members/invite-member-dialog.tsx",
  "components/members/member-table.tsx",
  "components/payroll/adjustment-drawer.tsx",
  "components/payroll/assign-overtime-policy-components/assign-overtime-policy-fields.tsx",
  "components/payroll/assign-overtime-policy-modal.tsx",
  "components/payroll/batch-calculation-modal.tsx",
  "components/payroll/bonus-policy-manager.tsx",
  "components/payroll/employee-salary-config.tsx",
  "components/payroll/overtime-policy-manager.tsx",
  "components/payroll/overtime-rule-editor.tsx",
  "components/payroll/payroll-detail-drawer.tsx",
  "components/payroll/payroll-rules-manager.tsx",
  "components/payroll/payroll-table.tsx",
  "components/payroll/payslip-modal.tsx",
  "components/payroll/rule-assignment-manager.tsx",
  "components/payroll/rule-assignment/assignment-card.tsx",
  "components/payroll/rule-assignment/components/modal-actions.tsx",
  "components/payroll/rule-assignment/scope-selector.tsx",
  "components/payroll/rule-tier-manager.tsx",
  "components/payroll/salary-base-group-manager.tsx",
  "components/payroll/salary-component-manager.tsx",
  "components/payroll/summary-table/payroll-summary-table-row.tsx",
  "components/payroll/summary-table/selection-banner.tsx",
  "components/payroll/summary-table/table-head.tsx",
  "components/people/employment-phase-manager.tsx",
  "components/shifts/create-shift-modal.tsx",
  "components/shifts/edit-shift-modal.tsx",
  "components/shifts/monthly-roster-grid.tsx",
  "components/shifts/pattern-assignment-form.tsx",
  "components/shifts/schedule-trace-popover.tsx",
  "components/shifts/shift-list.tsx",
  "components/shifts/shift-picker-card.tsx",
  "components/shifts/shift-picker-modal.tsx",
  "components/shifts/shift-picker-parts.tsx",
  "components/shifts/weekly-roster-grid.tsx",
  "components/templates/approval-list-layout.tsx",
  "components/time-tracking/__tests__/punch-card.test.tsx",
  "components/time-tracking/adjustment-request-modal.tsx",
  "components/time-tracking/hold-to-punch-button.tsx",
  "components/time-tracking/punch-card.tsx",
  "hooks/__tests__/use-confirm.test.tsx",
  "pages/__tests__/working-patterns-page.test.tsx",
  "pages/approvals/components/pre-ot-approvals-tab.tsx",
  "pages/approvals/ot-approvals-page.tsx",
  "pages/assignment-rules/components/assignment-rule-row.tsx",
  "pages/attendance/components/adjustments-table/header.tsx",
  "pages/attendance/components/adjustments-table/row.tsx",
  "pages/attendance/components/adjustments-tabs.tsx",
  "pages/attendance/components/attendance-logs-filters.tsx",
  "pages/attendance/components/attendance-logs-tabs.tsx",
  "pages/attendance/components/working-patterns/__tests__/modals.test.tsx",
  "pages/attendance/components/working-patterns/modals.tsx",
  "pages/attendance/components/working-patterns/pattern-card.tsx",
  "pages/ess/components/create-ot-request-modal.tsx",
  "pages/ess/components/my-ot-assignments-list.tsx",
  "pages/ess/ot-page.tsx",
  "pages/leave-requests-page.tsx",
  "pages/monitor/header.tsx",
  "pages/payroll/components/payroll-header.tsx",
  "pages/profile/components/personalization-tab.tsx",
  "pages/profile/components/profile-header.tsx",
  "pages/settings/components/audit-logs/audit-log-pagination.tsx",
  "pages/settings/components/holidays/holiday-form.tsx",
  "pages/settings/components/holidays/holiday-header.tsx",
  "pages/settings/components/holidays/holiday-table.tsx",
  "pages/settings/components/holidays/year-navigation.tsx",
  "pages/settings/components/leave/leave-assignments-tab.tsx",
  "pages/settings/components/leave/leave-settings-tabs.tsx",
  "pages/settings/components/overtime/approval-workflows-tab.tsx",
  "pages/settings/components/overtime/overtime-quotas-tab.tsx",
  "pages/settings/components/overtime/upsert-config-modal.tsx",
  "pages/settings/components/overtime/upsert-workflow-modal.tsx",
  "pages/settings/general-settings.tsx",
  "pages/settings/organization-settings.tsx",
  "pages/settings/overtime-settings.tsx",
  "pages/settings/payroll-settings.tsx",
  "pages/settings/time-settings.tsx",
  "pages/settings/time-settings/reprocess-modal.tsx",
  "pages/shift-swaps/guide.tsx",
  "pages/shift-swaps/list-item.tsx",
  "pages/shifts/__tests__/shifts-components.test.tsx",
  "pages/shifts/header.tsx",
  "pages/shifts/tabs.tsx",
  "pages/system-roles/components/assign-role-modal.tsx",
  "pages/system-roles/components/system-role-card.tsx",
  "pages/working-patterns-editor-page.tsx",
  "pages/workspace/components/permissions-header.tsx",
  "pages/workspace/components/permissions-matrix-subcomponents.tsx",
  "pages/workspace/components/role-card-subcomponents.tsx",
  "pages/workspace/workspace-roles-page.tsx",
  "pages/employees/lifecycle-tab.tsx",
  "pages/settings/settings-layout.tsx",
  "components/custom-fields/__tests__/custom-field-editor.test.tsx",
  "components/custom-fields/custom-fields-filter-popover.tsx",
  "components/custom-fields/custom-field-editor.tsx",
  "components/features/employees/drawer-header.tsx",
  "components/shifts/shifts-section-nav.tsx",
  "components/layout/bottom-nav.tsx",
  "components/layout/bottom-drawer.tsx",
  "components/layout/__tests__/bottom-nav.test.tsx",
  "components/layout/__tests__/bottom-drawer.test.tsx",
]);

function walk(dir: string, excludeUiDir: string): string[] {
  let results: string[] = [];
  const list = fs.readdirSync(dir);
  list.forEach(function (file) {
    const fullPath = path.join(dir, file);
    const stat = fs.statSync(fullPath);
    if (stat.isDirectory()) {
      if (fullPath !== excludeUiDir) {
        results = results.concat(walk(fullPath, excludeUiDir));
      }
    } else if (file.endsWith(".tsx") || file.endsWith(".jsx")) {
      results.push(fullPath);
    }
  });
  return results;
}

describe("UI Standards: Button Compliance", () => {
  it("should not use native <button> elements outside the legacy whitelist and components/ui", () => {
    const sourcePath = path.resolve(__dirname, "..");
    const excludeUiDir = path.join(sourcePath, "components", "ui");
    const files = walk(sourcePath, excludeUiDir);
    const newViolations: string[] = [];

    // Match <button element, but not <button-group or custom elements starting with button-
    const nativeButtonRegex = /<button\b/g;

    files.forEach((file) => {
      const relPath = path.relative(sourcePath, file);
      
      const content = fs.readFileSync(file, "utf8");
      // Clean comments to avoid matches inside comments
      const cleanContent = content
        .replace(/\/\*[\s\S]*?\*\//g, "")
        .replace(/\/\/.*/g, "");

      if (nativeButtonRegex.test(cleanContent)) {
        if (!LEGACY_VIOLATIONS.has(relPath)) {
          newViolations.push(relPath);
        }
      }
    });

    if (newViolations.length > 0) {
      console.error("NEW VIOLATIONS:", newViolations);
    }

    expect(
      newViolations,
      `Found native <button> elements in new or un-whitelisted files. 
Please use the project's custom <Button> component from "@/components/ui/button" instead.`
    ).toHaveLength(0);
  });
});
