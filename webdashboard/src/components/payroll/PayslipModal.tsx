import React from 'react';
import { useTranslation } from 'react-i18next';
import * as Dialog from '@radix-ui/react-dialog';
import { X, Printer, Download } from 'lucide-react';
import { Employee, Payslip, Workspace } from '@/lib/api/types';
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

  const handlePrint = () => {
    window.print();
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={onClose}>
      <Dialog.Portal>
        <Dialog.Overlay className='fixed inset-0 bg-slate-900/60 backdrop-blur-sm z-[100] animate-in fade-in duration-300 print:hidden' />
        <Dialog.Content className='fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-[900px] max-h-[90vh] bg-slate-100 rounded-[2rem] shadow-2xl z-[101] overflow-hidden flex flex-col animate-in zoom-in-95 duration-300 print:static print:translate-x-0 print:translate-y-0 print:max-h-none print:w-auto print:bg-white print:shadow-none print:rounded-none'>
          
          {/* Action Bar */}
          <div className='p-6 bg-white border-b border-slate-200 flex items-center justify-between print:hidden'>
            <div className='flex items-center gap-4'>
              <h2 className='text-sm font-black uppercase tracking-widest text-slate-400'>Official Payslip Preview</h2>
            </div>
            <div className='flex items-center gap-3'>
              <button 
                onClick={handlePrint}
                className='flex items-center gap-2 px-6 py-2.5 bg-slate-900 text-white rounded-xl text-[10px] font-black uppercase tracking-widest hover:bg-slate-800 transition-all shadow-lg active:scale-95'
              >
                <Printer size={14} /> Print Payslip
              </button>
              <Dialog.Close asChild>
                <button className='w-10 h-10 rounded-xl bg-slate-100 flex items-center justify-center text-slate-400 hover:text-rose-600 transition-all'>
                  <X size={20} />
                </button>
              </Dialog.Close>
            </div>
          </div>

          {/* Scrollable Document Area */}
          <div className='flex-1 overflow-y-auto p-4 md:p-12 print:p-0 print:overflow-visible'>
             <div className='bg-white rounded-2xl shadow-sm overflow-hidden print:shadow-none print:rounded-none'>
                <PayslipDocument 
                  employee={employee}
                  payslip={payslip}
                  workspace={workspace}
                />
             </div>
          </div>
          
          {/* Footer Warning */}
          <div className='p-4 bg-amber-50 border-t border-amber-100 text-center print:hidden'>
            <p className='text-[9px] font-bold text-amber-700 uppercase tracking-widest'>
              Use the print button above to save as PDF or print a physical copy.
            </p>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}