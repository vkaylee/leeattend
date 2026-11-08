import type { TFunction } from 'i18next';

import { CalendarDays, Building2 } from 'lucide-react';
import React from 'react';

import { Typography } from "@/components/ui/typography";
import { Organization, Payslip } from '@/lib/api/types';

interface PayslipHeaderProps {
  formatDate: (date: string) => string;
  formatMonthYear: (date: string) => string;
  payslip: Payslip;
  t: TFunction;
  organization: Organization | null;
}

export function PayslipHeader({ organization, payslip, formatMonthYear, formatDate, t }: PayslipHeaderProps) {
  return (
    <div className='flex justify-between items-start border-b-2 border-slate-900 pb-10 mb-10'>
      <div className='space-y-4'>
        <div className='flex items-center gap-3'>
          <div className='w-12 h-12 bg-slate-900 text-white rounded-xl flex items-center justify-center'>
            <Building2 size={24} />
          </div>
          <div>
            <Typography className='text-2xl font-bold uppercase tracking-tight' data-testid="organization-name" variant="h1">{organization?.name ?? 'LeeAttend'}</Typography>
            <Typography className='text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]' variant="p">{t('payroll.document.system_subtitle')}</Typography>
          </div>
        </div>
        <div className='inline-flex items-center gap-2 px-3 py-1 bg-emerald-50 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-300 rounded-full border border-emerald-100 dark:border-emerald-800 print:bg-white dark:bg-slate-900 print:border-slate-200 dark:border-slate-700'>
          <span className='text-[10px] font-bold uppercase tracking-widest'>{t('payroll.document.verified')}</span>
        </div>
      </div>

      <div className='text-right'>
        <Typography className='text-4xl font-bold uppercase tracking-tighter text-slate-700 dark:text-slate-200 mb-1' data-testid="payslip-title" variant="h2">{t('payroll.document.title')}</Typography>
        <div className='flex items-center justify-end gap-2 text-slate-500 dark:text-slate-400'>
          <CalendarDays size={14} />
          <span className='text-sm font-bold uppercase tracking-widest' data-testid="period-month">
            {t('common.month')} {formatMonthYear(payslip.period_start)}
          </span>
        </div>
        <Typography className='text-[10px] font-bold text-slate-500 dark:text-slate-400 mt-2 uppercase tracking-widest' data-testid="period-range" variant="p">
          {t('payroll.document.period')}: {formatDate(payslip.period_start)} - {formatDate(payslip.period_end)}
        </Typography>
      </div>
    </div>
  );
}
