import { client, fetch } from "./client";

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
    const { data } = await fetch<ShiftSwapRequest[]>("GET", "/workspaces/{workspace_id}/shift-swaps", {
        params: {
            path: { workspace_id: wsId },
            query: { status: status },
        },
    });
    if (!data) throw new Error("No response data");
    return data;
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
    const { data: resData } = await fetch<ShiftSwapRequest>("POST", "/workspaces/{workspace_id}/shift-swaps", {
        params: { path: { workspace_id: wsId } },
        body: data,
    });
    if (!resData) throw new Error("No response data");
    return resData;
}

export async function approveShiftSwap(
    wsId: string,
    swapId: string,
    data: { status: ShiftSwapStatus },
): Promise<void> {
    await client.PUT("/workspaces/{workspace_id}/shift-swaps/{id}/approve", {
        params: {
            path: { workspace_id: wsId, id: swapId },
        },
        body: data,
    });
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
    const { data: resData } = await client.GET("/workspaces/{workspace_id}/shift-swaps/stats", {
        params: {
            path: { workspace_id: wsId },
        },
    });
    if (!resData) throw new Error("No response data");
    return resData;
}
