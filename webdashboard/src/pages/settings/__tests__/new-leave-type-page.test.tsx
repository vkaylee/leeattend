import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import { ApiError } from "@/lib/api/errors";
import * as api from "@/lib/api/leaves";

import { NewLeaveTypePage } from "../new-leave-type-page";

const mockNavigate = vi.fn();
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
  Link: ({ children, to, search }: { children: React.ReactNode; to: string; search?: Record<string, string> }) => (
    <a href={to} onClick={(e) => { e.preventDefault(); mockNavigate({ to, search }); }}>
      {children}
    </a>
  ),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string, options?: Record<string, unknown>) => {
      if (options?.name) {
        return (fallback || key).replace("{{name}}", String(options.name));
      }
      return fallback || key;
    },
  }),
}));

vi.mock("@/lib/api/leaves", () => ({
  fetchLeaveTypes: vi.fn(),
  createLeaveType: vi.fn(),
  restoreLeaveType: vi.fn(),
  updateLeaveTypeConfig: vi.fn(),
  deleteLeaveType: vi.fn(),
}));

const mockToast = { success: vi.fn(), error: vi.fn() };
vi.mock("@/components/ui/toast", () => ({
  useToast: () => mockToast,
}));

function renderPage() {
  const testQueryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });
  return render(
    <QueryClientProvider client={testQueryClient}>
      <NewLeaveTypePage />
    </QueryClientProvider>
  );
}

describe("NewLeaveTypePage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders Step 1: Template Gallery by default with templates and search", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);
    renderPage();

    expect(await screen.findByRole("heading", { name: /Annual Leave/i })).toBeDefined();
    expect(screen.getByText(/Create Leave Type/i)).toBeDefined();
    expect(screen.getByText(/1\. Choose Template/i)).toBeDefined();
    expect(screen.getByText(/Mental Health Day/i)).toBeDefined();
    expect(screen.getByText(/Skip to manual/i)).toBeDefined();
  });

  it("filters templates by category and search keyword", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);
    renderPage();

    await screen.findByRole("heading", { name: /Annual Leave/i });

    const searchInput = screen.getByPlaceholderText(/Search templates/i);
    fireEvent.change(searchInput, { target: { value: "Sick" } });

    expect(screen.getByText(/Sick Leave/i)).toBeDefined();
    expect(screen.queryByText(/Bereavement/i)).toBeNull();
  });

  it("selecting a template moves to Step 2 with pre-filled data", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);
    renderPage();

    const templateCard = await screen.findByRole("heading", { name: /Mental Health Day/i });
    fireEvent.click(templateCard.closest("[role='button']")!);

    expect(await screen.findByText(/Using Template/i)).toBeDefined();
    expect(screen.getAllByText(/Mental Health Day/i).length).toBeGreaterThan(0);

    const nameInput = screen.getByPlaceholderText(/e.g. Annual Leave/i) as HTMLInputElement;
    const codeInput = screen.getByPlaceholderText(/e.g. AL, SL, UL/i) as HTMLInputElement;
    expect(nameInput.value).toBe("Mental Health Day");
    expect(codeInput.value).toBe("MHD");
  });

  it("clicking Skip to manual navigates to Step 2 with blank form", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);
    renderPage();

    const skipButton = await screen.findByText(/Skip to manual/i);
    fireEvent.click(skipButton);

    expect(await screen.findByText(/Type Code/i)).toBeDefined();
    expect(screen.queryByText(/Using Template/i)).toBeNull();

    const nameInput = screen.getByPlaceholderText(/e.g. Annual Leave/i) as HTMLInputElement;
    const codeInput = screen.getByPlaceholderText(/e.g. AL, SL, UL/i) as HTMLInputElement;
    expect(nameInput.value).toBe("");
    expect(codeInput.value).toBe("");
  });

  it("toggles Paid Status in step 2 form via slider track", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);
    renderPage();

    const skipButton = await screen.findByText(/Skip to manual/i);
    fireEvent.click(skipButton);

    const paidInput = screen.getByLabelText(/Paid Status/i) as HTMLInputElement;
    const initialPaidState = paidInput.checked;

    const paidTrack = paidInput.nextElementSibling!;
    fireEvent.click(paidTrack);

    expect(paidInput.checked).toBe(!initialPaidState);
  });

  it("detects duplicate codes case-insensitively and warns user", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([
      { id: "1", code: "AL", name: "Annual Leave", is_paid: true, policy_config: {} },
    ] as never);
    renderPage();

    const skipButton = await screen.findByText(/Skip to manual/i);
    fireEvent.click(skipButton);

    const codeInput = screen.getByPlaceholderText(/e.g. AL, SL, UL/i);
    fireEvent.change(codeInput, { target: { value: "al" } });

    expect(await screen.findByText(/A leave type with this code already exists/i)).toBeDefined();
  });

  it("navigates back to templates step when clicking Change Template button", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);
    renderPage();

    const templateCard = await screen.findByRole("heading", { name: /Annual Leave/i });
    fireEvent.click(templateCard.closest("[role='button']")!);

    expect(await screen.findByText(/← Change Template/i)).toBeDefined();

    fireEvent.click(screen.getByText(/← Change Template/i));
    expect(await screen.findByText(/1\. Choose Template/i)).toBeDefined();
    expect(screen.getByText(/Start faster with a professional setup/i)).toBeDefined();
  });

  it("navigates back to /settings/leave?tab=types when clicking back link or Cancel", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);
    renderPage();

    const backLink = await screen.findByText(/Back to Leave Settings/i);
    fireEvent.click(backLink.closest("a")!);

    expect(mockNavigate).toHaveBeenCalledWith({
      to: "/settings/leave",
      search: { tab: "types" },
    });

    mockNavigate.mockClear();

    const skipButton = await screen.findByText(/Skip to manual/i);
    fireEvent.click(skipButton);

    const cancelButton = await screen.findByText(/Cancel/i);
    fireEvent.click(cancelButton);

    expect(mockNavigate).toHaveBeenCalledWith({
      to: "/settings/leave",
      search: { tab: "types" },
    });
  });

  it("renders live organization preview and updates card in real-time as user types", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);
    renderPage();

    const skipButton = await screen.findByText(/Skip to manual/i);
    fireEvent.click(skipButton);

    expect(await screen.findByText(/Live Organization Preview/i)).toBeDefined();
    expect(screen.getByText(/Full immutable audit trail & ledger tracking/i)).toBeDefined();

    const nameInput = screen.getByPlaceholderText(/e.g. Annual Leave/i);
    const codeInput = screen.getByPlaceholderText(/e.g. AL, SL, UL/i);

    fireEvent.change(nameInput, { target: { value: "Study Leave" } });
    fireEvent.change(codeInput, { target: { value: "STD" } });

    expect(screen.getByText(/Code: STD/i)).toBeDefined();
  });

  it("successfully creates leave type and navigates back", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);
    vi.mocked(api.createLeaveType).mockResolvedValue({
      id: "new-1",
      code: "REMOTE",
      name: "Remote Work",
      is_paid: true,
      policy_config: {},
    } as never);

    renderPage();

    const skipButton = await screen.findByText(/Skip to manual/i);
    fireEvent.click(skipButton);

    const codeInput = screen.getByPlaceholderText(/e.g. AL, SL, UL/i);
    const nameInput = screen.getByPlaceholderText(/e.g. Annual Leave/i);

    fireEvent.change(codeInput, { target: { value: "REMOTE" } });
    fireEvent.change(nameInput, { target: { value: "Remote Work" } });

    fireEvent.click(screen.getByText(/Confirm & Create/i));

    await waitFor(() => {
      expect(api.createLeaveType).toHaveBeenCalledWith({
        code: "REMOTE",
        name: "Remote Work",
        is_paid: true,
      });
    });

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith({
        to: "/settings/leave",
        search: { tab: "types" },
      });
    });
  });

  it("shows restore modal on 409 retired code conflict and allows restoring", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);
    const conflictError = new ApiError(409, "Conflict", "CONFLICT", {
      error_code: "CONFLICT",
      message: "Retired leave type exists",
      retry: {
        reason: "retired_leave_type",
        leave_type_id: "retired-uuid-1",
        name: "Old Sabbatical",
      },
    });
    vi.mocked(api.createLeaveType).mockRejectedValue(conflictError);
    vi.mocked(api.restoreLeaveType).mockResolvedValue({} as never);

    renderPage();

    const skipButton = await screen.findByText(/Skip to manual/i);
    fireEvent.click(skipButton);

    fireEvent.change(screen.getByPlaceholderText(/e.g. AL, SL, UL/i), { target: { value: "SABB" } });
    fireEvent.change(screen.getByPlaceholderText(/e.g. Annual Leave/i), { target: { value: "Sabbatical" } });

    fireEvent.click(screen.getByText(/Confirm & Create/i));

    expect(await screen.findByRole("heading", { name: /Restore Leave Type/i })).toBeDefined();
    expect(screen.getByText(/A leave type with this code was retired. Restore "Old Sabbatical" instead of creating a new one\?/i)).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "Restore" }));

    await waitFor(() => {
      expect(api.restoreLeaveType).toHaveBeenCalledWith("retired-uuid-1");
    });

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith({
        to: "/settings/leave",
        search: { tab: "types" },
      });
    });
  });
});
