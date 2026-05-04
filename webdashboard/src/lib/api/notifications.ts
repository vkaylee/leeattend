import { apiFetch } from "./helpers";

export type NotificationType = "leave_approved" | "leave_rejected" | "attendance_missing" | "system" | "message" | "info" | "alert" | (string & {});

export interface AppNotification {
  content: string;
  created_at: string;
  id: string;
  is_read: boolean;
  link: string | null;
  title: string;
  type: NotificationType;
  user_id: string;
}

export interface NotificationResponse {
  next_cursor: string | null;
  notifications: AppNotification[];
  unread_count: number;
}

export interface UnreadCountResponse {
  count: number;
}

export async function fetchNotifications(
  limit = 20,
  cursor?: string
): Promise<NotificationResponse> {
  const url = new URL("/api/v1/notifications", window.location.origin);
  url.searchParams.append("limit", limit.toString());
  if (cursor) {
    url.searchParams.append("cursor", cursor);
  }

  const response = await apiFetch(url.pathname + url.search);
  if (!response.ok) {
    throw new Error("Failed to fetch notifications");
  }
  return response.json() as Promise<NotificationResponse>;
}

export async function fetchUnreadCount(): Promise<UnreadCountResponse> {
  const response = await apiFetch("/api/v1/notifications/unread-count");
  if (!response.ok) {
    throw new Error("Failed to fetch unread count");
  }
  return response.json() as Promise<UnreadCountResponse>;
}

export async function markNotificationAsRead(id: string): Promise<void> {
  const response = await apiFetch(`/api/v1/notifications/${id}/read`, {
    method: "PATCH",
  });
  if (!response.ok) {
    throw new Error("Failed to mark notification as read");
  }
}

export async function markAllNotificationsAsRead(): Promise<void> {
  const response = await apiFetch("/api/v1/notifications/read-all", {
    method: "POST",
  });
  if (!response.ok) {
    throw new Error("Failed to mark all notifications as read");
  }
}
