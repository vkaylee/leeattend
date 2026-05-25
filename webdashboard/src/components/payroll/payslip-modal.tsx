import * as Dialog from '@radix-ui/react-dialog';
import { X, Printer } from 'lucide-react';
import React, { useRef } from 'react';
import { useTranslation } from 'react-i18next';

import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from '@/hooks/use-datetime';
import { Employee, Payslip, Workspace } from '@/lib/api/types';

import PayslipDocument from './payslip-document';
import { usePayslipPrint } from './use-payslip-print';

interface PayslipModalProps {
  employee: Employee;
  isOpen: boolean;
  onClose: () => void;
  payslip: Payslip;
  workspace: Workspace | null;
}

export default function PayslipModal({
  isOpen,
  onClose,
  employee,
  payslip,
  workspace
}: PayslipModalProps) {
  const { t } = useTranslation();
  const { formatMonthYear } = useDateTimeFormatter();
  const printRef = useRef<HTMLDivElement>(null);

  const handlePrint = usePayslipPrint(printRef, employee.full_name);

  return (
    <Dialog.Root onOpenChange={onClose} open={isOpen}>
      <Dialog.Portal>
        <Dialog.Overlay className='fixed inset-0 bg-slate-900/60 backdrop-blur-sm z-[100] animate-in fade-in duration-300 print:hidden' />
        <Dialog.Content className='fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-[900px] max-h-[90vh] bg-slate-100 dark:bg-slate-700 rounded-[2rem] shadow-2xl z-[101] overflow-hidden flex flex-col animate-in zoom-in-95 duration-300 print:static print:translate-x-0 print:translate-y-0 print:max-h-none print:w-auto print:bg-white dark:bg-slate-900 print:shadow-none print:rounded-none' data-testid="payslip-modal">
          
          <Dialog.Title className="sr-only">Official Payslip Preview</Dialog.Title>
          <Dialog.Description className="sr-only">
            Preview of the payslip for {employee.full_name} for the period starting {formatMonthYear(payslip.period_start)}
          </Dialog.Description>

          {/* Action Bar */}
          <div className='p-6 bg-white dark:bg-slate-900 border-b border-slate-200 dark:border-slate-700 flex items-center justify-between print:hidden'>
            <div className='flex items-center gap-4'>
              <div>
                <Typography className='text-sm font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400' variant="h2">Official Payslip Preview</Typography>
                <Typography className='text-xs font-bold text-slate-600 dark:text-slate-300 mt-1' variant="p">
                  {t('common.month')} {formatMonthYear(payslip.period_start)}
                </Typography>
              </div>
            </div>
            <div className='flex items-center gap-3'>
              <button
                className='flex items-center gap-2 px-6 py-2.5 bg-slate-900 text-white rounded-xl text-[10px] font-bold uppercase tracking-widest hover:bg-slate-800 transition-all shadow-lg active:scale-95'
                data-testid="payslip-print-btn"
                onClick={handlePrint}
              >
                <Printer size={14} /> Print Payslip
              </button>
              <Dialog.Close asChild>
                <button className='w-10 h-10 rounded-xl bg-slate-100 dark:bg-slate-700 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 transition-all' data-testid="payslip-close-btn">
                  <X size={20} />
                </button>
              </Dialog.Close>
            </div>
          </div>

          {/* Scrollable Document Area */}
          <div className='flex-1 overflow-y-auto p-4 md:p-12 print:p-0 print:overflow-visible'>
             <div className='bg-white dark:bg-slate-900 rounded-2xl shadow-sm overflow-hidden print:shadow-none print:rounded-none' ref={printRef}>
                <PayslipDocument
                  employee={employee}
                  payslip={payslip}
                  workspace={workspace}
                />
             </div>
          </div>
          
          {/* Footer Warning */}
          <div className='p-4 bg-amber-50 dark:bg-amber-950 border-t border-amber-100 dark:border-amber-800 text-center print:hidden'>
            <Typography className='text-[9px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest' variant="p">
              Use the print button above to save as PDF or print a physical copy.
            </Typography>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}