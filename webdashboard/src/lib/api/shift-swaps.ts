import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";

export type ShiftSwapStatus = "pending_target" | "pending_admin" | "approved" | "rejected" | "cancelled";

export interface ShiftSwapRequest {
    approved_by_user_id: string | null;
    created_at: string;
    id: string;
    reason: string | null;
    requested_by_user_id: string | null;
    requester_id: string;
    requester_name?: string | null;
    requester_shift_id: string | null;
    requester_shift_name?: string | null;
    status: ShiftSwapStatus;
    swap_date: string;
    target_id: string;
    target_name?: string | null;
    target_shift_id: string | null;
    target_shift_name?: string | null;
    updated_at: string;
    workspace_id: string;
}

export async function fetchShiftSwaps(
    wsId: string,
    status?: ShiftSwapStatus,
): Promise<ShiftSwapRequest[]> {
    let url = `${API_BASE_URL}/workspaces/${wsId}/shift-swaps`;
    if (status) url += `?status=${status}`;
    const response = await apiFetch(url);
    if (!response.ok) throw new Error("Failed to fetch shift swaps");
    return response.json() as Promise<ShiftSwapRequest[]>;
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
    return response.json() as Promise<ShiftSwapRequest>;
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
    approved: number;
    approved_this_month: number;
    cancelled: number;
    pending_admin: number;
    pending_target: number;
    rejected: number;
    rejected_this_month: number;
}

export async function fetchShiftSwapStats(
    wsId: string,
): Promise<SwapStats> {
    const response = await apiFetch(
        `${API_BASE_URL}/workspaces/${wsId}/shift-swaps/stats`,
    );
    if (!response.ok) throw new Error("Failed to fetch swap stats");
    return response.json() as Promise<SwapStats>;
}
