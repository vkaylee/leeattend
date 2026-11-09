import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen, fireEvent } from '@testing-library/react';

import { describe, it, expect, vi, beforeEach } from 'vitest';

import { ToastProvider } from '../../components/ui/toast';
import { useOrganization } from "../../hooks/use-organization";
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
    fetchPayslips: vi.fn(),
    calculateBatchPayslips: vi.fn(),
    fetchWorkspaceAttendanceAdjustments: vi.fn(),
    fetchWorkspace: vi.fn(),
  };
});

vi.mock("../../hooks/use-organization", () => ({
  useOrganization: vi.fn(),
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
    vi.mocked(useOrganization).mockReturnValue({
      organization: {
        id: "test-ws-id",
        name: "Test Workspace",
      },
      organizationName: "Test Workspace",
      isLoading: false,
      isError: false,
      error: null,
    } as unknown as ReturnType<typeof useOrganization>);
    vi.mocked(api.fetchEmployeePayslips).mockResolvedValue([]);
    vi.mocked(api.fetchPayslips).mockResolvedValue([]);
    vi.mocked(api.fetchWorkspaceAttendanceAdjustments).mockResolvedValue({ data: [], total: 0, page: 1, limit: 20 });
  });

  it('renders employee list and summary table', async () => {
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: [
        { id: 'emp-1', workspace_id: 'ws-1', full_name: 'John Doe', employee_id_number: 'EMP001', created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z', tags: [] }
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

    expect((await screen.findAllByText('John Doe'))[0]).toBeDefined();
  });

  it('shows detail drawer when an employee is clicked', async () => {
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: [
        { id: 'emp-1', workspace_id: 'ws-1', full_name: 'John Doe', employee_id_number: 'EMP001', created_at: '2025-01-01T00:00:00Z', updated_at: '2025-01-01T00:00:00Z', tags: [] }
      ],
      total: 1,
      page: 1,
      limit: 3000,
    });
    vi.mocked(useOrganization).mockReturnValue({
      organization: {
        id: "test-ws-id",
        name: "Test Workspace",
      },
      organizationName: "Test Workspace",
      isLoading: false,
      isError: false,
      error: null,
    } as unknown as ReturnType<typeof useOrganization>);
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

    const employeeTexts = await screen.findAllByText('John Doe');
    const employeeRow = employeeTexts.map((el) => el.closest('tr')).find(Boolean);
    fireEvent.click(employeeRow!);

    expect((await screen.findAllByText(/Lịch sử phiếu lương/i))[0]).toBeDefined();
  });
});