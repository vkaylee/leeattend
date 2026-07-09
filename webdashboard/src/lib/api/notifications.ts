import { client, fetch } from "./client";

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
  const { data: resData } = await fetch<NotificationResponse>("GET", "/v1/notifications", {
    params: { query: { limit, cursor } },
  });
  if (!resData) throw new Error("No response data");
  return resData;
}

export async function fetchUnreadCount(): Promise<UnreadCountResponse> {
  const { data: resData } = await client.GET(`/v1/notifications/unread-count`, {});
  if (!resData) throw new Error("Failed to fetch unread count");
  return resData;
}

export async function markNotificationAsRead(id: string): Promise<void> {
  await client.PATCH("/v1/notifications/{id}/read", {
            params: { path: { id } }
      });
}

export async function markAllNotificationsAsRead(): Promise<void> {
  await client.POST(`/v1/notifications/read-all`, {});
}
