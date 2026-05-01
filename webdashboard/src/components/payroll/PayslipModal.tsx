import React, { useRef, useCallback } from 'react';

import * as Dialog from '@radix-ui/react-dialog';
import { X, Printer } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { useDateTimeFormatter } from '@/hooks/use-datetime';
import { Employee, Payslip, Workspace } from '@/lib/api/types';

import { useToast } from '@/components/ui/Toast';

import PayslipDocument from './PayslipDocument';

interface PayslipModalProps {
  isOpen: boolean;
  onClose: () => void;
  employee: Employee;
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
  const toast = useToast();
  const printRef = useRef<HTMLDivElement>(null);

  const handlePrint = useCallback(() => {
    if (!printRef.current) return;

    // Create a new window for printing
    const printWindow = window.open('', '_blank', 'width=600,height=800');
    if (!printWindow) {
      toast.error('Please allow pop-ups to print the payslip.');
      return;
    }

    // Get the payslip content
    const content = printRef.current.innerHTML;

    // Write the print document with A5 page size
    printWindow.document.write(`
      <!DOCTYPE html>
      <html>
        <head>
          <title>Payslip - ${employee.full_name}</title>
          <style>
            @page {
              size: A5 portrait;
              margin: 5mm;
            }

            * {
              margin: 0;
              padding: 0;
              box-sizing: border-box;
            }

            body {
              font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
              font-size: 7px;
              line-height: 1.3;
              color: #0f172a;
              background: white;
              -webkit-print-color-adjust: exact;
              print-color-adjust: exact;
            }

            /* A5 container - fit content on single page */
            .payslip-container {
              width: 138mm;
              max-height: 200mm;
              padding: 3mm;
              background: white;
              overflow: hidden;
            }

            /* Override PayslipDocument padding */
            #payslip-document {
              padding: 4mm !important;
              max-width: 100% !important;
            }

            /* Typography */
            .font-bold { font-weight: 600; }
            .font-bold { font-weight: 500; }
            .uppercase { text-transform: uppercase; }
            .italic { font-style: italic; }
            .text-center { text-align: center; }
            .text-right { text-align: right; }
            .text-left { text-align: left; }

            /* Text sizes - compact for A5 single page */
            .text-5xl { font-size: 16px; }
            .text-4xl { font-size: 12px; }
            .text-2xl { font-size: 10px; }
            .text-xl { font-size: 9px; }
            .text-sm { font-size: 7px; }
            [class*="text-[10px]"] { font-size: 6px; }
            [class*="text-[9px]"] { font-size: 5px; }
            [class*="text-[8px]"] { font-size: 4px; }

            /* Colors */
            .text-slate-700 dark:text-slate-200 { color: #0f172a; }
            .text-slate-500 dark:text-slate-400 { color: #64748b; }
            .text-slate-500 dark:text-slate-400 { color: #94a3b8; }
            .text-slate-300 dark:text-slate-600 { color: #cbd5e1; }
            .text-emerald-700 dark:text-emerald-300 { color: #047857; }
            .text-emerald-600 dark:text-emerald-300 { color: #059669; }
            .text-rose-600 dark:text-rose-300 { color: #e11d48; }
            .text-amber-700 dark:text-amber-300 { color: #b45309; }

            .bg-white dark:bg-slate-900 { background: white; }
            .bg-slate-900 { background: #0f172a; }
            .bg-slate-50 dark:bg-slate-800 { background: #f8fafc; }
            .bg-emerald-50 dark:bg-emerald-950 { background: #ecfdf5; }
            .bg-amber-50 dark:bg-amber-950 { background: #fffbeb; }

            /* Layout */
            .flex { display: flex; }
            .grid { display: grid; }
            .items-center { align-items: center; }
            .items-start { align-items: start; }
            .justify-between { justify-content: space-between; }
            .justify-end { justify-content: flex-end; }
            .gap-2 { gap: 2px; }
            .gap-3 { gap: 3px; }
            .gap-4 { gap: 4px; }
            .gap-6 { gap: 6px; }
            .gap-12 { gap: 10px; }
            .space-y-1 > * + * { margin-top: 1px; }
            .space-y-2 > * + * { margin-top: 2px; }
            .space-y-3 > * + * { margin-top: 3px; }
            .space-y-4 > * + * { margin-top: 4px; }
            .space-y-6 > * + * { margin-top: 6px; }
            .space-y-8 > * + * { margin-top: 8px; }

            .grid-cols-2 { grid-template-columns: repeat(2, 1fr); }
            [class*="grid-cols-[100px"] { grid-template-columns: 45px 1fr; }
            [class*="grid-cols-[1fr_250px]"] { grid-template-columns: 1fr 80px; }

            /* Compact spacing for single page */
            .p-3 { padding: 3px; }
            .p-4 { padding: 4px; }
            .p-8 { padding: 6px; }
            .p-16 { padding: 8px; }
            .px-3 { padding-left: 3px; padding-right: 3px; }
            .py-1 { padding-top: 1px; padding-bottom: 1px; }
            .py-4 { padding-top: 3px; padding-bottom: 3px; }
            .pb-2 { padding-bottom: 2px; }
            .pb-4 { padding-bottom: 4px; }
            .pb-10 { padding-bottom: 8px; }
            .pt-2 { padding-top: 2px; }
            .pt-4 { padding-top: 4px; }
            .pt-8 { padding-top: 6px; }
            .pt-10 { padding-top: 8px; }
            .mb-1 { margin-bottom: 1px; }
            .mb-4 { margin-bottom: 4px; }
            .mb-6 { margin-bottom: 6px; }
            .mb-10 { margin-bottom: 8px; }
            .mb-12 { margin-bottom: 10px; }
            .mt-2 { margin-top: 2px; }
            .mt-20 { margin-top: 12px; }

            /* Borders */
            .border { border: 1px solid #e2e8f0; }
            .border-t { border-top: 1px solid #e2e8f0; }
            .border-b { border-bottom: 1px solid #e2e8f0; }
            .border-b-2 { border-bottom: 1px solid; }
            .border-t-2 { border-top: 1px solid; }
            .border-slate-900 { border-color: #0f172a; }
            .border-slate-100 dark:border-slate-800 { border-color: #f1f5f9; }
            .border-slate-200 dark:border-slate-700 { border-color: #e2e8f0; }
            .border-slate-50 dark:border-slate-800 { border-color: #f8fafc; }
            .border-emerald-100 dark:border-emerald-800 { border-color: #d1fae5; }
            .border-amber-100 dark:border-amber-800 { border-color: #fef3c7; }

            /* Border radius - smaller for compact */
            .rounded-full { border-radius: 9999px; }
            .rounded-xl { border-radius: 4px; }
            .rounded-3xl { border-radius: 8px; }

            /* Icons - smaller */
            .w-12 { width: 16px; }
            .h-12 { height: 16px; }
            [class*="w-1.5"] { width: 2px; }
            [class*="h-1.5"] { height: 2px; }

            /* Table */
            table { width: 100%; border-collapse: collapse; }
            .divide-y > * + * { border-top: 1px solid #f8fafc; }

            /* Tracking - reduced */
            .tracking-tight { letter-spacing: -0.01em; }
            .tracking-tighter { letter-spacing: -0.02em; }
            .tracking-widest { letter-spacing: 0.05em; }
            [class*="tracking-[0.2em]"] { letter-spacing: 0.1em; }
            [class*="tracking-[0.3em]"] { letter-spacing: 0.12em; }
            [class*="tracking-[0.5em]"] { letter-spacing: 0.15em; }

            .leading-none { line-height: 1; }

            /* Hide SVG icons for print */
            svg { display: none; }

            /* Opacity */
            .opacity-50 { opacity: 1; }

            /* Hide footer for compact fit */
            footer { margin-top: 6px !important; padding-top: 4px !important; }
          </style>
        </head>
        <body>
          <div class="payslip-container">
            ${content}
          </div>
        </body>
      </html>
    `);

    printWindow.document.close();

    // Wait for content to load, then print
    printWindow.onload = () => {
      printWindow.focus();
      printWindow.print();
      printWindow.close();
    };

    // Fallback for browsers that don't fire onload
    setTimeout(() => {
      if (!printWindow.closed) {
        printWindow.focus();
        printWindow.print();
        printWindow.close();
      }
    }, 500);
  }, [employee.full_name]);

  return (
    <Dialog.Root open={isOpen} onOpenChange={onClose}>
      <Dialog.Portal>
        <Dialog.Overlay className='fixed inset-0 bg-slate-900/60 backdrop-blur-sm z-[100] animate-in fade-in duration-300 print:hidden' />
        <Dialog.Content data-testid="payslip-modal" className='fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-[900px] max-h-[90vh] bg-slate-100 dark:bg-slate-700 rounded-[2rem] shadow-2xl z-[101] overflow-hidden flex flex-col animate-in zoom-in-95 duration-300 print:static print:translate-x-0 print:translate-y-0 print:max-h-none print:w-auto print:bg-white dark:bg-slate-900 print:shadow-none print:rounded-none'>
          
          <Dialog.Title className="sr-only">Official Payslip Preview</Dialog.Title>
          <Dialog.Description className="sr-only">
            Preview of the payslip for {employee.full_name} for the period starting {formatMonthYear(payslip.period_start)}
          </Dialog.Description>

          {/* Action Bar */}
          <div className='p-6 bg-white dark:bg-slate-900 border-b border-slate-200 dark:border-slate-700 flex items-center justify-between print:hidden'>
            <div className='flex items-center gap-4'>
              <div>
                <h2 className='text-sm font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400'>Official Payslip Preview</h2>
                <p className='text-xs font-bold text-slate-600 dark:text-slate-300 mt-1'>
                  {t('common.month')} {formatMonthYear(payslip.period_start)}
                </p>
              </div>
            </div>
            <div className='flex items-center gap-3'>
              <button
                onClick={handlePrint}
                data-testid="payslip-print-btn"
                className='flex items-center gap-2 px-6 py-2.5 bg-slate-900 text-white rounded-xl text-[10px] font-bold uppercase tracking-widest hover:bg-slate-800 transition-all shadow-lg active:scale-95'
              >
                <Printer size={14} /> Print Payslip
              </button>
              <Dialog.Close asChild>
                <button data-testid="payslip-close-btn" className='w-10 h-10 rounded-xl bg-slate-100 dark:bg-slate-700 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 transition-all'>
                  <X size={20} />
                </button>
              </Dialog.Close>
            </div>
          </div>

          {/* Scrollable Document Area */}
          <div className='flex-1 overflow-y-auto p-4 md:p-12 print:p-0 print:overflow-visible'>
             <div ref={printRef} className='bg-white dark:bg-slate-900 rounded-2xl shadow-sm overflow-hidden print:shadow-none print:rounded-none'>
                <PayslipDocument
                  employee={employee}
                  payslip={payslip}
                  workspace={workspace}
                />
             </div>
          </div>
          
          {/* Footer Warning */}
          <div className='p-4 bg-amber-50 dark:bg-amber-950 border-t border-amber-100 dark:border-amber-800 text-center print:hidden'>
            <p className='text-[9px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-widest'>
              Use the print button above to save as PDF or print a physical copy.
            </p>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}