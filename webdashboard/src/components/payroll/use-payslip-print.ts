import { useCallback } from 'react';

import { useToast } from '@/components/ui/toast';

const PAYSLIP_PRINT_CSS = `
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
  .text-slate-700, .dark\\:text-slate-200 { color: #0f172a; }
  .text-slate-500, .dark\\:text-slate-400 { color: #64748b; }
  .text-slate-300, .dark\\:text-slate-600 { color: #cbd5e1; }
  .text-emerald-700, .dark\\:text-emerald-300 { color: #047857; }
  .text-emerald-600, .dark\\:text-emerald-300 { color: #059669; }
  .text-rose-600, .dark\\:text-rose-300 { color: #e11d48; }
  .text-amber-700, .dark\\:text-amber-300 { color: #b45309; }

  .bg-white, .dark\\:bg-slate-900 { background: white; }
  .bg-slate-900 { background: #0f172a; }
  .bg-slate-50, .dark\\:bg-slate-800 { background: #f8fafc; }
  .bg-emerald-50, .dark\\:bg-emerald-950 { background: #ecfdf5; }
  .bg-amber-50, .dark\\:bg-amber-950 { background: #fffbeb; }

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
  .border-slate-100, .dark\\:border-slate-800 { border-color: #f1f5f9; }
  .border-slate-200, .dark\\:border-slate-700 { border-color: #e2e8f0; }
  .border-slate-50, .dark\\:border-slate-800 { border-color: #f8fafc; }
  .border-emerald-100, .dark\\:border-emerald-800 { border-color: #d1fae5; }
  .border-amber-100, .dark\\:border-amber-800 { border-color: #fef3c7; }

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
`;

export function usePayslipPrint(
  printRef: React.RefObject<HTMLDivElement | null>,
  employeeName: string
) {
  const toast = useToast();

  return useCallback(() => {
    if (!printRef.current) return;

    const printWindow = window.open('', '_blank', 'width=600,height=800');
    if (!printWindow) {
      toast.error('Please allow pop-ups to print the payslip.');
      return;
    }

    const content = printRef.current.innerHTML;

    printWindow.document.title = `Payslip - ${employeeName}`;
    const style = printWindow.document.createElement('style');
    style.textContent = PAYSLIP_PRINT_CSS;
    printWindow.document.head.appendChild(style);

    printWindow.document.body.innerHTML = `
      <div class="payslip-container">
        ${content}
      </div>
    `;
    printWindow.document.close();

    printWindow.onload = () => {
      printWindow.focus();
      printWindow.print();
      printWindow.close();
    };

    setTimeout(() => {
      if (!printWindow.closed) {
        printWindow.focus();
        printWindow.print();
        printWindow.close();
      }
    }, 500);
  }, [employeeName, toast, printRef]);
}
