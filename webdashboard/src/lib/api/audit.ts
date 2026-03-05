import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import type { AuditLog, PaginatedResponse } from "./types";

export const fetchAuditLogs = async (
    workspaceId: string,
    params: { page: number; limit: number }
): Promise<PaginatedResponse<AuditLog>> => {
    const query = new URLSearchParams({
        page: params.page.toString(),
        limit: params.limit.toString(),
    });

    const response = await apiFetch(`${API_BASE_URL}/workspaces/${workspaceId}/audit-logs?${query}`);
    if (!response.ok) {
        throw new Error("Failed to fetch audit logs");
    }
    return response.json();
};
