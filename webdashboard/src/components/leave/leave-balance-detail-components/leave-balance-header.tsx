import { X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { WorkspaceLeaveBalance } from "@/lib/api";

export function LeaveBalanceHeader({ balance, onClose }: { balance: WorkspaceLeaveBalance; onClose: () => void }) {
  const { t } = useTranslation();
  return (
    <div className="sticky top-0 bg-white dark:bg-slate-900 border-b border-slate-100 dark:border-slate-800 px-6 md:px-8 py-5 md:py-6 rounded-t-3xl z-10 flex-shrink-0">
      <div className="flex items-start justify-between gap-4">
        <div className="flex items-center gap-4 flex-1 min-w-0">
          <div className="w-14 h-14 bg-primary-50 dark:bg-primary-950 rounded-2xl flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold text-base shadow-inner border border-primary-100 dark:border-primary-800/50 uppercase flex-shrink-0">
            {balance.employee_name.split(' ').map((n: any) => n[0]).join('').slice(0, 2)}
          </div>
          <div className="flex-1 min-w-0">
            <Typography className="text-xl md:text-2xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase leading-tight truncate" variant="h2">
              {balance.employee_name}
            </Typography>
            <div className="flex items-center gap-2 mt-2">
              <span className="inline-block px-3 py-1.5 bg-slate-50 dark:bg-slate-800 text-slate-600 dark:text-slate-300 rounded-xl text-xs font-bold uppercase tracking-wider border border-slate-200 dark:border-slate-700 shadow-sm">
                {balance.leave_type_name}
              </span>
            </div>
          </div>
        </div>
        <Button
          aria-label={t("common.close")}
          className="h-11 w-11 p-2.5 text-slate-500 dark:text-slate-400 hover:text-slate-600 dark:hover:text-slate-300"
          onClick={onClose}
          size="icon"
          variant="ghost"
        >
          <X size={24} />
        </Button>
      </div>
    </div>
  );
}
