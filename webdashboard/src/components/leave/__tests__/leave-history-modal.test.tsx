import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import React from 'react';

import { vi, describe, it, expect, beforeEach } from 'vitest';

import * as api from '@/lib/api';

import { LeaveHistoryModal } from '../leave-history-modal';


// Mock react-dom BEFORE imports that might use it
vi.mock('react-dom', () => ({
    createPortal: vi.fn((node) => node),
}));

vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (keyPath: string, fallback?: string) => fallback ?? keyPath }),
}));

vi.mock('@/components/ui/toast', () => ({
    useToast: () => ({
        success: vi.fn(),
        error: vi.fn(),
    }),
}));

vi.mock('@/lib/api', async () => {
    const actual = await vi.importActual('@/lib/api');
    return {
        ...actual,
        fetchLeaveHistory: vi.fn(),
        adjustLeaveBalance: vi.fn(),
        fetchLeaveTypes: vi.fn(),
    };
});

describe('LeaveHistoryModal', () => {
    let queryClient: QueryClient;

    beforeEach(() => {
        vi.clearAllMocks();
        queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
 
        vi.mocked(api.fetchLeaveHistory).mockResolvedValue([
            {
                id: 'h-1',
                date: '2026-02-01',
                transaction_type: 'usage',
                amount: -2,
                reason: 'Sick Leave Usage',
                created_at: '2026-02-01T00:00:00Z',
            }
        ]);
 
        vi.mocked(api.fetchLeaveTypes).mockResolvedValue([
            { id: 'lt-1', name: 'Annual Leave' }
        ] as unknown as never);
    });

    const defaultProps = {
        wsId: 'test-ws-id',
        employeeId: 'emp-1',
        employeeName: 'John Doe',
        isOpen: true,
        onClose: vi.fn(),
    };

    it('renders history modal with specific leave type passed in', async () => {
        render(
            <QueryClientProvider client={queryClient}>
                <LeaveHistoryModal {...defaultProps} leaveTypeId="lt-1" leaveTypeName="Annual Leave" />
            </QueryClientProvider>
        );

        // Header includes employee name and leave type
        expect(screen.getByText('leave_balances.history.title: John Doe — Annual Leave')).toBeDefined();

        // fetchLeaveHistory called with specific leave_type_id
        await waitFor(() => {
            expect(api.fetchLeaveHistory).toHaveBeenCalledWith('test-ws-id', 'emp-1', 'lt-1');
        });
    });

    it('pre-fills manual adjustment form with leaveTypeId', async () => {
        render(
            <QueryClientProvider client={queryClient}>
                <LeaveHistoryModal {...defaultProps} leaveTypeId="lt-1" />
            </QueryClientProvider>
        );

        // Click Manual Adjust tab
        fireEvent.click(screen.getByText('leave_balances.history.tab_adjust'));

        await waitFor(() => {
            const select = screen.getByRole('combobox');
            expect((select as HTMLSelectElement).value).toBe('lt-1');
        });
    });

    it('invalidates workspace-leave-balances cache upon successful adjustment', async () => {
        // Setup successful adjustment
 
        vi.mocked(api.adjustLeaveBalance).mockResolvedValue({} as unknown as never);
        const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');

        render(
            <QueryClientProvider client={queryClient}>
                <LeaveHistoryModal {...defaultProps} leaveTypeId="lt-1" />
            </QueryClientProvider>
        );

        // Click Manual Adjust
        fireEvent.click(screen.getByText('leave_balances.history.tab_adjust'));

        // Fill form and submit
        await waitFor(() => {
            const amountInput = screen.getByPlaceholderText('leave_balances.history.amount_placeholder');
            const reasonInput = screen.getByPlaceholderText('leave_balances.history.reason_placeholder');

            fireEvent.change(amountInput, { target: { value: '1' } });
            fireEvent.change(reasonInput, { target: { value: 'Bonus' } });
        });

        fireEvent.click(screen.getByText('leave_balances.history.confirm_btn'));

        // Validate cache invalidation
        await waitFor(() => {
            expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["leave-history"] });
            expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ["workspace-leave-balances"] });
        });
    });
});
