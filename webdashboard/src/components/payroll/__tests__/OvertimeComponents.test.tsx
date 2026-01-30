import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import OvertimeRuleEditor from '../OvertimeRuleEditor';
import AssignOvertimePolicyModal from '../AssignOvertimePolicyModal';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter } from 'react-router-dom';

vi.mock('@/lib/api/payroll', () => ({
  addOvertimeRule: vi.fn().mockResolvedValue({}),
  fetchOvertimePolicies: vi.fn().mockResolvedValue([]),
  assignOvertimePolicy: vi.fn().mockResolvedValue(true),
}));

vi.mock('@/components/ui/Toast', () => ({
  useToast: () => ({ success: vi.fn(), error: vi.fn() }),
}));

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

const renderWithProviders = (ui: React.ReactElement) => {
  return render(
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>{ui}</BrowserRouter>
    </QueryClientProvider>
  );
};

describe('OvertimeRuleEditor', () => {
  it('renders rule list and add button', () => {
    const rules = [
      { id: '1', multiplier: 1.5, time_start: '18:00:00', time_end: '22:00:00', priority: 0, policy_id: 'pol-1', days_of_week: 127, is_holiday: false }
    ];
    renderWithProviders(<OvertimeRuleEditor wsId="ws-1" policyId="pol-1" rules={rules} />);

    expect(screen.getByText(/1.5x/)).toBeDefined();
    // Use multi-language matcher
    expect(screen.getByText(/Add Rule|Thêm quy tắc/i)).toBeDefined();
  });

  it('shows add form when clicked', () => {
    renderWithProviders(<OvertimeRuleEditor wsId="ws-1" policyId="pol-1" rules={[]} />);
    fireEvent.click(screen.getByText(/Add Rule|Thêm quy tắc/i));
    expect(screen.getByText(/Multiplier/i)).toBeDefined();
  });
});

describe('AssignOvertimePolicyModal', () => {
  it('renders all assignee types', () => {
    renderWithProviders(<AssignOvertimePolicyModal wsId="ws-1" isOpen={true} onClose={vi.fn()} />);

    expect(screen.getByTestId('assignee-type-employee')).toBeDefined();
    expect(screen.getByTestId('assignee-type-department')).toBeDefined();
    expect(screen.getByTestId('assignee-type-workspace')).toBeDefined();
  });

  it('hides Target ID input when Global is selected', async () => {
    renderWithProviders(<AssignOvertimePolicyModal wsId="ws-1" isOpen={true} onClose={vi.fn()} />);

    fireEvent.click(screen.getByTestId('assignee-type-workspace'));
    await waitFor(() => {
      expect(screen.queryByTestId('assign-employee-search')).toBeNull();
      expect(screen.queryByTestId('assign-dept-select')).toBeNull();
    });
  });
});
