import { API_BASE_URL } from "./config";
import { apiFetch } from "./helpers";
import { Holiday } from './types';
import { apiClient } from "./client";

export async function fetchHolidays(
    wsId: string,
    year?: number,
): Promise<Holiday[]> {
    const url = year
        ? `${API_BASE_URL}/workspaces/${wsId}/holidays?year=${year.toString()}`
        : `${API_BASE_URL}/workspaces/${wsId}/holidays`;
    const { data: resData } = await apiClient.get<Holiday[]>(`/${url}`);
    return resData;
}

export async function createHoliday(
    wsId: string,
    data: Omit<Holiday, 'id' | 'workspace_id' | 'created_at' | 'updated_at'>,
): Promise<Holiday> {
    const { data: resData } = await apiClient.post<Holiday>(`/workspaces/${wsId}/holidays`, data);
    return resData;
}

export async function updateHoliday(
    wsId: string,
    holidayId: string,
    data: Partial<Holiday>,
): Promise<Holiday> {
    const { data: resData } = await apiClient.patch<Holiday>(`/workspaces/${wsId}/holidays/${holidayId}`, data);
    return resData;
}

export async function deleteHoliday(
    wsId: string,
    holidayId: string,
): Promise<void> {
    await apiClient.delete(`/workspaces/${wsId}/holidays/${holidayId}`);
}
