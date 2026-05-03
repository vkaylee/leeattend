import React from "react";
import { useTranslation } from "react-i18next";

import { AuditLogFilters } from "./components/audit-logs/audit-log-filters";
import { AuditLogHeader } from "./components/audit-logs/audit-log-header";
import { AuditLogPagination } from "./components/audit-logs/audit-log-pagination";
import { AuditLogTable } from "./components/audit-logs/audit-log-table";
import { useAuditLogs } from "./hooks/use-audit-logs";

export default function AuditLogPage() {
    const { i18n } = useTranslation();
    const {
        data,
        isLoading,
        isError,
        page,
        limit,
        actionFilter,
        startDateFilter,
        endDateFilter,
        handleActionFilterChange,
        handleStartDateChange,
        handleEndDateChange,
        handlePageChange,
    } = useAuditLogs();

    return (
        <div className="space-y-6 max-w-7xl mx-auto">
            <AuditLogHeader />

            <div className="bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-800 overflow-hidden shadow-sm">
                <AuditLogFilters
                    actionFilter={actionFilter}
                    endDateFilter={endDateFilter}
                    onActionChange={handleActionFilterChange}
                    onEndDateChange={handleEndDateChange}
                    onStartDateChange={handleStartDateChange}
                    startDateFilter={startDateFilter}
                />

                <AuditLogTable
                    isError={isError}
                    isLoading={isLoading}
                    language={i18n.language}
                    logs={data?.data}
                />

                {!!data && (
                    <AuditLogPagination
                        isLoading={isLoading}
                        limit={limit}
                        onPageChange={handlePageChange}
                        page={page}
                        total={data.total}
                    />
                  )}
            </div>
        </div>
    );
}
