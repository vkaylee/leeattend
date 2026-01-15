import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import RoleManagementModal from "../RoleManagementModal";
import { useMemberRoles } from "@/hooks/use-workspace";

// Mock the hook
vi.mock("@/hooks/use-workspace", () => ({
  useMemberRoles: vi.fn(),
  useWorkspace: vi.fn(() => ({
    workspace: {
      date_format: "DD/MM/YYYY",
      time_format: "24h",
    },
    isLoading: false,
  })),
}));

describe("RoleManagementModal", () => {
  const mockUpdatePrimaryRole = vi.fn();
  const mockAssignRole = vi.fn();
  const mockRevokeRole = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useMemberRoles as any).mockReturnValue({
      roles: { primary: "member", extra: [] },
      isLoading: false,
      updatePrimaryRole: mockUpdatePrimaryRole,
      assignRole: mockAssignRole,
      revokeRole: mockRevokeRole,
    });
    
    // Mock window.confirm and alert
    vi.stubGlobal("confirm", vi.fn(() => true));
    vi.stubGlobal("alert", vi.fn());
  });

  it("renders correctly when open", () => {
    render(<RoleManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);
    expect(screen.getByText("Quản lý quyền hạn")).toBeDefined();
    expect(screen.getByText("Quyền chính (Vĩnh viễn)")).toBeDefined();
  });

  it("returns null when closed", () => {
    const { container } = render(<RoleManagementModal userId="user-1" isOpen={false} onClose={() => {}} />);
    expect(container.firstChild).toBeNull();
  });

  it("shows loading state", () => {
    (useMemberRoles as any).mockReturnValue({
      isLoading: true,
    });
    render(<RoleManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);
    expect(screen.getByText("Đang tải dữ liệu...")).toBeDefined();
  });

  it("handles updating primary role", async () => {
    const user = userEvent.setup();
    render(<RoleManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);
    
    const primarySelect = screen.getByLabelText("Quyền chính (Vĩnh viễn)");
    await user.selectOptions(primarySelect, "admin");

    expect(window.confirm).toHaveBeenCalled();
    await waitFor(() => {
      expect(mockUpdatePrimaryRole).toHaveBeenCalledWith("admin");
    });
  });

  it("handles assigning extra role", async () => {
    const user = userEvent.setup();
    render(<RoleManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);
    
    const roleSelect = screen.getByLabelText("Vai trò gán");
    await user.selectOptions(roleSelect, "admin");

    const dateInput = screen.getByLabelText("Ngày hết hạn");
    fireEvent.change(dateInput, { target: { value: "2026-12-31T23:59" } });

    await user.click(screen.getByText("Gán quyền"));

    await waitFor(() => {
      expect(mockAssignRole).toHaveBeenCalledWith(expect.objectContaining({
        role: "admin",
        expiresAt: expect.any(String)
      }));
    });
  });

  it("handles revoking extra role", async () => {
    (useMemberRoles as any).mockReturnValue({
      roles: { 
        primary: "member", 
        extra: [{ id: "role-1", role: "admin", expires_at: "2026-12-31T23:59:00Z" }] 
      },
      isLoading: false,
      revokeRole: mockRevokeRole,
    });

    const user = userEvent.setup();
    render(<RoleManagementModal userId="user-1" isOpen={true} onClose={() => {}} />);
    
    expect(screen.getByText("admin")).toBeDefined();
    
    const revokeBtn = screen.getByTitle("Gỡ bỏ");
    await user.click(revokeBtn);
    
    expect(window.confirm).toHaveBeenCalled();
    expect(mockRevokeRole).toHaveBeenCalledWith("role-1");
  });
});