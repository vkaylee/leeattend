import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../lib/api";
import { AssignmentRule } from "../../lib/api/rules";
import AssignmentRuleEditorPage from "../assignment-rule-editor-page";

// Mock TanStack Router
const mockProceed = vi.fn();
const mockReset = vi.fn();
let mockBlockerStatus = "unblocked";
const mockNavigate = vi.fn();
let mockParams = { ruleId: undefined as string | undefined };

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
  useParams: () => mockParams,
  useBlocker: () => ({ status: mockBlockerStatus, proceed: mockProceed, reset: mockReset }),
  useLocation: () => ({ pathname: "/assignment-rules/new", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock API
vi.mock("../../lib/api", () => ({
  fetchAssignmentRules: vi.fn(),
  createAssignmentRule: vi.fn(),
  updateAssignmentRule: vi.fn(),
  fetchWorkingPatterns: vi.fn(),
  fetchDepartments: vi.fn(),
  fetchLocations: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
}));

// Mock Toast
const mockToastSuccess = vi.fn();
const mockToastError = vi.fn();
vi.mock("../../components/ui/toast", () => ({
  useToast: () => ({
    success: mockToastSuccess,
    error: mockToastError,
    warning: vi.fn(),
  }),
}));

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string | { defaultValue?: string }) => {
      const translations: Record<string, string> = {
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
        "patterns.form.unsaved_changes_cancel": "Stay",
        "patterns.form.unsaved_changes_confirm": "Leave",
        "patterns.form.unsaved_changes": "Unsaved changes warning",
        "common.warning": "Warning",
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

const mockPatterns = [
  { id: "pat-1", name: "Standard 5-day", workspace_id: "ws-1" },
];

const mockDepartments = [
  { id: "dept-1", name: "Engineering", workspace_id: "ws-1" },
];

const mockRules: AssignmentRule[] = [
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
];

describe("AssignmentRuleEditorPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    mockBlockerStatus = "unblocked";
    mockParams = { ruleId: undefined };
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("ws-1");
    vi.mocked(api.fetchAssignmentRules).mockResolvedValue(mockRules);
    vi.mocked(api.fetchWorkingPatterns).mockResolvedValue(mockPatterns as never);
    vi.mocked(api.fetchDepartments).mockResolvedValue(mockDepartments as never);
    vi.mocked(api.fetchLocations).mockResolvedValue([]);
    vi.mocked(api.createAssignmentRule).mockResolvedValue(mockRules[0]);
    vi.mocked(api.updateAssignmentRule).mockResolvedValue(mockRules[0]);
  });

  const renderPage = () =>
    render(
      <QueryClientProvider client={queryClient}>
        <AssignmentRuleEditorPage />
      </QueryClientProvider>,
    );

  it("submits the create form with correct payload", async () => {
    const user = userEvent.setup();
    renderPage();

    // Fill rule name
    const nameInput = await screen.findByPlaceholderText("e.g. Day shift for operations");
    await user.type(nameInput, "New Ruleset");

    // Select pattern
    const patternSelect = screen.getByTestId("pattern-combobox");
    await user.selectOptions(patternSelect, "pat-1");

    // Submit
    const submitBtn = screen.getByText("CREATE");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(api.createAssignmentRule).toHaveBeenCalledWith(
        "ws-1",
        expect.objectContaining({
          name: "New Ruleset",
          pattern_id: "pat-1",
          priority: 0,
        }),
      );
    });

    expect(mockToastSuccess).toHaveBeenCalled();
    expect(mockNavigate).toHaveBeenCalledWith({ to: "/shifts/rules", search: { tab: "rules" } });
  });

  it("loads and edits an existing rule", async () => {
    const user = userEvent.setup();
    mockParams = { ruleId: "rule-1" };
    renderPage();

    const nameInput = await screen.findByDisplayValue("Day Shift Engineers");
    await user.clear(nameInput);
    await user.type(nameInput, "Updated Name");

    const submitBtn = screen.getByText("SAVE CHANGES");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(api.updateAssignmentRule).toHaveBeenCalledWith(
        "ws-1",
        "rule-1",
        expect.objectContaining({
          name: "Updated Name",
          pattern_id: "pat-1",
        }),
      );
    });

    expect(mockToastSuccess).toHaveBeenCalled();
  });

  it("navigates back when Cancel is clicked", async () => {
    const user = userEvent.setup();
    renderPage();

    const cancelBtn = await screen.findByText("CANCEL");
    await user.click(cancelBtn);

    expect(mockNavigate).toHaveBeenCalledWith({ to: "/shifts/rules", search: { tab: "rules" } });
  });

  it("triggers ConfirmDialog when navigation is blocked", async () => {
    const user = userEvent.setup();
    mockBlockerStatus = "blocked";
    renderPage();

    // Verify warning dialog is open
    const warningMsg = await screen.findByText("Unsaved changes warning");
    expect(warningMsg).toBeInTheDocument();

    const stayBtn = screen.getByText("Stay");
    const leaveBtn = screen.getByText("Leave");
    expect(stayBtn).toBeInTheDocument();
    expect(leaveBtn).toBeInTheDocument();

    // Click stay
    await user.click(stayBtn);
    expect(mockReset).toHaveBeenCalled();

    // Click leave
    await user.click(leaveBtn);
    expect(mockProceed).toHaveBeenCalled();
  });
});
