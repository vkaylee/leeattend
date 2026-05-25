import { 
  Building2, 
  CalendarDays, 
  ShieldCheck
} from 'lucide-react';
import React from 'react';
import { useTranslation } from 'react-i18next';

import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from '@/hooks/use-datetime';
import { Employee, Payslip, Workspace } from '@/lib/api/types';

interface PayslipDocumentProps {
  employee: Employee;
  payslip: Payslip;
  workspace: Workspace | null;
}

import type { TFunction } from 'i18next';

interface PayslipHeaderProps {
  formatDate: (date: string) => string;
  formatMonthYear: (date: string) => string;
  payslip: Payslip;
  t: TFunction;
  workspace: Workspace | null;
}

function PayslipHeader({ workspace, payslip, formatMonthYear, formatDate, t }: PayslipHeaderProps) {
  return (
    <div className='flex justify-between items-start border-b-2 border-slate-900 pb-10 mb-10'>
      <div className='space-y-4'>
        <div className='flex items-center gap-3'>
          <div className='w-12 h-12 bg-slate-900 text-white rounded-xl flex items-center justify-center'>
            <Building2 size={24} />
          </div>
          <div>
            <Typography className='text-2xl font-bold uppercase tracking-tight' data-testid="workspace-name" variant="h1">{workspace?.name ?? 'LeeAttend'}</Typography>
            <Typography className='text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]' variant="p">{t('payroll.document.system_subtitle')}</Typography>
          </div>
        </div>
        <div className='inline-flex items-center gap-2 px-3 py-1 bg-emerald-50 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-300 rounded-full border border-emerald-100 dark:border-emerald-800 print:bg-white dark:bg-slate-900 print:border-slate-200 dark:border-slate-700'>
          <ShieldCheck size={12} />
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

interface EmployeeDetailsProps {
  employee: Employee;
  formatAmount: (value: string | number) => string;
  payslip: Payslip;
  t: TFunction;
}

function EmployeeDetails({ employee, payslip, formatAmount, t }: EmployeeDetailsProps) {
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

interface EarningsBreakdownProps {
  formatAmount: (value: string | number) => string;
  payslip: Payslip;
}

function EarningsBreakdown({ payslip, formatAmount }: EarningsBreakdownProps) {
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
          {payslip.salary_components?.map((sc, index: number) => {
            if (!sc.component || !sc.definition) return null;
            
            const amountString = sc.component.amount || "0";
            const monthlyRate = parseFloat(amountString);
            const standardDays = parseFloat((payslip.segments?.[0]?.applied_std_days ?? 22).toString());
            const totalPaidDays = payslip.segments?.reduce((sum: number, seg) => sum + parseFloat(seg.paid_work_days.toString()), 0) ?? 0;
            const dailyRate = standardDays > 0 ? monthlyRate / standardDays : 0;
            const proratedAmount = dailyRate * totalPaidDays;

            return (
              <tr className='group' data-testid={`salary-component-${index}`} key={index}>
                <td className='py-4'>
                  <div className='flex items-center gap-3'>
                    <div className='w-1.5 h-1.5 bg-blue-600 rounded-full' />
                    <div>
                      <Typography className='font-bold text-slate-700 dark:text-slate-200' variant="p">{sc.definition.name}</Typography>
                      <Typography className='text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase' variant="p">
                        Code: {sc.definition.code} • Monthly: {formatAmount(monthlyRate)} • {totalPaidDays.toFixed(1)} Days @ {formatAmount(dailyRate)}/day
                      </Typography>
                    </div>
                  </div>
                </td>
                <td className='text-right py-4 font-bold text-slate-700 dark:text-slate-200'>
                  {formatAmount(proratedAmount)}
                </td>
              </tr>
            );
          })}

          {/* Base Salary Segments */}
          {(!payslip.salary_components || payslip.salary_components.length === 0) && payslip.segments?.map((seg, index: number) => (
            <tr className='group' data-testid={`salary-segment-${index}`} key={index}>
              <td className='py-4'>
                <div className='flex items-center gap-3'>
                  <div className='w-1.5 h-1.5 bg-blue-600 rounded-full' />
                  <div>
                    <Typography className='font-bold text-slate-700 dark:text-slate-200' variant="p">Base Salary {index > 0 ? ` (Segment ${index + 1})` : ''}</Typography>
                    <Typography className='text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase' variant="p">
                      {seg.start_date} - {seg.end_date} • {seg.paid_work_days.toFixed(1)} Days @ {formatAmount(seg.applied_base_salary)}
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
          {payslip.incentives?.map((inc, index: number) => (
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

interface PayslipTotalsProps {
  formatAmount: (value: string | number) => string;
  grossPay: number;
  payslip: Payslip;
}

function PayslipTotals({ payslip, grossPay, formatAmount }: PayslipTotalsProps) {
  return (
    <div className='pt-8 border-t-2 border-slate-900' data-testid="totals-section">
      <div className='grid grid-cols-[1fr_250px] gap-12'>
        <div className='space-y-4'>
            <Typography className='text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest italic' variant="p">
              Note: This is a computer-generated document and does not require a physical signature.
              Any discrepancies should be reported to the HR department within 3 working days.
            </Typography>
        </div>
        <div className='space-y-3'>
          <div className='flex justify-between text-sm'>
            <span className='font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest'>Gross Pay</span>
            <span className='font-bold text-slate-700 dark:text-slate-200' data-testid="gross-pay">{formatAmount(grossPay)}</span>
          </div>
          <div className='flex justify-between text-sm'>
            <span className='font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest'>Deductions</span>
            <span className='font-bold text-rose-600 dark:text-rose-300' data-testid="deductions">-{formatAmount(payslip.deductions_amount)}</span>
          </div>
          <div className='flex justify-between pt-4 border-t border-slate-100 dark:border-slate-800'>
            <span className='text-sm font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest'>Net Pay</span>
            <span className='text-xl font-bold text-slate-700 dark:text-slate-200' data-testid="net-pay-total">{formatAmount(payslip.net_pay_amount)}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function PayslipDocument({ employee, payslip, workspace }: PayslipDocumentProps) {
  const { t } = useTranslation();
  const { formatMonthYear, formatDate } = useDateTimeFormatter();
  const currencyFormatter = new Intl.NumberFormat('vi-VN', {
    style: 'currency',
    currency: 'VND',
    minimumFractionDigits: 0,
  });

  const formatAmount = (value: string | number) => {
    return currencyFormatter.format(typeof value === 'string' ? parseFloat(value) : value);
  };

  const totalIncentives = payslip.incentives?.reduce((accumulator, index) => accumulator + parseFloat(index.amount), 0) ?? 0;
  const grossPay = parseFloat(payslip.base_pay_amount) + parseFloat(payslip.overtime_pay_amount) + totalIncentives;

  return (
    <div className='bg-white dark:bg-slate-900 p-8 md:p-16 max-w-[210mm] mx-auto shadow-sm print:shadow-none print:p-0 text-slate-700 dark:text-slate-200 font-sans' data-testid="payslip-document" id='payslip-document'>
      <PayslipHeader formatDate={formatDate} formatMonthYear={formatMonthYear} payslip={payslip} t={t} workspace={workspace} />
      <EmployeeDetails employee={employee} formatAmount={formatAmount} payslip={payslip} t={t} />

      <div className='space-y-8' data-testid="earnings-breakdown">
        <EarningsBreakdown formatAmount={formatAmount} payslip={payslip} />
        <PayslipTotals formatAmount={formatAmount} grossPay={grossPay} payslip={payslip} />
      </div>

      <footer className='mt-20 pt-10 border-t border-slate-50 dark:border-slate-800 text-center space-y-2 opacity-50 print:opacity-100' data-testid="payslip-footer">
        <Typography className='text-[9px] font-bold uppercase tracking-[0.5em] text-slate-500 dark:text-slate-400' variant="p">Generated by LeeAttend System</Typography>
        <Typography className='text-[8px] font-bold text-slate-300 dark:text-slate-600 tracking-widest' variant="p">{new Date().toISOString()}</Typography>
      </footer>
    </div>
  );
}