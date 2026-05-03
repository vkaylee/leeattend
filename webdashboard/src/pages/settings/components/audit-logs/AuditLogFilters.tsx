import React from "react";
import { useTranslation } from "react-i18next";
import { Activity } from "lucide-react";
import { Combobox } from "@/components/ui/combobox";

interface AuditLogFiltersProps {
    actionFilter: string;
    onActionChange: (value: string) => void;
    startDateFilter: string;
    onStartDateChange: (value: string) => void;
    endDateFilter: string;
    onEndDateChange: (value: string) => void;
}

export function AuditLogFilters({
    actionFilter,
    onActionChange,
    startDateFilter,
    onStartDateChange,
    endDateFilter,
    onEndDateChange,
}: AuditLogFiltersProps) {
    const { t } = useTranslation();

    const options = [
        { value: "user_login", label: `[${t("audit_logs.filters.groups.user_auth")}] ${t("auth.login.title")}` },
        { value: "user_create", label: `[${t("audit_logs.filters.groups.user_auth")}] ${t("common.create")} ${t("common.user")}` },
        { value: "user_update", label: `[${t("audit_logs.filters.groups.user_auth")}] ${t("common.update")} ${t("common.user")}` },
        { value: "user_delete", label: `[${t("audit_logs.filters.groups.user_auth")}] ${t("common.delete")} ${t("common.user")}` },
        { value: "role_assign", label: `[${t("audit_logs.filters.groups.roles_perms")}] ${t("workspace.roles.title")} ${t("common.assign")}` },
        { value: "role_revoke", label: `[${t("audit_logs.filters.groups.roles_perms")}] ${t("workspace.roles.title")} ${t("common.revoke")}` },
        { value: "permission_override_set", label: `[${t("audit_logs.filters.groups.roles_perms")}] ${t("common.permissions")} ${t("common.set")}` },
        { value: "leave_request_create", label: `[${t("audit_logs.filters.groups.domain")}] ${t("leave.request.create")}` },
        { value: "leave_request_approve", label: `[${t("audit_logs.filters.groups.domain")}] ${t("leave.request.approve")}` },
        { value: "attendance_log_create", label: `[${t("audit_logs.filters.groups.domain")}] ${t("attendance.log.create")}` },
        { value: "shift_swap_request", label: `[${t("audit_logs.filters.groups.domain")}] ${t("attendance.shift_swap.request")}` },
    ];

    return (
        <div className="border-b border-slate-200 dark:border-slate-800 px-6 py-4 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div className="flex items-center gap-2">
                <Activity className="h-5 w-5 text-slate-400" />
                <h3 className="font-medium text-slate-900 dark:text-white">{t("audit_logs.section_title")}</h3>
            </div>

            <div className="flex flex-wrap items-center gap-3">
                <Combobox
                    className="w-[280px] text-sm bg-slate-50 dark:bg-slate-800 border-slate-200 dark:border-slate-700 rounded-lg text-slate-700 dark:text-slate-200 py-[5px]"
                    onChange={onActionChange}
                    options={options}
                    placeholder={t("audit_logs.filters.all_actions")}
                    value={actionFilter}
                />
                <div className="flex items-center gap-2">
                    <input
                        aria-label={t("audit_logs.filters.start_date")}
                        className="text-sm bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500"
                        onChange={(e) => onStartDateChange(e.target.value)}
                        type="date"
                        value={startDateFilter}
                    />
                    <span className="text-slate-400">-</span>
                    <input
                        aria-label={t("audit_logs.filters.end_date")}
                        className="text-sm bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500"
                        onChange={(e) => onEndDateChange(e.target.value)}
                        type="date"
                        value={endDateFilter}
                    />
                </div>
            </div>
        </div>
    );
}
