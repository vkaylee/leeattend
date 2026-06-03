import React from 'react';

import { Typography } from "@/components/ui/typography";
import { Payslip, SalaryComponentWithDefinition, PayrollSegment, IncentiveEvaluationResult } from '@/lib/api/types';

import { SalaryComponentRow } from './salary-component-row';

interface EarningsBreakdownProps {
  formatAmount: (value: string | number) => string;
  payslip: Payslip;
}

export function EarningsBreakdown({ payslip, formatAmount }: EarningsBreakdownProps) {
  return (
    <div>
      <Typography className='text-[10px] font-bold uppercase tracking-[0.3em] text-slate-500 dark:text-slate-400 border-b border-slate-100 dark:border-slate-800 pb-4 mb-4' variant="h3">Earnings Breakdown</Typography>
      <table className='w-full text-sm'>
        <thead>
          <tr className='text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400'>
            <th className='text-left pb-4'>Description</th>
            <th className='text-right pb-4'>Amount</th>
          </tr>
        </thead>
        <tbody className='divide-y divide-slate-50 dark:divide-slate-800'>
          {/* Salary Components Breakdown */}
          {payslip.salary_components?.map((sc: SalaryComponentWithDefinition, index: number) => (
            <SalaryComponentRow
              formatAmount={formatAmount}
              index={index}
              key={index}
              payslip={payslip}
              sc={sc}
            />
          ))}

          {/* Base Salary Segments */}
          {(!payslip.salary_components || payslip.salary_components.length === 0) && payslip.segments?.map((seg: PayrollSegment, index: number) => (
            <tr className='group' data-testid={`salary-segment-${index}`} key={index}>
              <td className='py-4'>
                <div className='flex items-center gap-3'>
                  <div className='w-1.5 h-1.5 bg-blue-600 rounded-full' />
                  <div>
                    <Typography className='font-bold text-slate-700 dark:text-slate-200' variant="p">Base Salary {index > 0 ? ` (Segment ${index + 1})` : ''}</Typography>
                    <Typography className='text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase' variant="p">
                      {seg.start_date} - {seg.end_date} • {(typeof seg.paid_work_days === 'string' ? parseFloat(seg.paid_work_days) : seg.paid_work_days).toFixed(1)} Days @ {formatAmount(seg.applied_base_salary)}
                    </Typography>
                  </div>
                </div>
              </td>
              <td className='text-right py-4 font-bold text-slate-700 dark:text-slate-200'>
                {formatAmount(seg.amount)}
              </td>
            </tr>
          ))}

          {/* Overtime */}
          {parseFloat(payslip.overtime_pay_amount) > 0 && (
            <tr data-testid="overtime-row">
              <td className='py-4'>
                <div className='flex items-center gap-3'>
                  <div className='w-1.5 h-1.5 bg-emerald-50 dark:bg-emerald-950 rounded-full' />
                  <div>
                    <Typography className='font-bold text-slate-700 dark:text-slate-200 uppercase' variant="p">Overtime Compensation</Typography>
                    <Typography className='text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase' variant="p">{payslip.overtime_units} Units</Typography>
                  </div>
                </div>
              </td>
              <td className='text-right py-4 font-bold text-slate-700 dark:text-slate-200'>
                {formatAmount(payslip.overtime_pay_amount)}
              </td>
            </tr>
          )}

          {/* Incentives */}
          {payslip.incentives?.map((inc: IncentiveEvaluationResult, index: number) => (
            <tr data-testid={`incentive-${index}`} key={index}>
              <td className='py-4'>
                <div className='flex items-center gap-3'>
                  <div className='w-1.5 h-1.5 bg-amber-400 rounded-full' />
                  <Typography className='font-bold text-slate-700 dark:text-slate-200 uppercase' variant="p">{inc.policy_name}</Typography>
                </div>
              </td>
              <td className='text-right py-4 font-bold text-slate-700 dark:text-slate-200'>
                {formatAmount(inc.amount)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
