import React, { act } from "react";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import DebugWorkspaceTools from "../DebugWorkspaceTools";
import * as api from "../../lib/api";
import { vi, describe, it, expect, beforeEach } from "vitest";

vi.mock("../../lib/api", () => ({
  getCurrentWorkspaceId: vi.fn(),
  clearWorkspaceData: vi.fn(),
}));

vi.mock("../Toast", () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}));

describe("DebugWorkspaceTools", () => {
  const mockWsId = "workspace-123";

  beforeEach(() => {
    vi.clearAllMocks();
    (api.getCurrentWorkspaceId as any).mockReturnValue(mockWsId);
    // Mock window.confirm to always return true
    vi.stubGlobal("confirm", vi.fn(() => true));
  });

  it("renders all clear buttons", () => {
    render(<DebugWorkspaceTools />);
    expect(screen.getByText("Chấm công")).toBeInTheDocument();
    expect(screen.getByText("Lịch trình")).toBeInTheDocument();
    expect(screen.getByText("Nghỉ phép")).toBeInTheDocument();
    expect(screen.getByText("Lương & Hợp đồng")).toBeInTheDocument();
    expect(screen.getByText("FULL RESET (Cẩn thận!)")).toBeInTheDocument();
  });

  it("calls clearWorkspaceData with correct group when a button is clicked", async () => {
    (api.clearWorkspaceData as any).mockResolvedValue(undefined);
    render(<DebugWorkspaceTools />);

    const attendanceBtn = screen.getByText("Chấm công");
    await act(async () => {
      fireEvent.click(attendanceBtn);
    });

    expect(window.confirm).toHaveBeenCalled();
    expect(api.clearWorkspaceData).toHaveBeenCalledWith(mockWsId, "attendance");

    await waitFor(() => {
        expect(screen.queryByText("Đang xóa...")).not.toBeInTheDocument();
    });
  });

  it("does not call clearWorkspaceData if confirm is cancelled", async () => {
    vi.stubGlobal("confirm", vi.fn(() => false));
    render(<DebugWorkspaceTools />);

    const attendanceBtn = screen.getByText("Chấm công");
    await act(async () => {
      fireEvent.click(attendanceBtn);
    });

    expect(window.confirm).toHaveBeenCalled();
    expect(api.clearWorkspaceData).not.toHaveBeenCalled();
  });

  it("calls clearWorkspaceData with full_reset when reset button is clicked", async () => {
    (api.clearWorkspaceData as any).mockResolvedValue(undefined);
    render(<DebugWorkspaceTools />);

    const resetBtn = screen.getByText("FULL RESET (Cẩn thận!)");
    await act(async () => {
      fireEvent.click(resetBtn);
    });

    expect(api.clearWorkspaceData).toHaveBeenCalledWith(mockWsId, "full_reset");
  });
});
