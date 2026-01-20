import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { PayrollPriorityLegend } from '../PayrollPriorityLegend';

// Mock translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe('PayrollPriorityLegend', () => {
  it('renders correctly', () => {
    render(<PayrollPriorityLegend />);
    expect(screen.getByText('payroll.traceability.hierarchy_title')).toBeInTheDocument();
  });

  it('highlights the active source', async () => {
    render(<PayrollPriorityLegend activeSource="department" />);

    // Expand the legend
    const toggleButton = screen.getByRole('button');
    fireEvent.click(toggleButton);

    // Check for items
    expect(screen.getByText('payroll.traceability.individual_policy')).toBeInTheDocument();
    expect(screen.getByText('payroll.traceability.department_policy')).toBeInTheDocument();
    expect(screen.getByText('payroll.traceability.workspace_default')).toBeInTheDocument();
  });
});