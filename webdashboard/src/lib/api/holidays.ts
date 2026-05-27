import { Holiday } from './types';
import { client } from "./client";

export async function fetchHolidays(
    wsId: string,
    year?: number,
): Promise<Holiday[]> {
    const { data: resData } = await client.GET("/workspaces/{workspace_id}/holidays", {
        params: {
            path: { workspace_id: wsId },
            query: year ? { year } : undefined,
        },
    });
    if (!resData) throw new Error("No response data");
    return resData as unknown as Holiday[];
}

export async function createHoliday(
    wsId: string,
    data: Omit<Holiday, 'id' | 'workspace_id' | 'created_at' | 'updated_at'>,
): Promise<Holiday> {
    const { data: resData } = await client.POST("/workspaces/{workspace_id}/holidays", {
        params: {
            path: { workspace_id: wsId },
        },
        body: data as any,
    });
    if (!resData) throw new Error("No response data");
    return resData as unknown as Holiday;
}

export async function updateHoliday(
    wsId: string,
    holidayId: string,
    data: Partial<Holiday>,
): Promise<Holiday> {
    const { data: resData } = await client.PUT("/workspaces/{workspace_id}/holidays/{id}", {
        params: {
            path: { workspace_id: wsId, id: holidayId },
        },
        body: data as any,
    });
    if (!resData) throw new Error("No response data");
    return resData as unknown as Holiday;
}

export async function deleteHoliday(
    wsId: string,
    holidayId: string,
): Promise<void> {
    await client.DELETE("/workspaces/{workspace_id}/holidays/{id}", {
        params: {
            path: { workspace_id: wsId, id: holidayId },
        },
    });
}
