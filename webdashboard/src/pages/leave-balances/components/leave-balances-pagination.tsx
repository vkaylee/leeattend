import { ChevronLeft, ChevronRight } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";

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
        <Button
          className="h-8 w-8 p-0 rounded-xl"
          disabled={page === 1}
          onClick={() => { onPageChange(Math.max(1, page - 1)); }}
          variant="outline"
        >
          <ChevronLeft className="w-4 h-4 text-slate-600 dark:text-slate-300" />
        </Button>

        <div className="flex items-center gap-1">
          {Array.from({ length: Math.min(5, totalPages) }, (_, index) => {
            let p = index + 1;
            if (totalPages > 5) {
              if (page > 3) p = page - 2 + index;
              if (p > totalPages) p = totalPages - (4 - index);
            }

            return (
              <Button
                className="h-8 w-8 rounded-xl text-xs"
                key={p}
                onClick={() => { onPageChange(p); }}
                variant={page === p ? "primary" : "outline"}
              >
                {p}
              </Button>
            );
          })}
        </div>

        <Button
          className="h-8 w-8 p-0 rounded-xl"
          disabled={page === totalPages}
          onClick={() => { onPageChange(Math.min(totalPages, page + 1)); }}
          variant="outline"
        >
          <ChevronRight className="w-4 h-4 text-slate-600 dark:text-slate-300" />
        </Button>
      </div>
    </div>
  );
};

