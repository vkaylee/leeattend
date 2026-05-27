import { apiFetch } from "./helpers";
import { apiClient } from "./client";

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
  const params = new URLSearchParams({ limit: limit.toString() });
  if (cursor) params.append("cursor", cursor);

  const { data: resData } = await apiClient.get<NotificationResponse>(`/v1/notifications?${params.toString()}`);
    return resData;
}

export async function fetchUnreadCount(): Promise<UnreadCountResponse> {
  const { data: resData } = await apiClient.get<UnreadCountResponse>(`/v1/notifications/unread-count`);
    return resData;
}

export async function markNotificationAsRead(id: string): Promise<void> {
  await apiClient.patch(`/v1/notifications/${id}/read`, {});
}

export async function markAllNotificationsAsRead(): Promise<void> {
  await apiClient.post(`/v1/notifications/read-all`, {});
}
