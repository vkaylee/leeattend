import { render, screen } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import ShiftsRulesPage from "../shifts-rules-page";

// Mock child components
vi.mock("../assignment-rules-page", () => ({
  default: ({ hideHeaderTitle }: { hideHeaderTitle?: boolean }) => (
    <div data-testid="assignment-rules-page">
      AssignmentRulesPage (hideHeaderTitle={String(hideHeaderTitle)})
    </div>
  ),
}));

vi.mock("../working-patterns-page", () => ({
  default: ({ hideHeaderTitle }: { hideHeaderTitle?: boolean }) => (
    <div data-testid="working-patterns-page">
      WorkingPatternsPage (hideHeaderTitle={String(hideHeaderTitle)})
    </div>
  ),
}));

// Mock getCurrentWorkspaceId — return workspace by default
const mockGetCurrentWorkspaceId = vi.fn().mockReturnValue("ws-test-123");
vi.mock("@/lib/api", () => ({
  getCurrentWorkspaceId: () => mockGetCurrentWorkspaceId(),
}));

// Mock useSearch return value
const mockUseSearch = vi.fn().mockReturnValue({});
vi.mock("@tanstack/react-router", () => ({
  useSearch: () => mockUseSearch(),
}));

describe("ShiftsRulesPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockGetCurrentWorkspaceId.mockReturnValue("ws-test-123");
    mockUseSearch.mockReturnValue({});
  });

  it("renders WorkingPatternsPage by default when no tab specified", () => {
    mockUseSearch.mockReturnValue({});

    render(<ShiftsRulesPage />);

    expect(screen.getByTestId("working-patterns-page")).toBeDefined();
    expect(screen.queryByTestId("assignment-rules-page")).toBeNull();
  });

  it("renders WorkingPatternsPage when tab is 'patterns'", () => {
    mockUseSearch.mockReturnValue({ tab: "patterns" });

    render(<ShiftsRulesPage />);

    expect(screen.getByTestId("working-patterns-page")).toBeDefined();
    expect(screen.queryByTestId("assignment-rules-page")).toBeNull();
  });

  it("renders AssignmentRulesPage when tab is 'rules'", () => {
    mockUseSearch.mockReturnValue({ tab: "rules" });

    render(<ShiftsRulesPage />);

    expect(screen.getByTestId("assignment-rules-page")).toBeDefined();
    expect(screen.queryByTestId("working-patterns-page")).toBeNull();
  });

  it("renders AssignmentRulesPage when tab is any non-patterns value", () => {
    mockUseSearch.mockReturnValue({ tab: "assignments" });

    render(<ShiftsRulesPage />);

    expect(screen.getByTestId("assignment-rules-page")).toBeDefined();
    expect(screen.queryByTestId("working-patterns-page")).toBeNull();
  });

  it("passes hideHeaderTitle=true to child components", () => {
    mockUseSearch.mockReturnValue({ tab: "patterns" });

    render(<ShiftsRulesPage />);

    expect(screen.getByText(/hideHeaderTitle=true/)).toBeDefined();
  });

  it("shows select workspace message when no workspace selected", () => {
    mockGetCurrentWorkspaceId.mockReturnValue(null);

    render(<ShiftsRulesPage />);

    // The "select workspace" text should appear (resolved by setup.tsx i18n mock to Vietnamese)
    expect(screen.getByText(/Chọn Workspace/i)).toBeDefined();
    // Child pages should NOT render
    expect(screen.queryByTestId("working-patterns-page")).toBeNull();
    expect(screen.queryByTestId("assignment-rules-page")).toBeNull();
  });
});
