import type { AuditLog, PaginatedResponse } from "./types";

import { fetch } from "./client";

export const fetchAuditLogs = async (
  params: { page: number; limit: number; action?: string; startDate?: string; endDate?: string }
): Promise<PaginatedResponse<AuditLog>> => {
  const { data: resData } = await fetch<PaginatedResponse<AuditLog>>("GET", "/v1/audit-logs", {
    params: {
      query: {
        page: params.page,
        limit: params.limit,
        action: params.action,
        start_date: params.startDate,
        end_date: params.endDate,
      },
    },
  });
  return resData;
};
