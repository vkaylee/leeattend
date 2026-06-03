import type { TFunction } from 'i18next';

import React from 'react';

import { Typography } from "@/components/ui/typography";
import { Employee, Payslip } from '@/lib/api/types';

interface EmployeeDetailsProps {
  employee: Employee;
  formatAmount: (value: string | number) => string;
  payslip: Payslip;
  t: TFunction;
}

export function EmployeeDetails({ employee, payslip, formatAmount, t }: EmployeeDetailsProps) {
  return (
    <div className='grid grid-cols-2 gap-12 mb-12'>
      <div className='space-y-6' data-testid="employee-details">
        <Typography className='text-[10px] font-bold uppercase tracking-[0.3em] text-slate-500 dark:text-slate-400 border-b border-slate-100 dark:border-slate-800 pb-2' variant="h3">{t('payroll.document.employee_details')}</Typography>
        <div className='grid grid-cols-[100px_1fr] gap-y-4 text-sm'>
          <span className='font-bold text-slate-500 dark:text-slate-400 uppercase text-[10px] tracking-widest'>{t('payroll.document.name')}</span>
          <span className='font-bold text-slate-700 dark:text-slate-200 uppercase' data-testid="employee-name">{employee.full_name}</span>

          <span className='font-bold text-slate-500 dark:text-slate-400 uppercase text-[10px] tracking-widest'>{t('payroll.document.employee_id')}</span>
          <span className='font-bold text-slate-700 dark:text-slate-200' data-testid="employee-id">{employee.employee_id_number}</span>

          <span className='font-bold text-slate-500 dark:text-slate-400 uppercase text-[10px] tracking-widest'>{t('payroll.document.department')}</span>
          <span className='font-bold text-slate-700 dark:text-slate-200' data-testid="employee-department">{employee.department_name ?? 'N/A'}</span>
        </div>
      </div>

      <div className='bg-slate-50 dark:bg-slate-800 rounded-3xl p-8 border border-slate-100 dark:border-slate-800 print:bg-white dark:bg-slate-900 print:border-slate-200 dark:border-slate-700' data-testid="net-payment-box">
        <Typography className='text-[10px] font-bold uppercase tracking-[0.3em] text-slate-500 dark:text-slate-400 mb-6 text-center' variant="h3">{t('payroll.document.net_payment')}</Typography>
        <div className='text-center space-y-1'>
          <Typography className='text-5xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter leading-none' data-testid="net-pay-amount" variant="p">
            {formatAmount(payslip.net_pay_amount)}
          </Typography>
          <Typography className='text-[10px] font-bold text-emerald-600 dark:text-emerald-300 uppercase tracking-[0.2em] pt-2' variant="p">Paid via Bank Transfer</Typography>
        </div>
      </div>
    </div>
  );
}
