import { client } from "./client";
import type { Holiday } from './types';

export async function fetchHolidays(
    year?: number,
): Promise<Holiday[]> {
    const { data: resData } = await client.GET<{ data: Holiday[] }>("/api/v1/holidays", {
        params: {
            query: year ? { year } : undefined,
        },
    });
    return resData.data;
}

export async function createHoliday(
    data: Omit<Holiday, 'id' | 'created_at' | 'updated_at'>,
): Promise<Holiday> {
    const { data: resData } = await client.POST<Holiday>("/api/v1/holidays", {
        body: data,
    });
    return resData;
}

export async function updateHoliday(
    holidayId: string,
    data: Partial<Holiday>,
): Promise<Holiday> {
    const { data: resData } = await client.PUT<Holiday>("/api/v1/holidays/{id}", {
        params: {
            path: { id: holidayId },
        },
        body: data,
    });
    return resData;
}

export async function deleteHoliday(
    holidayId: string,
): Promise<void> {
    await client.DELETE("/api/v1/holidays/{id}", {
        params: {
            path: { id: holidayId },
        },
    });
}
