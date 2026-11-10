import { client } from "./client";

export type ShiftSwapStatus = "pending_target" | "pending_admin" | "approved" | "rejected" | "cancelled";

export interface ShiftSwapRequest {
    approved_by_user_id?: string | null;
    created_at: string;
    id: string;
    reason?: string | null;
    requested_by_user_id?: string | null;
    requester_id: string;
    requester_name?: string | null;
    requester_shift_id?: string | null;
    requester_shift_name?: string | null;
    status: ShiftSwapStatus;
    swap_date: string;
    target_id: string;
    target_name?: string | null;
    target_shift_id?: string | null;
    target_shift_name?: string | null;
    updated_at: string;
}

export async function fetchShiftSwaps(
    status?: ShiftSwapStatus,
): Promise<ShiftSwapRequest[]> {
    const { data } = await client.GET<ShiftSwapRequest[]>("/v1/shift-swaps", {
        params: {
            query: { status: status },
        },
    });
    return data;
}

export async function createShiftSwap(
    data: {
        requester_id: string;
        target_id: string;
        swap_date: string;
        reason?: string;
    },
): Promise<ShiftSwapRequest> {
    const { data: resData } = await client.POST<ShiftSwapRequest>("/v1/shift-swaps", {
        body: data,
    });
    return resData;
}

export async function approveShiftSwap(
    swapId: string,
    data: { status: ShiftSwapStatus },
): Promise<void> {
    await client.PUT("/v1/shift-swaps/{id}/approve", {
        params: {
            path: { id: swapId },
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

export async function fetchShiftSwapStats(): Promise<SwapStats> {
  const { data: resData } = await client.GET<SwapStats>("/v1/shift-swaps/stats");
  return resData;
}
