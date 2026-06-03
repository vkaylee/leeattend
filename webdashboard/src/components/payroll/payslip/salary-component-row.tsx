import React from 'react';

import { Typography } from "@/components/ui/typography";
import { EmployeeSalaryComponent, SalaryComponentDefinition, SalaryComponentWithDefinition, Payslip } from '@/lib/api/types';

interface SalaryComponentRowProps {
  formatAmount: (value: string | number) => string;
  index: number;
  payslip: Payslip;
  sc: SalaryComponentWithDefinition;
}

function getMonthlyRate(component: Partial<EmployeeSalaryComponent> | null | undefined): number {
  return parseFloat(component?.amount ?? '0');
}

function getStandardDays(payslip: Payslip): number {
  return parseFloat((payslip.segments?.[0]?.applied_std_days ?? 22).toString());
}

function getTotalPaidDays(payslip: Payslip): number {
  return payslip.segments?.reduce((sum: number, seg) => sum + parseFloat(seg.paid_work_days.toString()), 0) ?? 0;
}

function getDailyRate(monthlyRate: number, standardDays: number): number {
  return standardDays > 0 ? monthlyRate / standardDays : 0;
}

export function SalaryComponentRow({ sc, index, payslip, formatAmount }: SalaryComponentRowProps) {
  const component = sc.component as Partial<EmployeeSalaryComponent> | undefined;
  const definition = sc.definition as Partial<SalaryComponentDefinition> | undefined;

  const monthlyRate = getMonthlyRate(component);
  const standardDays = getStandardDays(payslip);
  const totalPaidDays = getTotalPaidDays(payslip);
  const dailyRate = getDailyRate(monthlyRate, standardDays);
  const proratedAmount = dailyRate * totalPaidDays;

  return (
    <tr className='group' data-testid={`salary-component-${index}`} key={index}>
      <td className='py-4'>
        <div className='flex items-center gap-3'>
          <div className='w-1.5 h-1.5 bg-blue-600 rounded-full' />
          <div>
            <Typography className='font-bold text-slate-700 dark:text-slate-200' variant="p">{definition?.name ?? 'N/A'}</Typography>
            <Typography className='text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase' variant="p">
              Code: {definition?.code ?? 'N/A'} • Monthly: {formatAmount(monthlyRate)} • {totalPaidDays.toFixed(1)} Days @ {formatAmount(dailyRate)}/day
            </Typography>
          </div>
        </div>
      </td>
      <td className='text-right py-4 font-bold text-slate-700 dark:text-slate-200'>
        {formatAmount(proratedAmount)}
      </td>
    </tr>
  );
}
