import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";

export type ShiftSwapStatus = "pending_target" | "pending_admin" | "approved" | "rejected" | "cancelled";

export interface ShiftSwapRequest {
    id: string;
    workspace_id: string;
    requester_id: string;
    target_id: string;
    swap_date: string;
    requester_shift_id: string | null;
    target_shift_id: string | null;
    status: ShiftSwapStatus;
    reason: string | null;
    requested_by_user_id: string | null;
    approved_by_user_id: string | null;
    created_at: string;
    updated_at: string;
    requester_name?: string | null;
    target_name?: string | null;
    requester_shift_name?: string | null;
    target_shift_name?: string | null;
}

export async function fetchShiftSwaps(
    wsId: string,
    status?: ShiftSwapStatus,
): Promise<ShiftSwapRequest[]> {
    let url = `${API_BASE_URL}/workspaces/${wsId}/shift-swaps`;
    if (status) url += `?status=${status}`;
    const response = await apiFetch(url);
    if (!response.ok) throw new Error("Failed to fetch shift swaps");
    return response.json();
}

export async function createShiftSwap(
    wsId: string,
    data: {
        requester_id: string;
        target_id: string;
        swap_date: string;
        reason?: string;
    },
): Promise<ShiftSwapRequest> {
    const response = await apiFetch(
        `${API_BASE_URL}/workspaces/${wsId}/shift-swaps`,
        {
            method: "POST",
            body: JSON.stringify(data),
        },
    );
    if (!response.ok) throw new Error("Failed to create shift swap");
    return response.json();
}

export async function approveShiftSwap(
    wsId: string,
    swapId: string,
    data: { status: ShiftSwapStatus },
): Promise<void> {
    const response = await apiFetch(
        `${API_BASE_URL}/workspaces/${wsId}/shift-swaps/${swapId}/approve`,
        {
            method: "PUT",
            body: JSON.stringify(data),
        },
    );
    if (!response.ok) throw new Error("Failed to approve shift swap");
}

export interface SwapStats {
    pending_target: number;
    pending_admin: number;
    approved: number;
    rejected: number;
    cancelled: number;
    approved_this_month: number;
    rejected_this_month: number;
}

export async function fetchShiftSwapStats(
    wsId: string,
): Promise<SwapStats> {
    const response = await apiFetch(
        `${API_BASE_URL}/workspaces/${wsId}/shift-swaps/stats`,
    );
    if (!response.ok) throw new Error("Failed to fetch swap stats");
    return response.json();
}
