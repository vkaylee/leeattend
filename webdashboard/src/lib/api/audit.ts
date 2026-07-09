import type { AuditLog, PaginatedResponse } from "./types";

import { client, fetch } from "./client";

export const fetchAuditLogs = async (
  workspaceId: string,
  params: { page: number; limit: number; action?: string; startDate?: string; endDate?: string }
): Promise<PaginatedResponse<AuditLog>> => {
  const { data: resData } = await fetch<PaginatedResponse<AuditLog>>("GET", "/workspaces/{ws_id}/audit-logs", {
    params: {
      path: { ws_id: workspaceId },
      query: {
        page: params.page,
        limit: params.limit,
        action: params.action,
        start_date: params.startDate,
        end_date: params.endDate,
      },
    },
  });
  if (!resData) throw new Error("No response data");
  return resData;
};

