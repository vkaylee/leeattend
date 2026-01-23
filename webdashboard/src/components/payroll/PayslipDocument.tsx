import React from 'react';
import { useTranslation } from 'react-i18next';
import { Employee, Payslip, Workspace } from '@/lib/api/types';
import { useDateTimeFormatter } from '@/hooks/use-datetime';
import { 
  Building2, 
  CalendarDays, 
  ShieldCheck
} from 'lucide-react';

interface PayslipDocumentProps {
  employee: Employee;
  payslip: Payslip;
  workspace: Workspace | null;
}

export default function PayslipDocument({ employee, payslip, workspace }: PayslipDocumentProps) {
  const { t } = useTranslation();
  const { formatMonthYear, formatDate } = useDateTimeFormatter();
  const currencyFormatter = new Intl.NumberFormat('vi-VN', {
    style: 'currency',
    currency: 'VND',
    minimumFractionDigits: 0,
  });

  const formatAmount = (val: string | number) => {
    return currencyFormatter.format(typeof val === 'string' ? parseFloat(val) : val);
  };

  const totalIncentives = payslip.incentives?.reduce((acc, i) => acc + parseFloat(i.amount), 0) || 0;
  const grossPay = parseFloat(payslip.base_pay_amount) + parseFloat(payslip.overtime_pay_amount) + totalIncentives;

  return (
    <div className='bg-white p-8 md:p-16 max-w-[210mm] mx-auto shadow-sm print:shadow-none print:p-0 text-slate-900 font-sans' id='payslip-document'>
      {/* Document Header */}
      <div className='flex justify-between items-start border-b-2 border-slate-900 pb-10 mb-10'>
        <div className='space-y-4'>
          <div className='flex items-center gap-3'>
            <div className='w-12 h-12 bg-slate-900 text-white rounded-xl flex items-center justify-center'>
              <Building2 size={24} />
            </div>
            <div>
              <h1 className='text-2xl font-black uppercase tracking-tight'>{workspace?.name || 'LeeAttend'}</h1>
              <p className='text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em]'>Attendance & Payroll System</p>
            </div>
          </div>
          <div className='inline-flex items-center gap-2 px-3 py-1 bg-emerald-50 text-emerald-700 rounded-full border border-emerald-100 print:bg-white print:border-slate-200'>
            <ShieldCheck size={12} />
            <span className='text-[10px] font-black uppercase tracking-widest'>Verified Document</span>
          </div>
        </div>
        
        <div className='text-right'>
          <h2 className='text-4xl font-black uppercase tracking-tighter text-slate-900 mb-1'>Pay Slip</h2>
          <div className='flex items-center justify-end gap-2 text-slate-500'>
            <CalendarDays size={14} />
            <span className='text-sm font-bold uppercase tracking-widest'>
              {t('common.month')} {formatMonthYear(payslip.period_start)}
            </span>
          </div>
          <p className='text-[10px] font-bold text-slate-400 mt-2 uppercase tracking-widest'>
            Period: {formatDate(payslip.period_start)} - {formatDate(payslip.period_end)}
          </p>
        </div>
      </div>

      {/* Employee & Summary Info */}
      <div className='grid grid-cols-2 gap-12 mb-12'>
        <div className='space-y-6'>
          <h3 className='text-[10px] font-black uppercase tracking-[0.3em] text-slate-400 border-b border-slate-100 pb-2'>Employee Details</h3>
          <div className='grid grid-cols-[100px_1fr] gap-y-4 text-sm'>
            <span className='font-bold text-slate-400 uppercase text-[10px] tracking-widest'>Name</span>
            <span className='font-black text-slate-900 uppercase'>{employee.full_name}</span>
            
            <span className='font-bold text-slate-400 uppercase text-[10px] tracking-widest'>Employee ID</span>
            <span className='font-bold text-slate-900'>{employee.employee_id_number}</span>
            
            <span className='font-bold text-slate-400 uppercase text-[10px] tracking-widest'>Department</span>
            <span className='font-bold text-slate-900'>{employee.department_name || 'N/A'}</span>
          </div>
        </div>
        
        <div className='bg-slate-50 rounded-3xl p-8 border border-slate-100 print:bg-white print:border-slate-200'>
          <h3 className='text-[10px] font-black uppercase tracking-[0.3em] text-slate-400 mb-6 text-center'>Net Payment</h3>
          <div className='text-center space-y-1'>
            <p className='text-5xl font-black text-slate-900 tracking-tighter leading-none'>
              {formatAmount(payslip.net_pay_amount)}
            </p>
            <p className='text-[10px] font-black text-emerald-600 uppercase tracking-[0.2em] pt-2'>Paid via Bank Transfer</p>
          </div>
        </div>
      </div>

      {/* Main Breakdown Table */}
      <div className='space-y-8'>
        <div>
          <h3 className='text-[10px] font-black uppercase tracking-[0.3em] text-slate-400 border-b border-slate-100 pb-4 mb-4'>Earnings Breakdown</h3>
          <table className='w-full text-sm'>
            <thead>
              <tr className='text-[10px] font-black uppercase tracking-widest text-slate-400'>
                <th className='text-left pb-4'>Description</th>
                <th className='text-right pb-4'>Amount</th>
              </tr>
            </thead>
            <tbody className='divide-y divide-slate-50'>
              {/* Salary Components Breakdown */}
              {payslip.salary_components?.map((sc, idx) => {
                const monthlyRate = parseFloat(sc.component.amount);
                const standardDays = payslip.segments?.[0]?.applied_std_days || 22;
                const totalPaidDays = payslip.segments?.reduce((sum, seg) => sum + seg.paid_work_days, 0) || 0;
                const dailyRate = monthlyRate / standardDays;
                const proratedAmount = dailyRate * totalPaidDays;

                return (
                  <tr key={idx} className='group'>
                    <td className='py-4'>
                      <div className='flex items-center gap-3'>
                        <div className='w-1.5 h-1.5 bg-blue-600 rounded-full' />
                        <div>
                          <p className='font-bold text-slate-900'>{sc.definition.name}</p>
                          <p className='text-[10px] text-slate-400 font-bold uppercase'>
                            Code: {sc.definition.code} • Monthly: {formatAmount(monthlyRate)} • {totalPaidDays.toFixed(1)} Days @ {formatAmount(dailyRate)}/day
                          </p>
                        </div>
                      </div>
                    </td>
                    <td className='text-right py-4 font-black text-slate-900'>
                      {formatAmount(proratedAmount)}
                    </td>
                  </tr>
                );
              })}

              {/* Base Salary Segments (if no components available) */}
              {(!payslip.salary_components || payslip.salary_components.length === 0) && payslip.segments?.map((seg, idx) => (
                <tr key={idx} className='group'>
                  <td className='py-4'>
                    <div className='flex items-center gap-3'>
                      <div className='w-1.5 h-1.5 bg-blue-600 rounded-full' />
                      <div>
                        <p className='font-bold text-slate-900'>Base Salary {idx > 0 ? ` (Segment ${idx + 1})` : ''}</p>
                        <p className='text-[10px] text-slate-400 font-bold uppercase'>
                          {seg.start_date} - {seg.end_date} • {seg.paid_work_days.toFixed(1)} Days @ {formatAmount(seg.applied_base_salary)}
                        </p>
                      </div>
                    </div>
                  </td>
                  <td className='text-right py-4 font-black text-slate-900'>
                    {formatAmount(seg.amount)}
                  </td>
                </tr>
              ))}
              
              {/* Overtime */}
              {parseFloat(payslip.overtime_pay_amount) > 0 && (
                <tr>
                  <td className='py-4'>
                    <div className='flex items-center gap-3'>
                      <div className='w-1.5 h-1.5 bg-emerald-500 rounded-full' />
                      <div>
                        <p className='font-bold text-slate-900 uppercase'>Overtime Compensation</p>
                        <p className='text-[10px] text-slate-400 font-bold uppercase'>{payslip.overtime_units} Units</p>
                      </div>
                    </div>
                  </td>
                  <td className='text-right py-4 font-black text-slate-900'>
                    {formatAmount(payslip.overtime_pay_amount)}
                  </td>
                </tr>
              )}

              {/* Incentives */}
              {payslip.incentives?.map((inc, i) => (
                <tr key={i}>
                  <td className='py-4'>
                    <div className='flex items-center gap-3'>
                      <div className='w-1.5 h-1.5 bg-amber-400 rounded-full' />
                      <p className='font-bold text-slate-900 uppercase'>{inc.policy_name}</p>
                    </div>
                  </td>
                  <td className='text-right py-4 font-black text-slate-900'>
                    {formatAmount(inc.amount)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {/* Totals Section */}
        <div className='pt-8 border-t-2 border-slate-900'>
          <div className='grid grid-cols-[1fr_250px] gap-12'>
            <div className='space-y-4'>
               <p className='text-[10px] font-bold text-slate-400 uppercase tracking-widest italic'>
                 Note: This is a computer-generated document and does not require a physical signature. 
                 Any discrepancies should be reported to the HR department within 3 working days.
               </p>
            </div>
            <div className='space-y-3'>
              <div className='flex justify-between text-sm'>
                <span className='font-bold text-slate-400 uppercase tracking-widest'>Gross Pay</span>
                <span className='font-black text-slate-900'>{formatAmount(grossPay)}</span>
              </div>
              <div className='flex justify-between text-sm'>
                <span className='font-bold text-slate-400 uppercase tracking-widest'>Deductions</span>
                <span className='font-black text-rose-600'>-{formatAmount(payslip.deductions_amount)}</span>
              </div>
              <div className='flex justify-between pt-4 border-t border-slate-100'>
                <span className='text-sm font-black text-slate-900 uppercase tracking-widest'>Net Pay</span>
                <span className='text-xl font-black text-slate-900'>{formatAmount(payslip.net_pay_amount)}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
      
      {/* Footer */}
      <footer className='mt-20 pt-10 border-t border-slate-50 text-center space-y-2 opacity-50 print:opacity-100'>
        <p className='text-[9px] font-black uppercase tracking-[0.5em] text-slate-400'>Generated by LeeAttend System</p>
        <p className='text-[8px] font-bold text-slate-300 tracking-widest'>{new Date().toISOString()}</p>
      </footer>
    </div>
  );
}