import { ChevronLeft, ChevronRight } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";

interface AuditLogPaginationProps {
    isLoading: boolean;
    limit: number;
    onPageChange: (newPage: number) => void;
    page: number;
    total: number;
}

export function AuditLogPagination({
    page,
    limit,
    total,
    isLoading,
    onPageChange,
}: AuditLogPaginationProps) {
    const { t } = useTranslation();

    if (total === 0) return null;

    return (
        <div className="border-t border-slate-200 dark:border-slate-800 px-6 py-4 flex items-center justify-between bg-slate-50/50 dark:bg-slate-800/10">
            <div className="text-sm text-slate-500 dark:text-slate-400">
                {t("audit_logs.pagination.showing")} <span className="font-medium text-slate-900 dark:text-white">{((page - 1) * limit) + 1}</span> {t("audit_logs.pagination.to")} <span className="font-medium text-slate-900 dark:text-white">{Math.min(page * limit, total)}</span> {t("audit_logs.pagination.of")} <span className="font-medium text-slate-900 dark:text-white">{total}</span> {t("audit_logs.pagination.entries")}
            </div>
            <div className="flex items-center gap-2">
                <Button
                    className="flex items-center justify-center h-8 px-3 rounded-md border border-slate-200 dark:border-slate-700 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    disabled={page === 1 || isLoading}
                    onClick={() => { onPageChange(Math.max(1, page - 1)); }}
                >
                    <ChevronLeft className="h-4 w-4 mr-1" />
                    {t("audit_logs.pagination.previous")}
                </Button>
                <Button
                    className="flex items-center justify-center h-8 px-3 rounded-md border border-slate-200 dark:border-slate-700 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    disabled={page * limit >= total || isLoading}
                    onClick={() => { onPageChange(page + 1); }}
                >
                    {t("audit_logs.pagination.next")}
                    <ChevronRight className="h-4 w-4 ml-1" />
                </Button>
            </div>
        </div>
    );
}
