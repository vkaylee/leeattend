import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import WorkspaceSettingsPage from "../WorkspaceSettingsPage";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/Toast";
import * as api from "../../lib/api";

// Mock API
vi.mock("../../lib/api", async () => {
  const actual = await vi.importActual("../../lib/api");
  return {
    ...actual,
    fetchWorkspace: vi.fn(),
    updateWorkspace: vi.fn(),
    getCurrentWorkspaceId: vi.fn(),
    fetchWorkspaceMembers: vi.fn(),
  };
});

// Mock hooks
vi.mock("@/hooks/use-workspace", () => ({
  useWorkspaceMembers: vi.fn(() => ({
    members: [],
    isLoading: false,
    addMember: vi.fn(),
    isAdding: false,
  })),
  useMemberRoles: vi.fn(() => ({
    roles: { primary: "member", extra: [] },
    isLoading: false,
    updatePrimaryRole: vi.fn(),
    assignRole: vi.fn(),
    revokeRole: vi.fn(),
  })),
}));

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
    },
  },
});

describe("WorkspaceSettingsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (api.getCurrentWorkspaceId as any).mockReturnValue("ws-123");
    (api.fetchWorkspace as any).mockResolvedValue({
      id: "ws-123",
      name: "Test Workspace",
      work_unit_minutes: 1,
      rounding_policy: "none",
      created_at: new Date().toISOString(),
    });
  });

  it("renders configuration section", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <WorkspaceSettingsPage />
        </ToastProvider>
      </QueryClientProvider>
    );

    expect(screen.getByText("Cấu hình tính công")).toBeDefined();
    expect(screen.getByText("Đơn vị tính công (Phút)")).toBeDefined();
    expect(screen.getByText("Quy tắc làm tròn")).toBeDefined();
  });

  it("allows changing and saving configuration", async () => {
    (api.updateWorkspace as any).mockImplementation(
      (wsId: string, data: any) =>
        new Promise((resolve) => setTimeout(() => resolve({ success: true, ...data }), 100)),
    );

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <WorkspaceSettingsPage />
        </ToastProvider>
      </QueryClientProvider>
    );

    // Wait for data to load and select to be available
    const unitSelect = (await screen.findByTestId("work-unit-select")) as HTMLSelectElement;
    await act(async () => {
      unitSelect.value = "30"; // Explicitly set the value
      fireEvent.change(unitSelect);
    });

    const policySelect = (screen.getByTestId("rounding-policy-select")) as HTMLSelectElement;
    await act(async () => {
      policySelect.value = "floor"; // Explicitly set the value
      fireEvent.change(policySelect);
    });

    const saveButton = screen.getByText("Lưu cấu hình");
    await act(async () => {
      fireEvent.click(saveButton);
    });

    await waitFor(() => {
      expect(api.updateWorkspace).toHaveBeenCalledWith("ws-123", {
        work_unit_minutes: 30,
        rounding_policy: "floor",
      });
    });
  });
});
