import type { TFunction } from 'i18next';

import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-react';
import React from 'react';
import { useTranslation } from 'react-i18next';

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

interface PaginationControlsProps {
  currentPage: number;
  onPageChange: (page: number) => void;
  onPageSizeChange?: (size: number) => void;
  pageSize: number;
  pageSizeOptions?: number[];
  totalItems: number;
  totalPages: number;
}

interface MobilePaginationProps {
  currentPage: number;
  onPageChange: (page: number) => void;
  t: TFunction;
  totalPages: number;
}

function MobilePagination({ currentPage, totalPages, onPageChange, t }: MobilePaginationProps) {
  return (
      <div className="flex justify-between flex-1 sm:hidden">
        <Button
          className="relative inline-flex items-center px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-700 dark:text-slate-200 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm dark:shadow-black/10 active:scale-95"
          disabled={currentPage === 1}
          onClick={() => { onPageChange(currentPage - 1); }}
        >
          {t('common.pagination.prev')}
        </Button>
        <Button
          className="relative ml-3 inline-flex items-center px-4 py-2 text-xs font-bold uppercase tracking-widest text-slate-700 dark:text-slate-200 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm active:scale-95"
          disabled={currentPage === totalPages}
          onClick={() => { onPageChange(currentPage + 1); }}
        >
          {t('common.pagination.next')}
        </Button>
      </div>
  );
}

interface DesktopPaginationProps {
  currentOptions: number[];
  currentPage: number;
  endItem: number;
  getPageNumbers: () => number[];
  onPageChange: (page: number) => void;
  onPageSizeChange?: (size: number) => void;
  pageSize: number;
  startItem: number;
  t: TFunction;
  totalItems: number;
  totalPages: number;
}

function DesktopPagination({
    currentPage,
    totalPages,
    onPageChange,
    onPageSizeChange,
    pageSize,
    totalItems,
    startItem,
    endItem,
    currentOptions,
    getPageNumbers,
    t
}: DesktopPaginationProps) {
  return (
      <div className="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between gap-8">
        <div className="flex items-center gap-6">
          <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest" variant="p">
            {t('common.pagination.showing')} <span className="text-slate-700 dark:text-slate-200 mx-1 font-mono">{startItem}</span> - <span className="text-slate-700 dark:text-slate-200 mx-1 font-mono">{endItem}</span> {t('common.pagination.out_of')} <span className="text-slate-700 dark:text-slate-200 mx-1 font-mono">{totalItems}</span> {t('common.pagination.records')}
          </Typography>
          {!!onPageSizeChange && (
            <div className="flex items-center gap-3">
              <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t('common.pagination.rows_per_page', 'Rows per page')}:</span>
              <select
                className="block py-1 pl-3 pr-8 text-[10px] font-bold uppercase tracking-widest border-slate-200 dark:border-slate-700 rounded-xl focus:ring-primary-500 focus:border-primary-500 cursor-pointer bg-white dark:bg-slate-900 shadow-sm"
                onChange={(e) => { onPageSizeChange(Number(e.target.value)); }}
                value={pageSize}
              >
                {currentOptions.map((option: number) => (
                  <option key={option} value={option}>
                    {option}
                  </option>
                ))}
              </select>
            </div>
          )}
        </div>
        <div>
          <nav aria-label={t("common.pagination.page")} className="relative z-0 inline-flex gap-1">
            <Button
              className="relative inline-flex items-center w-9 h-9 justify-center text-sm font-medium text-slate-500 dark:text-slate-400 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl hover:text-primary-600 dark:text-primary-400 hover:border-primary-200 dark:border-primary-800 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm active:scale-95"
              disabled={currentPage === 1}
              onClick={() => { onPageChange(1); }}
            >
              <span className="sr-only">{t("common.pagination.first")}</span>
              <ChevronsLeft className="w-4 h-4" />
            </Button>
            <Button
              className="relative inline-flex items-center w-9 h-9 justify-center text-sm font-medium text-slate-500 dark:text-slate-400 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl hover:text-primary-600 dark:text-primary-400 hover:border-primary-200 dark:border-primary-800 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm active:scale-95"
              disabled={currentPage === 1}
              onClick={() => { onPageChange(currentPage - 1); }}
            >
              <span className="sr-only">{t("common.pagination.prev")}</span>
              <ChevronLeft className="w-4 h-4" />
            </Button>
            
            <div className="flex items-center gap-1 mx-1">
              {getPageNumbers().map((page: number) => (
                <Button
                  className={cn(
                    "relative inline-flex items-center w-9 h-9 justify-center text-[10px] font-bold rounded-xl transition-all shadow-sm active:scale-95 cursor-pointer border",
                    currentPage === page
                      ? 'z-10 bg-primary-600 text-white border-primary-500 shadow-lg shadow-primary-100'
                      : 'bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-300 border-slate-200 dark:border-slate-700 hover:border-primary-200 dark:border-primary-800 hover:text-primary-600 dark:text-primary-400'
                  )}
                  key={page}
                  onClick={() => { onPageChange(page); }}
                >
                  {page}
                </Button>
              ))}
            </div>

            <Button
              className="relative inline-flex items-center w-9 h-9 justify-center text-sm font-medium text-slate-500 dark:text-slate-400 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl hover:text-primary-600 dark:text-primary-400 hover:border-primary-200 dark:border-primary-800 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm active:scale-95"
              disabled={currentPage === totalPages}
              onClick={() => { onPageChange(currentPage + 1); }}
            >
              <span className="sr-only">{t("common.pagination.next")}</span>
              <ChevronRight className="w-4 h-4" />
            </Button>
            <Button
              className="relative inline-flex items-center w-9 h-9 justify-center text-sm font-medium text-slate-500 dark:text-slate-400 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl hover:text-primary-600 dark:text-primary-400 hover:border-primary-200 dark:border-primary-800 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer transition-all shadow-sm active:scale-95"
              disabled={currentPage === totalPages}
              onClick={() => { onPageChange(totalPages); }}
            >
              <span className="sr-only">{t("common.pagination.last")}</span>
              <ChevronsRight className="w-4 h-4" />
            </Button>
          </nav>
        </div>
      </div>
  );
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

    for (let index = start; index <= end; index++) {
      pages.push(index);
    }
    return pages;
  };

  if (totalItems <= pageSize && currentPage === 1 && !onPageSizeChange) {
     return (
        <div className="flex items-center justify-between px-4 py-3 bg-white dark:bg-slate-900 border-t border-gray-200 dark:border-slate-700 sm:px-6">
            <div className="text-sm text-gray-700 dark:text-slate-200">
                {t('common.pagination.showing')} <span className="font-medium">{startItem}</span> - <span className="font-medium">{endItem}</span> {t('common.pagination.out_of')} <span className="font-medium">{totalItems}</span> {t('common.pagination.records')}
            </div>
        </div>
     );
  }

  return (
    <div className="flex items-center justify-between px-4 py-3 sm:px-6 w-full">
      <MobilePagination
        currentPage={currentPage}
        onPageChange={onPageChange}
        t={t}
        totalPages={totalPages}
      />
      <DesktopPagination
        currentOptions={currentOptions}
        currentPage={currentPage}
        endItem={endItem}
        getPageNumbers={getPageNumbers}
        onPageChange={onPageChange}
        onPageSizeChange={onPageSizeChange}
        pageSize={pageSize}
        startItem={startItem}
        t={t}
        totalItems={totalItems}
        totalPages={totalPages}
      />
    </div>
  );
};

export default PaginationControls;
