import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import PayrollPage from '../PayrollPage';
import { BrowserRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ToastProvider } from '../../components/Toast';
import * as api from '../../lib/api';

// Partially mock the API library
vi.mock('../../lib/api', async () => {
  const actual = await vi.importActual('../../lib/api') as any;
  return {
    ...actual,
    fetchEmployees: vi.fn(),
    searchEmployees: vi.fn(),
    fetchEmployeePayslips: vi.fn(),
    fetchWorkspacePayslips: vi.fn(),
    calculateBatchPayslips: vi.fn(),
    getCurrentWorkspaceId: vi.fn(() => 'test-ws-id'),
    fetchWorkspaceAttendanceAdjustments: vi.fn(),
    fetchWorkspace: vi.fn(),
    fetchWorkspaceStats: vi.fn(),
  };
});

describe('PayrollPage', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });

    // Provide default mock values
    (api.fetchEmployees as any).mockResolvedValue({
      data: [],
      total: 0,
      page: 1,
      limit: 3000,
    });
    (api.searchEmployees as any).mockResolvedValue({
      data: [],
      total: 0,
      page: 1,
      limit: 100,
    });
    (api.fetchWorkspace as any).mockResolvedValue({ id: 'test-ws-id', name: 'Test Workspace' });
    (api.fetchWorkspaceStats as any).mockResolvedValue({ total_employees: 0, active_today: 0, on_leave: 0, pending_adjustments: 0 });
    (api.fetchEmployeePayslips as any).mockResolvedValue([]);
    (api.fetchWorkspacePayslips as any).mockResolvedValue([]);
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([]);
    (api.getCurrentWorkspaceId as any).mockReturnValue('test-ws-id');
  });

  it('renders employee list and summary table', async () => {
    (api.fetchEmployees as any).mockResolvedValue({
      data: [
        { id: 'emp-1', full_name: 'John Doe', employee_id_number: 'EMP001', hire_date: '2025-01-01', workspace_id: 'ws-1', is_active: true, created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' }
      ],
      total: 1,
      page: 1,
      limit: 3000,
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <PayrollPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    expect(await screen.findByText('John Doe')).toBeDefined();
  });

  it('shows detail drawer when an employee is clicked', async () => {
    (api.fetchEmployees as any).mockResolvedValue({
      data: [
        { id: 'emp-1', full_name: 'John Doe', employee_id_number: 'EMP001', hire_date: '2025-01-01', workspace_id: 'ws-1', is_active: true, created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' }
      ],
      total: 1,
      page: 1,
      limit: 3000,
    });
    (api.fetchEmployeePayslips as any).mockResolvedValue([
      { id: 'ps-1', employee_id: 'emp-1', period_start: '2025-12-01', period_end: '2025-12-31', base_pay_amount: '10000000', overtime_pay_amount: '400000', total_deductions: '0', net_pay_amount: '10400000', status: 'paid' }
    ]);

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            <PayrollPage />
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );

    const employeeText = await screen.findByText('John Doe');
    const employeeRow = employeeText.closest('tr');
    fireEvent.click(employeeRow!);

    expect((await screen.findAllByText(/Lịch sử phiếu lương/i))[0]).toBeDefined();
  });
});