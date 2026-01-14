import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import DepartmentsPage from "../DepartmentsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "../../lib/api";
import React from "react";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchDepartments: vi.fn(),
  createDepartment: vi.fn(),
  updateDepartment: vi.fn(),
  deleteDepartment: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  fetchUsers: vi.fn(),
}));

describe("DepartmentsPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
    });
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    (api.fetchDepartments as any).mockResolvedValue([
      { id: "dept-1", name: "Engineering", description: "Tech stuff", manager_id: "user-1" }
    ]);
    (api.fetchUsers as any).mockResolvedValue([
      { id: "user-1", full_name: "Manager One", email: "manager1@example.com" }
    ]);
  });

  it("renders without workspace selected", () => {
    (api.getCurrentWorkspaceId as any).mockReturnValue(null);
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <DepartmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    expect(screen.getByText("Vui lòng chọn Workspace")).toBeDefined();
  });

  it("renders departments list", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <DepartmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    expect(await screen.findByText("Engineering")).toBeDefined();
    expect(screen.getByText("Tech stuff")).toBeDefined();
    expect(screen.getByText("Manager One")).toBeDefined();
  });

  it("handles creating a new department", async () => {
    const user = userEvent.setup();
    (api.createDepartment as any).mockResolvedValue({ id: "new-dept", name: "HR" });

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <DepartmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    await user.click(screen.getByText("Thêm phòng ban"));

    const nameInput = await screen.findByLabelText("Tên phòng ban");
    await user.type(nameInput, "Human Resources");
    
    const descInput = screen.getByLabelText("Mô tả");
    await user.type(descInput, "People stuff");

    await user.click(screen.getByText("Tạo phòng ban"));

    await waitFor(() => {
      expect(api.createDepartment).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
        name: "Human Resources",
        description: "People stuff"
      }));
    });
  });

  it("handles updating a department", async () => {
    const user = userEvent.setup();
    (api.updateDepartment as any).mockResolvedValue({ id: "dept-1", name: "Engineering Updated" });

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <DepartmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    // Find the edit button (it's in the group that shows on hover, but in test it's always there)
    // There's only one department in beforeEach
    const editButtons = await screen.findAllByRole("button");
    // The edit button has an Edit2 icon. We might need a better way to find it.
    // Let's use the icon class or just find by index if we know the structure.
    // In our component, it's the first button in the absolute positioned div.
    
    // Actually, let's find it by clicking the one that has the Edit2 icon logic if possible, 
    // or just rely on the fact that it's the first button in the card actions.
    
    // Re-reading component:
    /*
    <button 
      onClick={() => { setEditingDept(dept); setIsModalOpen(true); }}
      className="..."
    >
      <Edit2 className="w-4 h-4" />
    </button>
    */
    
    await screen.findByText("Engineering");

    // In our component, it's the first button in the absolute positioned div.
    const editBtn = document.querySelector(".lucide-pen")?.parentElement;
    if (!editBtn) throw new Error("Edit button not found");
    
    await user.click(editBtn);

    const nameInput = await screen.findByLabelText("Tên phòng ban");
    await user.clear(nameInput);
    await user.type(nameInput, "Software Engineering");

    await user.click(screen.getByText("Cập nhật"));

    await waitFor(() => {
      expect(api.updateDepartment).toHaveBeenCalledWith("test-ws-id", "dept-1", expect.objectContaining({
        name: "Software Engineering"
      }));
    });
  });

  it("handles deleting a department", async () => {
    const user = userEvent.setup();
    (api.deleteDepartment as any).mockResolvedValue({});
    vi.spyOn(window, "confirm").mockReturnValue(true);

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <DepartmentsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    await screen.findByText("Engineering");
    
    const deleteBtn = document.querySelector(".lucide-trash-2")?.parentElement;
    if (!deleteBtn) throw new Error("Delete button not found");
    
    await user.click(deleteBtn);

    expect(window.confirm).toHaveBeenCalled();
    await waitFor(() => {
      expect(api.deleteDepartment).toHaveBeenCalledWith("test-ws-id", "dept-1");
    });
  });
});
