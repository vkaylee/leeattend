import React from 'react';
import { render, screen } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import WorkspaceGuard from '../WorkspaceGuard';
import * as api from '@/lib/api';
import { useWorkspaces } from '@/hooks/use-workspace';
import { vi, describe, it, expect, beforeEach } from 'vitest';

vi.mock('@/lib/api', () => ({
  getCurrentWorkspaceId: vi.fn(),
  setCurrentWorkspaceId: vi.fn(),
}));

vi.mock('@/hooks/use-workspace', () => ({
  useWorkspaces: vi.fn(),
}));

const MockChild = () => <div>Workspace Content</div>;
const MockOnboarding = () => <div>Onboarding Page</div>;

describe('WorkspaceGuard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows loading spinner when fetching workspaces', () => {
    (api.getCurrentWorkspaceId as any).mockReturnValue(null);
    (useWorkspaces as any).mockReturnValue({ workspaces: [], isLoading: true, isError: false });

    render(
      <MemoryRouter initialEntries={['/']}>
        <WorkspaceGuard />
      </MemoryRouter>
    );

    expect(screen.getByText(/Đang tải không gian làm việc/i)).toBeInTheDocument();
  });

  it('redirects to /onboarding if user has no workspaces', () => {
    (api.getCurrentWorkspaceId as any).mockReturnValue(null);
    (useWorkspaces as any).mockReturnValue({ workspaces: [], isLoading: false, isError: false });

    render(
      <MemoryRouter initialEntries={['/']}>
        <Routes>
          <Route element={<WorkspaceGuard />}>
            <Route path="/" element={<MockChild />} />
          </Route>
          <Route path="/onboarding" element={<MockOnboarding />} />
        </Routes>
      </MemoryRouter>
    );

    expect(screen.getByText('Onboarding Page')).toBeInTheDocument();
  });

  it('automatically selects the first workspace if none active', () => {
    (api.getCurrentWorkspaceId as any).mockReturnValue(null);
    (useWorkspaces as any).mockReturnValue({ 
      workspaces: [{ id: 'ws-1', name: 'Workspace 1' }], 
      isLoading: false, 
      isError: false 
    });

    render(
      <MemoryRouter initialEntries={['/']}>
        <Routes>
          <Route element={<WorkspaceGuard />}>
            <Route path="/" element={<MockChild />} />
          </Route>
        </Routes>
      </MemoryRouter>
    );

    expect(api.setCurrentWorkspaceId).toHaveBeenCalledWith('ws-1');
  });

  it('renders children when authenticated and workspace selected', () => {
    (api.getCurrentWorkspaceId as any).mockReturnValue('ws-1');
    (useWorkspaces as any).mockReturnValue({ 
      workspaces: [{ id: 'ws-1', name: 'Workspace 1' }], 
      isLoading: false, 
      isError: false 
    });

    render(
      <MemoryRouter initialEntries={['/']}>
        <Routes>
          <Route element={<WorkspaceGuard />}>
            <Route path="/" element={<MockChild />} />
          </Route>
        </Routes>
      </MemoryRouter>
    );

    expect(screen.getByText('Workspace Content')).toBeInTheDocument();
  });
});