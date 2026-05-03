import { ChevronLeft, ChevronRight } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

interface LeaveBalancesPaginationProps {
  limit: number;
  onPageChange: (page: number) => void;
  page: number;
  totalItems: number;
  totalPages: number;
}

export const LeaveBalancesPagination: React.FC<LeaveBalancesPaginationProps> = ({
  limit,
  onPageChange,
  page,
  totalItems,
  totalPages,
}) => {
  const { t } = useTranslation();

  if (totalItems <= 0) return null;

  return (
    <div className="p-5 md:p-8 border-t border-slate-50 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30 flex flex-col md:flex-row items-center justify-between gap-4">
      <div className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
        {t("common.showing")} <span className="text-slate-600 dark:text-slate-300">{(page - 1) * limit + 1}</span> - <span className="text-slate-600 dark:text-slate-300">{Math.min(page * limit, totalItems)}</span> {t("common.of")} <span className="text-slate-600 dark:text-slate-300">{totalItems}</span>
      </div>

      <div className="flex items-center gap-2">
        <button
          className="p-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          disabled={page === 1}
          onClick={() => { onPageChange(Math.max(1, page - 1)); }}
        >
          <ChevronLeft className="w-4 h-4 text-slate-600 dark:text-slate-300" />
        </button>

        <div className="flex items-center gap-1">
          {Array.from({ length: Math.min(5, totalPages) }, (_, index) => {
            let p = index + 1;
            if (totalPages > 5) {
              if (page > 3) p = page - 2 + index;
              if (p > totalPages) p = totalPages - (4 - index);
            }

            return (
              <button
                className={cn(
                  "w-8 h-8 flex items-center justify-center rounded-xl text-xs font-bold transition-all",
                  page === p
                    ? "bg-primary-600 text-white shadow-md shadow-primary-100"
                    : "bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800"
                )}
                key={p}
                onClick={() => { onPageChange(p); }}
              >
                {p}
              </button>
            );
          })}
        </div>

        <button
          className="p-2 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-xl hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          disabled={page === totalPages}
          onClick={() => { onPageChange(Math.min(totalPages, page + 1)); }}
        >
          <ChevronRight className="w-4 h-4 text-slate-600 dark:text-slate-300" />
        </button>
      </div>
    </div>
  );
};
