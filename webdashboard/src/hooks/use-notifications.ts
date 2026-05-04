import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo } from "react";

import {
  AppNotification,
  fetchNotifications,
  fetchUnreadCount,
  markAllNotificationsAsRead,
  markNotificationAsRead,
} from "@/lib/api";

// Query Keys
export const notificationKeys = {
  all: ["notifications"] as const,
  list: () => [...notificationKeys.all, "list"] as const,
  unreadCount: () => [...notificationKeys.all, "unread-count"] as const,
};

export interface UseNotificationsReturn {
  error: Error | null;
  fetchNextPage: () => void;
  hasNextPage: boolean;
  isError: boolean;
  isFetchingNextPage: boolean;
  isLoading: boolean;
  isMarkingAllAsRead: boolean;
  isMarkingAsRead: boolean;
  markAllAsRead: () => Promise<void>;
  markAsRead: (id: string) => Promise<void>;
  notifications: AppNotification[];
  unreadCount: number;
}

export function useNotifications(limit = 20): UseNotificationsReturn {
  const queryClient = useQueryClient();

  // 1. Fetch Unread Count (polling every 30s)
  const unreadCountQuery = useQuery({
    queryKey: notificationKeys.unreadCount(),
    queryFn: fetchUnreadCount,
    refetchInterval: 30000,
  });

  // 2. Fetch Infinite List of Notifications
  const infiniteQuery = useInfiniteQuery({
    queryKey: [...notificationKeys.list(), { limit }],
    queryFn: ({ pageParam }) => fetchNotifications(limit, pageParam),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
  });

  // Flatten the pages into a single array
  const notifications = useMemo(() => {
    return infiniteQuery.data?.pages.flatMap((page) => page.notifications) ?? [];
  }, [infiniteQuery.data]);

  // Derived unread count: prioritize the first page's unread_count if available,
  // otherwise fallback to the separate unreadCountQuery
  const unreadCount = useMemo(() => {
    if (infiniteQuery.data?.pages[0]?.unread_count !== undefined) {
      return infiniteQuery.data.pages[0].unread_count;
    }
    return unreadCountQuery.data?.count ?? 0;
  }, [infiniteQuery.data, unreadCountQuery.data]);

  // 3. Mutations
  const markAsReadMutation = useMutation({
    mutationFn: (id: string) => markNotificationAsRead(id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: notificationKeys.all });
    },
  });

  const markAllAsReadMutation = useMutation({
    mutationFn: () => markAllNotificationsAsRead(),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: notificationKeys.all });
    },
  });

  return {
    notifications,
    unreadCount,
    isLoading: infiniteQuery.isLoading,
    isError: infiniteQuery.isError,
    error: infiniteQuery.error,
    
    // Pagination
    hasNextPage: infiniteQuery.hasNextPage,
    fetchNextPage: () => { void infiniteQuery.fetchNextPage(); },
    isFetchingNextPage: infiniteQuery.isFetchingNextPage,

    // Actions
    markAsRead: markAsReadMutation.mutateAsync,
    isMarkingAsRead: markAsReadMutation.isPending,
    markAllAsRead: markAllAsReadMutation.mutateAsync,
    isMarkingAllAsRead: markAllAsReadMutation.isPending,
  };
}
