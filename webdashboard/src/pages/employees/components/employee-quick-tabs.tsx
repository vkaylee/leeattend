import { UserCheck, Users, UserX } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";

interface EmployeeQuickTabsProps {
  activeCount?: number;
  filterStatus: string;
  setFilterStatus: (status: string) => void;
  totalCount?: number;
}

export function EmployeeQuickTabs({
  filterStatus,
  setFilterStatus,
  activeCount,
  totalCount,
}: EmployeeQuickTabsProps) {
  const { t } = useTranslation();

  return (
    <div className="w-full">
      <Tabs onValueChange={setFilterStatus} value={filterStatus}>
        <TabsList className="border-b border-slate-200 dark:border-slate-800 bg-transparent gap-1 p-0">
          <TabsTrigger
            className={cn(
              "flex items-center gap-2 py-2.5 px-4 text-xs font-bold uppercase tracking-wider rounded-t-xl transition-all",
              filterStatus === "active"
                ? "bg-white dark:bg-slate-900 border-b-2 border-primary-500 text-primary-600 dark:text-primary-400 shadow-xs"
                : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
            )}
            data-testid="quick-tab-active"
            value="active"
          >
            <UserCheck size={14} />
            <span>{t("employees.table.active", "Hoạt động")}</span>
            {typeof activeCount === "number" && activeCount > 0 && (
              <span className="ml-1 text-[10px] px-1.5 py-0.5 rounded-full bg-primary-50 dark:bg-primary-950/60 text-primary-600 dark:text-primary-400 font-mono">
                {activeCount}
              </span>
            )}
          </TabsTrigger>

          <TabsTrigger
            className={cn(
              "flex items-center gap-2 py-2.5 px-4 text-xs font-bold uppercase tracking-wider rounded-t-xl transition-all",
              filterStatus === "terminated"
                ? "bg-white dark:bg-slate-900 border-b-2 border-primary-500 text-primary-600 dark:text-primary-400 shadow-xs"
                : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
            )}
            data-testid="quick-tab-terminated"
            value="terminated"
          >
            <UserX size={14} />
            <span>{t("employees.table.terminated", "Đã nghỉ việc")}</span>
          </TabsTrigger>

          <TabsTrigger
            className={cn(
              "flex items-center gap-2 py-2.5 px-4 text-xs font-bold uppercase tracking-wider rounded-t-xl transition-all",
              filterStatus === "all"
                ? "bg-white dark:bg-slate-900 border-b-2 border-primary-500 text-primary-600 dark:text-primary-400 shadow-xs"
                : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
            )}
            data-testid="quick-tab-all"
            value="all"
          >
            <Users size={14} />
            <span>{t("common.all", "Tất cả")}</span>
            {typeof totalCount === "number" && totalCount > 0 && (
              <span className="ml-1 text-[10px] px-1.5 py-0.5 rounded-full bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 font-mono">
                {totalCount}
              </span>
            )}
          </TabsTrigger>
        </TabsList>
      </Tabs>
    </div>
  );
}
