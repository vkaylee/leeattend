import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';

import { vi, describe, it, expect, beforeEach } from 'vitest';

import * as api from '@/lib/api';

import { LeaveBalanceDetailModal } from '../leave-balance-detail-modal';


// Mock react-dom BEFORE imports that might use it
vi.mock('react-dom', () => ({
  createPortal: vi.fn((node: React.ReactNode) => node),
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (keyPath: string, fallback?: string) => fallback ?? keyPath }),
}));

vi.mock('@/lib/api', async () => {
  const actual = await vi.importActual('@/lib/api');
  return {
    ...actual,
    fetchLeaveHistory: vi.fn(),
    getCurrentWorkspaceId: () => 'test-ws-id',
  };
});

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

describe('LeaveBalanceDetailModal', () => {
  const mockBalance = {
    employee_id: 'emp-1',
    employee_name: 'John Doe',
    leave_type_id: 'lt-1',
    leave_type_name: 'Annual Leave',
    entitlement: 12,
    accrued_amount: 5,
    carried_over: 2,
    used: 4,
    remaining: 15,
    year: 2026,
  };

  const mockHistory = [
    {
      id: 'h-1',
      date: '2026-02-01',
      transaction_type: 'usage',
      amount: -2,
      reason: 'Sick Leave',
      created_at: '2026-02-01T00:00:00Z',
    },
    {
      id: 'h-2',
      date: '2026-01-15',
      transaction_type: 'accrual',
      amount: 1,
      reason: 'Monthly Accrual',
      created_at: '2026-01-15T00:00:00Z',
    }
  ];

  beforeEach(() => {
    vi.clearAllMocks();
 
    vi.mocked(api.fetchLeaveHistory).mockResolvedValue(mockHistory as unknown as never);
  });

  it('renders balance breakdown and history', async () => {
    render(
      <QueryClientProvider client={queryClient}>
// eslint-disable-next-line @typescript-eslint/no-explicit-any
        <LeaveBalanceDetailModal balance={mockBalance as unknown as never} isOpen={true} onClose={vi.fn()} />
      </QueryClientProvider>
    );

    // Check header
    expect(screen.getByText('John Doe')).toBeDefined();
    expect(screen.getByText('Annual Leave')).toBeDefined();

    // Check hero balance
    expect(screen.getByText('15')).toBeDefined();

    // Check breakdown
    expect(screen.getByText('12')).toBeDefined();
    expect(screen.getByText('+5')).toBeDefined();
    expect(screen.getByText('+2')).toBeDefined();
    expect(screen.getByText('-4')).toBeDefined();

    // Check history items
    expect(await screen.findByText('Sick Leave')).toBeDefined();
    expect(screen.getByText('Monthly Accrual')).toBeDefined();
    expect(screen.getByText('-2')).toBeDefined();
    expect(screen.getByText('+1')).toBeDefined();
  });

  it('filters history by tabs', async () => {
    render(
      <QueryClientProvider client={queryClient}>
// eslint-disable-next-line @typescript-eslint/no-explicit-any
        <LeaveBalanceDetailModal balance={mockBalance as unknown as never} isOpen={true} onClose={vi.fn()} />
      </QueryClientProvider>
    );

    // Initial state: All items visible
    expect(await screen.findByText('Sick Leave')).toBeDefined();
    expect(screen.getByText('Monthly Accrual')).toBeDefined();

    // Click Usage tab
    const usageTab = screen.getByText('Sử dụng');
    fireEvent.click(usageTab);

    // Only usage should be visible
    expect(screen.getByText('Sick Leave')).toBeDefined();
    expect(screen.queryByText('Monthly Accrual')).toBeNull();

    // Click Accruals tab
    const accrualTab = screen.getByText('Tích lũy');
    fireEvent.click(accrualTab);

    // Only accrual should be visible
    expect(screen.queryByText('Sick Leave')).toBeNull();
    expect(screen.getByText('Monthly Accrual')).toBeDefined();
  });
});
