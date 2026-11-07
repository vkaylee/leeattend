import React from 'react';

import { Typography } from "@/components/ui/typography";
import { Payslip } from '@/lib/api/types';

interface PayslipTotalsProps {
  formatAmount: (value: string | number) => string;
  grossPay: number;
  payslip: Payslip;
}

export function PayslipTotals({ payslip, grossPay, formatAmount }: PayslipTotalsProps) {
  return (
    <div className='pt-8 border-t-2 border-slate-900 dark:border-slate-700' data-testid="totals-section">
      <div className='grid grid-cols-1 md:grid-cols-[1fr_280px] gap-8 md:gap-12'>
        <div className='space-y-4'>
          <Typography className='text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest italic' variant="p">
            Note: This is a computer-generated document and does not require a physical signature. Any discrepancies should be reported to the HR department within 3 working days.
          </Typography>
        </div>
        <div className='space-y-3'>
          <div className='flex justify-between text-sm'>
            <span className='font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest'>
              Gross Pay
            </span>
            <span className='font-bold text-slate-700 dark:text-slate-200' data-testid="gross-pay">
              {formatAmount(grossPay)}
            </span>
          </div>
          <div className='flex justify-between text-sm'>
            <span className='font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest'>
              Deductions
            </span>
            <span className='font-bold text-rose-600 dark:text-rose-400' data-testid="deductions">
              -{formatAmount(payslip.deductions_amount)}
            </span>
          </div>
          <div className='flex justify-between pt-4 border-t border-slate-100 dark:border-slate-800'>
            <span className='text-sm font-bold text-slate-900 dark:text-slate-100 uppercase tracking-widest'>
              Net Pay
            </span>
            <span className='text-xl font-black text-primary-600 dark:text-primary-400' data-testid="net-pay-total">
              {formatAmount(payslip.net_pay_amount)}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
