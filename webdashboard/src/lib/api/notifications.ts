import type { NotificationResponse, UnreadCountResponse } from "@/lib/api/types";

import { client } from "./client";


/**
 * The notification type discriminator as declared by the API schema.
 * Individual icon/display switches should handle specific known values
 * and fall through to a default branch for future types.
 */
export type NotificationType = string;

export async function fetchNotifications(
  limit = 20,
  cursor?: string
): Promise<NotificationResponse> {
  const { data } = await client.GET<NotificationResponse>("/v1/notifications", {
    params: { query: { limit, cursor } },
  });
  return data;
}

export async function fetchUnreadCount(): Promise<UnreadCountResponse> {
  const { data } = await client.GET<UnreadCountResponse>(`/v1/notifications/unread-count`, {});
  return data;
}

export async function markNotificationAsRead(id: string): Promise<void> {
  await client.PATCH("/v1/notifications/{id}/read", {
            params: { path: { id } }
      });
}

export async function markAllNotificationsAsRead(): Promise<void> {
  await client.POST(`/v1/notifications/read-all`, {});
}
