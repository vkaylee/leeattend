import { describe, it, expect, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useAuthorizedAgents } from '../use-agents';
import * as api from '@/lib/api';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';

// Mock the API
vi.mock('@/lib/api', () => ({
  fetchAuthorizedAgents: vi.fn(),
  createAuthorizedAgent: vi.fn(),
  updateAuthorizedAgent: vi.fn(),
  deleteAuthorizedAgent: vi.fn(),
  getCurrentUserId: vi.fn().mockReturnValue('user-123'),
  getCurrentWorkspaceId: vi.fn().mockReturnValue('ws-123'),
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

describe('useAuthorizedAgents', () => {
  it('fetches agents successfully', async () => {
    const mockAgents = [{ id: 'a-1', name: 'Agent 1', is_trusted: true }];
    vi.mocked(api.fetchAuthorizedAgents).mockResolvedValue(mockAgents as any);

    const { result } = renderHook(() => useAuthorizedAgents(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.agents).toEqual(mockAgents);
  });

  it('updates an agent successfully', async () => {
    const updatedAgent = { id: 'a-1', name: 'Updated Name' };
    vi.mocked(api.updateAuthorizedAgent).mockResolvedValue(updatedAgent as any);

    const { result } = renderHook(() => useAuthorizedAgents(), {
      wrapper: createWrapper(),
    });

    await result.current.updateAgent({ id: 'a-1', agent: { name: 'Updated Name' } });
    expect(api.updateAuthorizedAgent).toHaveBeenCalled();
  });
});
