import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen, fireEvent } from '@testing-library/react';

import { describe, it, expect, vi, beforeEach } from 'vitest';

import { ToastProvider } from '../../components/ui/toast';
import { useWorkspace } from "../../hooks/use-workspace";
import * as api from '../../lib/api';
import { Payslip } from "../../lib/api/types";
import PayrollPage from '../payroll-page';

// Partially mock the API library
 
vi.mock('../../lib/api', async () => {
  const actual = await vi.importActual('../../lib/api');
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
  };
});

vi.mock("../../hooks/use-workspace", () => ({
  useWorkspace: vi.fn(),
}));

describe('PayrollPage', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
 
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });

    // Provide default mock values
    vi.mocked(api.fetchEmployees).mockResolvedValue({
 
      data: [],
      total: 0,
      page: 1,
      limit: 3000,
    });
    vi.mocked(api.searchEmployees).mockResolvedValue({
 
      data: [],
 
      total: 0,
 
      page: 1,
 
      limit: 100,
 
    });
    vi.mocked(useWorkspace).mockReturnValue({
      workspace: {
        id: "test-ws-id",
 
        name: "Test Workspace",
      },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useWorkspace>);
    vi.mocked(api.fetchEmployeePayslips).mockResolvedValue([]);
    vi.mocked(api.fetchWorkspacePayslips).mockResolvedValue([]);
    vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue([]);
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue('test-ws-id');
  });

  it('renders employee list and summary table', async () => {
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: [
        { id: 'emp-1', workspace_id: 'ws-1', full_name: 'John Doe', employee_id_number: 'EMP001', original_hire_date: "2020-01-01", hire_date: '2025-01-01', is_active: true, created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' }
      ],
      total: 1,
      page: 1,
      limit: 3000,
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <PayrollPage />
        </ToastProvider>
      </QueryClientProvider>
    );

 
    expect(await screen.findByText('John Doe')).toBeDefined();
  });

  it('shows detail drawer when an employee is clicked', async () => {
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: [
        { id: 'emp-1', workspace_id: 'ws-1', full_name: 'John Doe', employee_id_number: 'EMP001', original_hire_date: "2020-01-01", hire_date: '2025-01-01', is_active: true, created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z' }
      ],
      total: 1,
      page: 1,
      limit: 3000,
    });
    vi.mocked(useWorkspace).mockReturnValue({
      workspace: {
        id: "test-ws-id",
        name: "Test Workspace",
        timezone: "Asia/Ho_Chi_Minh",
        created_at: "2024-01-01T00:00:00Z",
      },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useWorkspace>);
    vi.mocked(api.fetchEmployeePayslips).mockResolvedValue([
      { 
        id: 'ps-1', 
        employee_id: 'emp-1', 
        period_start: '2025-12-01', 
        period_end: '2025-12-31', 
        base_pay_amount: '10000000', 
        overtime_pay_amount: '400000', 
        holiday_work_premium_amount: '0',
        deductions_amount: '0', 
        net_pay_amount: '10400000', 
        status: 'paid',
        standard_units: 22,
        overtime_units: 10,
        created_at: '2025-01-01T00:00:00Z',
        updated_at: '2025-01-01T00:00:00Z'
      }
    ] as unknown as Payslip[]);

    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <PayrollPage />
        </ToastProvider>
      </QueryClientProvider>
    );

    const employeeText = await screen.findByText('John Doe');
    const employeeRow = employeeText.closest('tr');
    fireEvent.click(employeeRow!);

    expect((await screen.findAllByText(/Lịch sử phiếu lương/i))[0]).toBeDefined();
  });
});