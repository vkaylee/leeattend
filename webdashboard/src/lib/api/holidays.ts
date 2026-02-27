import { API_BASE_URL } from './config';
import { apiFetch } from './helpers';
import { Holiday } from './types';

export async function fetchHolidays(
    wsId: string,
    year?: number,
): Promise<Holiday[]> {
    const url = year
        ? `${API_BASE_URL}/workspaces/${wsId}/holidays?year=${year}`
        : `${API_BASE_URL}/workspaces/${wsId}/holidays`;
    const response = await apiFetch(url);
    if (!response.ok) throw new Error('Failed to fetch holidays');
    return response.json();
}

export async function createHoliday(
    wsId: string,
    data: Omit<Holiday, 'id' | 'workspace_id' | 'created_at' | 'updated_at'>,
): Promise<Holiday> {
    const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/holidays`, {
        method: 'POST',
        body: JSON.stringify(data),
    });
    if (!response.ok) {
        if (response.status === 409) throw new Error('A holiday already exists on this date');
        throw new Error('Failed to create holiday');
    }
    return response.json();
}

export async function updateHoliday(
    wsId: string,
    holidayId: string,
    data: Partial<Holiday>,
): Promise<Holiday> {
    const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/holidays/${holidayId}`, {
        method: 'PATCH',
        body: JSON.stringify(data),
    });
    if (!response.ok) throw new Error('Failed to update holiday');
    return response.json();
}

export async function deleteHoliday(
    wsId: string,
    holidayId: string,
): Promise<void> {
    const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/holidays/${holidayId}`, {
        method: 'DELETE',
    });
    if (!response.ok) throw new Error('Failed to delete holiday');
}
