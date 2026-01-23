import React from 'react';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from "@/lib/utils";

interface PaginationControlsProps {
  currentPage: number;
  totalPages: number;
  onPageChange: (page: number) => void;
  pageSize: number;
  totalItems: number;
  onPageSizeChange?: (size: number) => void;
  pageSizeOptions?: number[];
}

const PaginationControls: React.FC<PaginationControlsProps> = ({
  currentPage,
  totalPages,
  onPageChange,
  pageSize,
  totalItems,
  onPageSizeChange,
  pageSizeOptions = [10, 20, 50, 100],
}) => {
  const { t } = useTranslation();
  const startItem = totalItems === 0 ? 0 : (currentPage - 1) * pageSize + 1;
  const endItem = Math.min(currentPage * pageSize, totalItems);

  // Ensure current pageSize is in options if it's a custom value (like 28, 30, 31)
  const currentOptions = [...new Set([...pageSizeOptions, pageSize])].sort((a, b) => a - b);

  const getPageNumbers = () => {
    const pages = [];
    const maxVisible = 5;
    
    let start = Math.max(1, currentPage - Math.floor(maxVisible / 2));
    const end = Math.min(totalPages, start + maxVisible - 1);
    
    if (end - start + 1 < maxVisible) {
      start = Math.max(1, end - maxVisible + 1);
    }

    for (let i = start; i <= end; i++) {
      pages.push(i);
    }
    return pages;
  };

  if (totalItems <= pageSize && currentPage === 1 && !onPageSizeChange) {
     return (
        <div className="flex items-center justify-between px-4 py-3 bg-white border-t border-gray-200 sm:px-6">
            <div className="text-sm text-gray-700">
                {t('common.pagination.showing')} <span className="font-medium">{startItem}</span> - <span className="font-medium">{endItem}</span> {t('common.pagination.out_of')} <span className="font-medium">{totalItems}</span> {t('common.pagination.records')}
            </div>
        </div>
     );
  }

  return (
    <div className="flex items-center justify-between px-4 py-3 sm:px-6 w-full">
      <div className="flex justify-between flex-1 sm:hidden">
        <button
          onClick={() => onPageChange(currentPage - 1)}
          disabled={currentPage === 1}
          className="relative inline-flex items-center px-4 py-2 text-xs font-black uppercase tracking-widest text-slate-700 bg-white border border-slate-200 rounded-xl hover:bg-slate-50 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm active:scale-95"
        >
          {t('common.pagination.prev')}
        </button>
        <button
          onClick={() => onPageChange(currentPage + 1)}
          disabled={currentPage === totalPages}
          className="relative ml-3 inline-flex items-center px-4 py-2 text-xs font-black uppercase tracking-widest text-slate-700 bg-white border border-slate-200 rounded-xl hover:bg-slate-50 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm active:scale-95"
        >
          {t('common.pagination.next')}
        </button>
      </div>
      <div className="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between gap-8">
        <div className="flex items-center gap-6">
          <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest">
            {t('common.pagination.showing')} <span className="text-slate-900 mx-1 font-mono">{startItem}</span> - <span className="text-slate-900 mx-1 font-mono">{endItem}</span> {t('common.pagination.out_of')} <span className="text-slate-900 mx-1 font-mono">{totalItems}</span> {t('common.pagination.records')}
          </p>
          {onPageSizeChange && (
            <div className="flex items-center gap-3">
              <span className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t('common.pagination.rows_per_page', 'Rows per page')}:</span>
              <select
                value={pageSize}
                onChange={(e) => onPageSizeChange(Number(e.target.value))}
                className="block py-1 pl-3 pr-8 text-[10px] font-black uppercase tracking-widest border-slate-200 rounded-xl focus:ring-primary-500 focus:border-primary-500 cursor-pointer bg-white shadow-sm"
              >
                {currentOptions.map((option) => (
                  <option key={option} value={option}>
                    {option}
                  </option>
                ))}
              </select>
            </div>
          )}
        </div>
        <div>
          <nav className="relative z-0 inline-flex gap-1" aria-label="Pagination">
            <button
              onClick={() => onPageChange(1)}
              disabled={currentPage === 1}
              className="relative inline-flex items-center w-9 h-9 justify-center text-sm font-medium text-slate-400 bg-white border border-slate-200 rounded-xl hover:text-primary-600 hover:border-primary-200 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm active:scale-95"
            >
              <span className="sr-only">First</span>
              <ChevronsLeft className="w-4 h-4" />
            </button>
            <button
              onClick={() => onPageChange(currentPage - 1)}
              disabled={currentPage === 1}
              className="relative inline-flex items-center w-9 h-9 justify-center text-sm font-medium text-slate-400 bg-white border border-slate-200 rounded-xl hover:text-primary-600 hover:border-primary-200 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm active:scale-95"
            >
              <span className="sr-only">Previous</span>
              <ChevronLeft className="w-4 h-4" />
            </button>
            
            <div className="flex items-center gap-1 mx-1">
              {getPageNumbers().map((page) => (
                <button
                  key={page}
                  onClick={() => onPageChange(page)}
                  className={cn(
                    "relative inline-flex items-center w-9 h-9 justify-center text-[10px] font-black rounded-xl transition-all shadow-sm active:scale-95 cursor-pointer border",
                    currentPage === page
                      ? 'z-10 bg-primary-600 text-white border-primary-500 shadow-lg shadow-primary-100'
                      : 'bg-white text-slate-600 border-slate-200 hover:border-primary-200 hover:text-primary-600'
                  )}
                >
                  {page}
                </button>
              ))}
            </div>

            <button
              onClick={() => onPageChange(currentPage + 1)}
              disabled={currentPage === totalPages}
              className="relative inline-flex items-center w-9 h-9 justify-center text-sm font-medium text-slate-400 bg-white border border-slate-200 rounded-xl hover:text-primary-600 hover:border-primary-200 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm active:scale-95"
            >
              <span className="sr-only">Next</span>
              <ChevronRight className="w-4 h-4" />
            </button>
            <button
              onClick={() => onPageChange(totalPages)}
              disabled={currentPage === totalPages}
              className="relative inline-flex items-center w-9 h-9 justify-center text-sm font-medium text-slate-400 bg-white border border-slate-200 rounded-xl hover:text-primary-600 hover:border-primary-200 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm active:scale-95"
            >
              <span className="sr-only">Last</span>
              <ChevronsRight className="w-4 h-4" />
            </button>
          </nav>
        </div>
      </div>
    </div>
  );
};

export default PaginationControls;
