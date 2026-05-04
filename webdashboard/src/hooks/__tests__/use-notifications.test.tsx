import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { beforeEach, describe, expect, it, vi } from "vitest";

import * as api from "@/lib/api";

import { useNotifications } from "../use-notifications";

// Mock the API module
vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/api")>();
  return {
    ...actual,
    fetchNotifications: vi.fn(),
    fetchUnreadCount: vi.fn(),
    markNotificationAsRead: vi.fn(),
    markAllNotificationsAsRead: vi.fn(),
  };
});

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
};

describe("useNotifications", () => {
  const mockNotifications = [
    {
      id: "1",
      user_id: "u1",
      type: "info",
      title: "Test 1",
      content: "Content 1",
      is_read: false,
      link: null,
      created_at: new Date().toISOString(),
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should fetch notifications and unread count", async () => {
    vi.mocked(api.fetchNotifications).mockResolvedValue({
      notifications: mockNotifications,
      next_cursor: null,
      unread_count: 1,
    });
    vi.mocked(api.fetchUnreadCount).mockResolvedValue({ count: 1 });

    const { result } = renderHook(() => useNotifications(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.notifications).toEqual(mockNotifications);
    expect(result.current.unreadCount).toBe(1);
  });

  it("should mark a notification as read", async () => {
    vi.mocked(api.fetchNotifications).mockResolvedValue({
      notifications: mockNotifications,
      next_cursor: null,
      unread_count: 1,
    });
    vi.mocked(api.markNotificationAsRead).mockResolvedValue(undefined);

    const { result } = renderHook(() => useNotifications(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await result.current.markAsRead("1");

    expect(api.markNotificationAsRead).toHaveBeenCalledWith("1");
  });

  it("should mark all as read", async () => {
    vi.mocked(api.fetchNotifications).mockResolvedValue({
      notifications: mockNotifications,
      next_cursor: null,
      unread_count: 1,
    });
    vi.mocked(api.markAllNotificationsAsRead).mockResolvedValue(undefined);

    const { result } = renderHook(() => useNotifications(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await result.current.markAllAsRead();

    expect(api.markAllNotificationsAsRead).toHaveBeenCalled();
  });
});
