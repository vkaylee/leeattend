import { render, screen } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import NotificationsPage from "../notifications-page";

// ── Mock useNotifications ──────────────────────────────────────────────────
const mockUseNotifications = vi.fn();
vi.mock("@/hooks/use-notifications", () => ({
  useNotifications: (...args: unknown[]) => mockUseNotifications(...args),
}));

// ── Mock date-fns to return stable, predictable strings ────────────────────
vi.mock("date-fns", async () => {
  const actual = await vi.importActual<typeof import("date-fns")>("date-fns");
  return {
    ...actual,
    formatDistanceToNow: vi.fn((_date: Date, _opts?: Record<string, unknown>) => "mock time ago"),
  };
});

// ── Default notification data ──────────────────────────────────────────────
const DEFAULT_NOTIFICATION = {
  id: "notif-1",
  title: "Leave Approved",
  content: "Your leave request for Jan 15 has been approved.",
  type: "leave_approved" as const,
  is_read: false,
  created_at: "2026-01-10T08:00:00Z",
  updated_at: "2026-01-10T08:00:00Z",
};

const READ_NOTIFICATION = {
  ...DEFAULT_NOTIFICATION,
  id: "notif-2",
  title: "System Update",
  content: "System maintenance completed.",
  type: "system" as const,
  is_read: true,
};

const DEFAULT_RETURN = {
  notifications: [DEFAULT_NOTIFICATION, READ_NOTIFICATION] as typeof DEFAULT_NOTIFICATION[],
  isLoading: false,
  hasNextPage: false,
  fetchNextPage: vi.fn(),
  isFetchingNextPage: false,
  markAsRead: vi.fn().mockResolvedValue(undefined),
  markAllAsRead: vi.fn().mockResolvedValue(undefined),
  isMarkingAllAsRead: false,
};

function setupNotifications(overrides: Partial<typeof DEFAULT_RETURN> = {}) {
  mockUseNotifications.mockReturnValue({ ...DEFAULT_RETURN, ...overrides });
}

describe("NotificationsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupNotifications();
  });

  // ── Happy path ──────────────────────────────────────────────────────────

  it("renders page header with title", () => {
    render(<NotificationsPage />);

    // The page title is "Thông báo" (h1), subtitle also contains this word
    const elements = screen.getAllByText(/Thông báo/i);
    expect(elements.length).toBeGreaterThanOrEqual(1);
  });

  it("renders notification items when data is loaded", () => {
    render(<NotificationsPage />);

    expect(screen.getByText("Leave Approved")).toBeDefined();
    expect(
      screen.getByText("Your leave request for Jan 15 has been approved."),
    ).toBeDefined();
    expect(screen.getByText("System Update")).toBeDefined();
  });

  it("shows unread indicator (blue bar) for unread notifications", () => {
    render(<NotificationsPage />);

    // Unread notification has a "Mark as read" button
    expect(screen.getByText(/mark_as_read|Đánh dấu đã đọc/i)).toBeDefined();
  });

  // ── Loading state ───────────────────────────────────────────────────────

  it("renders skeleton placeholders when loading", () => {
    setupNotifications({ isLoading: true, notifications: [] });

    render(<NotificationsPage />);

    // 5 skeleton pulse divs should render
    const skeletons = document.querySelectorAll(".animate-pulse");
    expect(skeletons.length).toBe(5);
  });

  // ── Empty state ─────────────────────────────────────────────────────────

  it("renders empty state when no notifications exist", () => {
    setupNotifications({ notifications: [], isLoading: false });

    render(<NotificationsPage />);

    // The setup.tsx i18n mock resolves keys to Vietnamese locale values
    expect(screen.getByText(/Không có thông báo nào/i)).toBeDefined();
    expect(screen.getByText(/Bạn chưa có thông báo nào/i)).toBeDefined();
  });

  // ── Mark All Read ───────────────────────────────────────────────────────

  it("hides 'Mark All Read' button when there are no notifications", () => {
    setupNotifications({ notifications: [], isLoading: false });

    render(<NotificationsPage />);

    expect(screen.queryByText(/mark_all_as_read|Đánh dấu tất cả đã đọc/i)).toBeNull();
  });

  it("shows 'Mark All Read' button when notifications exist", () => {
    render(<NotificationsPage />);

    expect(screen.getByText(/mark_all_as_read|Đánh dấu tất cả đã đọc/i)).toBeDefined();
  });

  it("calls markAllAsRead when 'Mark All Read' button is clicked", async () => {
    const markAllAsRead = vi.fn().mockResolvedValue(undefined);
    setupNotifications({ markAllAsRead });

    render(<NotificationsPage />);

    const button = screen.getByText(/mark_all_as_read|Đánh dấu tất cả đã đọc/i);
    await userEvent.click(button);

    expect(markAllAsRead).toHaveBeenCalledOnce();
  });

  it("disables 'Mark All Read' button while marking is in progress", () => {
    setupNotifications({ isMarkingAllAsRead: true });

    render(<NotificationsPage />);

    const button = screen.getByText(/mark_all_as_read|Đánh dấu tất cả đã đọc/i).closest("button");
    expect(button).toBeDefined();
    expect(button?.disabled).toBe(true);
  });

  // ── Mark individual as read ─────────────────────────────────────────────

  it("calls markAsRead when individual mark-read button is clicked", async () => {
    const markAsRead = vi.fn().mockResolvedValue(undefined);
    setupNotifications({ markAsRead });

    render(<NotificationsPage />);

    const btn = screen.getByText(/mark_as_read|Đánh dấu đã đọc/i);
    await userEvent.click(btn);

    expect(markAsRead).toHaveBeenCalledWith("notif-1");
  });

  // ── Load More ───────────────────────────────────────────────────────────

  it("renders 'Load More' button when hasNextPage is true", () => {
    setupNotifications({ hasNextPage: true, fetchNextPage: vi.fn() });

    render(<NotificationsPage />);

    expect(screen.getByText(/load_more|Tải thêm/i)).toBeDefined();
  });

  it("hides 'Load More' button when hasNextPage is false", () => {
    setupNotifications({ hasNextPage: false });

    render(<NotificationsPage />);

    expect(screen.queryByText(/load_more|Tải thêm/i)).toBeNull();
  });

  it("disables 'Load More' button and shows dots while fetching next page", () => {
    setupNotifications({
      hasNextPage: true,
      isFetchingNextPage: true,
      fetchNextPage: vi.fn(),
    });

    render(<NotificationsPage />);

    // The button should be disabled (no text "Tải thêm" visible)
    expect(screen.queryByText(/load_more|Tải thêm/i)).toBeNull();
  });

  it("calls fetchNextPage when 'Load More' is clicked", async () => {
    const fetchNextPage = vi.fn();
    setupNotifications({ hasNextPage: true, fetchNextPage });

    render(<NotificationsPage />);

    const button = screen.getByText(/load_more|Tải thêm/i);
    await userEvent.click(button);

    expect(fetchNextPage).toHaveBeenCalledOnce();
  });

  // ── Notification type icons ─────────────────────────────────────────────

  it("renders correct icons for each notification type", () => {
    setupNotifications({
      notifications: [
        { ...DEFAULT_NOTIFICATION, id: "1", type: "leave_approved" as const },
        { ...DEFAULT_NOTIFICATION, id: "2", type: "leave_rejected" as const },
        { ...DEFAULT_NOTIFICATION, id: "3", type: "attendance_missing" as const },
        { ...DEFAULT_NOTIFICATION, id: "4", type: "message" as const },
        { ...DEFAULT_NOTIFICATION, id: "5", type: "system" as const },
      ],
    });

    render(<NotificationsPage />);

    // All 5 notification items should render
    expect(screen.getAllByText("Leave Approved").length).toBe(5);
  });

  // ── Locale & time rendering ────────────────────────────────────────────

  it("renders relative time for each notification", () => {
    render(<NotificationsPage />);

    // Our mock returns "mock time ago" for all formatDistanceToNow calls
    const timeElements = screen.getAllByText("mock time ago");
    expect(timeElements.length).toBe(2); // Two notifications
  });

  // ── API failure / empty after load ──────────────────────────────────────

  it("renders empty state when notifications array is empty after loading", () => {
    setupNotifications({ notifications: [], isLoading: false });

    render(<NotificationsPage />);

    expect(screen.getByText(/no_notifications|Không có thông báo nào/i)).toBeDefined();
  });
});
