import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import UnifiedInboxPage from "../unified-inbox-page";
import * as batchHook from "../hooks/use-batch-approval";
import * as inboxHook from "../hooks/use-unified-inbox";
import { UnifiedApprovalItem } from "../types/unified-approvals";

// ── Mocks ─────────────────────────────────────────────────────────────

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
  }),
}));

const mockBatchApprove = vi.fn().mockResolvedValue({ succeeded: ["1"], failed: [] });
const mockBatchReject = vi.fn().mockResolvedValue({ succeeded: ["1"], failed: [] });

const sampleItems: UnifiedApprovalItem[] = [
  {
    id: "item-1",
    type: "leave",
    status: "pending",
    requester: {
      id: "emp-1",
      name: "Nguyễn Văn A",
      idNumber: "EMP001",
      departmentName: "Kỹ thuật",
      departmentId: "dept-1",
    },
    submittedAt: "2026-11-01T08:00:00Z",
    targetDate: "2026-11-10",
    title: "Đơn xin nghỉ phép",
    subtitle: "Nghỉ phép năm (1 ngày)",
    details: {
      dateStr: "2026-11-10",
      reason: "Việc gia đình",
    },
    impact: {
      departmentName: "Kỹ thuật",
      colleaguesAwayCount: 1,
      colleaguesAwayNames: ["Trần Văn B"],
      isHighImpact: false,
    },
  },
  {
    id: "item-2",
    type: "overtime",
    status: "pending",
    requester: {
      id: "emp-2",
      name: "Lê Thị C",
      idNumber: "EMP002",
      departmentName: "Vận hành",
      departmentId: "dept-2",
    },
    submittedAt: "2026-11-02T08:00:00Z",
    targetDate: "2026-11-02",
    title: "Xác nhận tăng ca (OT)",
    subtitle: "Thời lượng: 120 phút (2.0h)",
    details: {
      dateStr: "2026-11-02",
      reason: "Hỗ trợ sự kiện cuối tuần",
    },
  },
];

describe("UnifiedInboxPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    vi.spyOn(batchHook, "useBatchApproval").mockReturnValue({
      batchApprove: mockBatchApprove,
      batchReject: mockBatchReject,
      isProcessing: false,
    });
  });

  it("renders page header, title, and pending counter badge", () => {
    vi.spyOn(inboxHook, "useUnifiedInbox").mockReturnValue({
      items: sampleItems,
      allItems: sampleItems,
      pendingCounts: { all: 2, leave: 1, overtime: 1, adjustment: 0, shift_swap: 0 },
      isLoading: false,
      filter: { type: "all", status: "pending", searchQuery: "" },
      setFilter: vi.fn(),
      selectedIds: new Set(),
      toggleSelect: vi.fn(),
      selectAll: vi.fn(),
      clearSelection: vi.fn(),
      isAllSelected: false,
      selectedItem: null,
      setSelectedItem: vi.fn(),
      refetchAll: vi.fn().mockResolvedValue(undefined),
    });

    render(<UnifiedInboxPage />);
    expect(screen.getByTestId("unified-inbox-page")).toBeInTheDocument();
    expect(screen.getByText("Hộp thư Phê duyệt")).toBeInTheDocument();
    expect(screen.getByTestId("pending-total-badge")).toHaveTextContent("2 chờ duyệt");
  });

  it("renders approval items with requester names and titles", () => {
    vi.spyOn(inboxHook, "useUnifiedInbox").mockReturnValue({
      items: sampleItems,
      allItems: sampleItems,
      pendingCounts: { all: 2, leave: 1, overtime: 1, adjustment: 0, shift_swap: 0 },
      isLoading: false,
      filter: { type: "all", status: "pending", searchQuery: "" },
      setFilter: vi.fn(),
      selectedIds: new Set(),
      toggleSelect: vi.fn(),
      selectAll: vi.fn(),
      clearSelection: vi.fn(),
      isAllSelected: false,
      selectedItem: null,
      setSelectedItem: vi.fn(),
      refetchAll: vi.fn().mockResolvedValue(undefined),
    });

    render(<UnifiedInboxPage />);
    expect(screen.getByText("Nguyễn Văn A")).toBeInTheDocument();
    expect(screen.getByText("Đơn xin nghỉ phép")).toBeInTheDocument();
    expect(screen.getByText("Lê Thị C")).toBeInTheDocument();
    expect(screen.getByText("Xác nhận tăng ca (OT)")).toBeInTheDocument();
  });

  it("displays empty state when items list is empty", () => {
    vi.spyOn(inboxHook, "useUnifiedInbox").mockReturnValue({
      items: [],
      allItems: [],
      pendingCounts: { all: 0, leave: 0, overtime: 0, adjustment: 0, shift_swap: 0 },
      isLoading: false,
      filter: { type: "all", status: "pending", searchQuery: "" },
      setFilter: vi.fn(),
      selectedIds: new Set(),
      toggleSelect: vi.fn(),
      selectAll: vi.fn(),
      clearSelection: vi.fn(),
      isAllSelected: false,
      selectedItem: null,
      setSelectedItem: vi.fn(),
      refetchAll: vi.fn().mockResolvedValue(undefined),
    });

    render(<UnifiedInboxPage />);
    expect(screen.getByTestId("inbox-empty-state")).toBeInTheDocument();
    expect(screen.getByText("Không có yêu cầu chờ xử lý")).toBeInTheDocument();
  });

  it("renders batch toolbar when items are selected and triggers batch approve", async () => {
    const clearSelectionMock = vi.fn();

    vi.spyOn(inboxHook, "useUnifiedInbox").mockReturnValue({
      items: sampleItems,
      allItems: sampleItems,
      pendingCounts: { all: 2, leave: 1, overtime: 1, adjustment: 0, shift_swap: 0 },
      isLoading: false,
      filter: { type: "all", status: "pending", searchQuery: "" },
      setFilter: vi.fn(),
      selectedIds: new Set(["item-1"]),
      toggleSelect: vi.fn(),
      selectAll: vi.fn(),
      clearSelection: clearSelectionMock,
      isAllSelected: false,
      selectedItem: null,
      setSelectedItem: vi.fn(),
      refetchAll: vi.fn().mockResolvedValue(undefined),
    });

    render(<UnifiedInboxPage />);
    expect(screen.getByTestId("approval-batch-toolbar")).toBeInTheDocument();
    expect(screen.getByText("yêu cầu đã chọn")).toBeInTheDocument();

    const batchApproveBtn = screen.getByTestId("batch-approve-btn");
    fireEvent.click(batchApproveBtn);

    await waitFor(() => {
      expect(mockBatchApprove).toHaveBeenCalledWith([sampleItems[0]]);
      expect(clearSelectionMock).toHaveBeenCalled();
    });
  });

  it("opens rejection reason modal when clicking batch reject and submits with reason", async () => {
    vi.spyOn(inboxHook, "useUnifiedInbox").mockReturnValue({
      items: sampleItems,
      allItems: sampleItems,
      pendingCounts: { all: 2, leave: 1, overtime: 1, adjustment: 0, shift_swap: 0 },
      isLoading: false,
      filter: { type: "all", status: "pending", searchQuery: "" },
      setFilter: vi.fn(),
      selectedIds: new Set(["item-1"]),
      toggleSelect: vi.fn(),
      selectAll: vi.fn(),
      clearSelection: vi.fn(),
      isAllSelected: false,
      selectedItem: null,
      setSelectedItem: vi.fn(),
      refetchAll: vi.fn().mockResolvedValue(undefined),
    });

    render(<UnifiedInboxPage />);
    const batchRejectBtn = screen.getByTestId("batch-reject-btn");
    fireEvent.click(batchRejectBtn);

    expect(screen.getByTestId("rejection-reason-modal")).toBeInTheDocument();

    const textarea = screen.getByTestId("rejection-reason-textarea");
    fireEvent.change(textarea, { target: { value: "Thiếu người trực ca" } });

    const confirmBtn = screen.getByTestId("rejection-confirm-btn");
    fireEvent.click(confirmBtn);

    await waitFor(() => {
      expect(mockBatchReject).toHaveBeenCalledWith([sampleItems[0]], "Thiếu người trực ca");
    });
  });
});
