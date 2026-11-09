import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import * as api from "../../lib/api";
import AssignmentRulesPage from "../assignment-rules-page";

// Mock TanStack Router
const mockNavigate = vi.fn();
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string | { defaultValue?: string }) => {
      const translations: Record<string, string> = {
        "rules.title": "Assignment Rules",
        "rules.subtitle": "Configure automatic shift assignment",
        "rules.create_btn": "Create Rule",
        "rules.table.priority": "Priority",
        "rules.table.name": "Name",
        "rules.table.pattern": "Pattern",
        "rules.table.conditions": "Conditions",
        "rules.match_all": "Matches all employees",
        "rules.empty_list": "No rules yet",
        "rules.messages.create_success": "Rule created",
        "rules.messages.update_success": "Rule updated",
        "rules.messages.delete_success": "Rule deleted",
        "rules.info_priority": "Rules are evaluated in priority order",
        "rules.edit_title": "Edit Rule",
        "rules.create_title": "Create New Rule",
        "rules.form_subtitle": "Define when this pattern should be applied",
        "rules.form.name": "Rule Name",
        "rules.form.name_placeholder": "e.g. Day shift for operations",
        "rules.form.priority": "Priority",
        "rules.form.pattern": "Working Pattern",
        "rules.form.tags_placeholder": "tag1, tag2",
        "rules.conditions_title": "Conditions",
        "rules.condition_help": "Leave empty to match all employees",
        "patterns.form.select_pattern_placeholder": "Select a pattern",
        "common.cancel": "Cancel",
        "common.create": "Create",
        "common.edit": "Edit",
        "common.delete": "Delete",
        "common.save_changes": "Save Changes",
        "common.confirm_delete": "Are you sure you want to delete?",
        "common.department": "Department",
        "common.all_departments": "All departments",
        "common.tags": "Tags",
        "common.inactive": "Inactive",
        "common.did_you_know": "Did you know?",
      };
      const value = translations[key];
      if (value) return value;
      if (typeof fallback === "string") return fallback;
      if (fallback && typeof fallback === "object" && "defaultValue" in fallback && fallback.defaultValue) return fallback.defaultValue;
      return key;
    },
    i18n: { language: "en" },
  }),
}));

// Mock Combobox
vi.mock("../../components/ui/combobox", () => ({
  Combobox: ({ options, value, onChange, placeholder, "data-testid": testId }: { options: { value: string; label: string }[]; value: string; onChange: (v: string) => void; placeholder?: string; "data-testid"?: string }) => (
    <select
      data-testid={testId}
      onChange={(e) => { onChange(e.target.value); }}
      value={value}
    >
      <option value="">{placeholder}</option>
      {options.map((opt: { value: string; label: string }) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  ),
}));

// Mock APIs
vi.mock("../../lib/api", () => ({
  fetchOrganization: vi.fn().mockResolvedValue({ name: "Test Organization" }),
  fetchLocations: vi.fn().mockResolvedValue([]),
  fetchAssignmentRules: vi.fn(),
  createAssignmentRule: vi.fn(),
  updateAssignmentRule: vi.fn(),
  deleteAssignmentRule: vi.fn(),
  fetchWorkingPatterns: vi.fn(),
  fetchDepartments: vi.fn(),
  AssignmentRule: undefined,
  AssignmentRuleConditions: undefined,
}));

const mockPatterns = [
  { id: "pat-1", name: "Standard 5-day", workspace_id: "ws-1" },
  { id: "pat-2", name: "Rotating 3-shift", workspace_id: "ws-1" },
];

const mockDepartments = [
  { id: "dept-1", name: "Engineering", workspace_id: "ws-1" },
  { id: "dept-2", name: "Operations", workspace_id: "ws-1" },
];

const mockRules: api.AssignmentRule[] = [
  {
    id: "rule-1",
    workspace_id: "ws-1",
    name: "Day Shift Engineers",
    pattern_id: "pat-1",
    priority: 1,
    is_active: true,
    conditions: { department_id: "dept-1" },
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "rule-2",
    workspace_id: "ws-1",
    name: "Default Ops Schedule",
    pattern_id: "pat-2",
    priority: 2,
    is_active: true,
    conditions: { tags: ["ops", "floor"] },
    created_at: "2026-01-02T00:00:00Z",
    updated_at: "2026-01-02T00:00:00Z",
  },
];

describe("AssignmentRulesPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
    vi.mocked(api.fetchAssignmentRules).mockResolvedValue(mockRules);
    vi.mocked(api.fetchWorkingPatterns).mockResolvedValue(mockPatterns as never);
    vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as never);
    vi.mocked(api.createAssignmentRule).mockResolvedValue(mockRules[0]);
    vi.mocked(api.updateAssignmentRule).mockResolvedValue(mockRules[0]);
    vi.mocked(api.deleteAssignmentRule).mockResolvedValue(undefined);
  });

  const renderPage = () =>
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <AssignmentRulesPage />
        </ToastProvider>
      </QueryClientProvider>,
    );

  // ── Render Tests ──────────────────────────────────────────

  it("renders the page title and subtitle", async () => {
    renderPage();
    expect(await screen.findByText("Assignment Rules")).toBeInTheDocument();
    expect(screen.getByText("Configure automatic shift assignment")).toBeInTheDocument();
  });

  it("renders the create button", async () => {
    renderPage();
    expect(await screen.findByText("CREATE RULE")).toBeInTheDocument();
  });

  it("renders rules table with data", async () => {
    renderPage();
    // ResponsiveTable renders both desktop and mobile views
    expect((await screen.findAllByText("Day Shift Engineers")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("Default Ops Schedule").length).toBeGreaterThan(0);
  });

  it("renders priority badges", async () => {
    renderPage();
    expect((await screen.findAllByText("P1")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("P2").length).toBeGreaterThan(0);
  });

  it("renders pattern badges for each rule", async () => {
    renderPage();
    expect((await screen.findAllByText("STANDARD 5-DAY")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("ROTATING 3-SHIFT").length).toBeGreaterThan(0);
  });

  it("renders condition badges (department + tags)", async () => {
    renderPage();
    // Department condition for rule-1 — ResponsiveTable renders both views
    expect((await screen.findAllByText("ENGINEERING")).length).toBeGreaterThan(0);
    // Tags for rule-2
    expect(screen.getAllByText("#OPS").length).toBeGreaterThan(0);
    expect(screen.getAllByText("#FLOOR").length).toBeGreaterThan(0);
  });

  it("renders info box", async () => {
    renderPage();
    expect(await screen.findByText("Did you know?")).toBeInTheDocument();
    expect(screen.getByText("Rules are evaluated in priority order")).toBeInTheDocument();
  });

  it("shows empty state when no rules", async () => {
    vi.mocked(api.fetchAssignmentRules).mockResolvedValue([]);
    renderPage();
    // ResponsiveTable renders both desktop and mobile empty states
    expect((await screen.findAllByText("No rules yet")).length).toBeGreaterThan(0);
  });

  it("does not crash and handles API fetch error gracefully", async () => {
    vi.mocked(api.fetchAssignmentRules).mockRejectedValue(new Error("Network Error"));
    renderPage();
    expect((await screen.findAllByText("No rules yet")).length).toBeGreaterThan(0);
  });

  // ── Navigation Tests ───────────────────────────────────────────

  it("navigates to create rule page when create button is clicked", async () => {
    const user = userEvent.setup();
    renderPage();

    const createButton = await screen.findByText("CREATE RULE");
    await user.click(createButton);

    expect(mockNavigate).toHaveBeenCalledWith({ to: "/shifts/rules/assignment-rules/new" });
  });

  it("navigates to edit rule page when edit button is clicked", async () => {
    const user = userEvent.setup();
    renderPage();

    // ResponsiveTable renders both views
    const names = await screen.findAllByText("Day Shift Engineers");
    expect(names.length).toBeGreaterThan(0);
    const editBtns = screen.getAllByTitle("Edit");
    await user.click(editBtns[0]);

    expect(mockNavigate).toHaveBeenCalledWith({ to: `/shifts/rules/assignment-rules/rule-1/edit` });
  });

  // ── Delete Tests ──────────────────────────────────────────

  it("calls deleteAssignmentRule when delete is confirmed", async () => {
    const user = userEvent.setup();

    renderPage();
    // ResponsiveTable renders both views
    const names = await screen.findAllByText("Day Shift Engineers");
    expect(names.length).toBeGreaterThan(0);

    const deleteBtns = screen.getAllByTitle("Delete");
    await user.click(deleteBtns[0]);

    const confirmButton = await screen.findByText("Confirm");
    await user.click(confirmButton);

    await waitFor(() => {
      expect(api.deleteAssignmentRule).toHaveBeenCalledWith("ws-1", "rule-1");
    });
  });

  it("does not delete when confirm is cancelled", async () => {
    const user = userEvent.setup();

    renderPage();
    // ResponsiveTable renders both views
    const names = await screen.findAllByText("Day Shift Engineers");
    expect(names.length).toBeGreaterThan(0);

    const deleteBtns = screen.getAllByTitle("Delete");
    await user.click(deleteBtns[0]);

    const cancelButton = await screen.findByText("Cancel");
    await user.click(cancelButton);

    expect(api.deleteAssignmentRule).not.toHaveBeenCalled();
  });
});
