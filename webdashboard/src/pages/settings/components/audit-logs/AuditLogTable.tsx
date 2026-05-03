import React from "react";
import { useTranslation } from "react-i18next";
import { Globe, Monitor, User } from "lucide-react";
import type { AuditLog } from "@/lib/api";

interface AuditLogTableProps {
    logs: AuditLog[] | undefined;
    isLoading: boolean;
    isError: boolean;
    language: string;
}

const formatDateTime = (dateString: string, lng = 'en-US') => {
    return new Intl.DateTimeFormat(lng === 'vi' ? 'vi-VN' : lng === 'zh' ? 'zh-CN' : 'en-US', {
        dateStyle: 'medium',
        timeStyle: 'medium',
    }).format(new Date(dateString));
};

const getBadgeColor = (action: string) => {
    if (action.includes("remove") || action.includes("delete") || action.includes("revoke"))
        return "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400 border-red-200 dark:border-red-800";
    if (action.includes("add") || action.includes("create") || action.includes("assign"))
        return "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 border-green-200 dark:border-green-800";
    if (action.includes("update") || action.includes("edit"))
        return "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400 border-blue-200 dark:border-blue-800";
    return "bg-slate-100 text-slate-800 dark:bg-slate-800 dark:text-slate-300 border-slate-200 dark:border-slate-700";
};

const formatActionName = (action: string) => {
    return action.split("_").map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(" ");
};

export function AuditLogTable({ logs, isLoading, isError, language }: AuditLogTableProps) {
    const { t } = useTranslation();

    if (isLoading) {
        return (
            <div className="overflow-x-auto">
                <table className="w-full text-sm text-left">
                    <thead className="text-xs text-slate-500 dark:text-slate-400 bg-slate-50 dark:bg-slate-800/50 uppercase">
                        <tr>
                            <th className="px-6 py-3 font-medium">{t("audit_logs.table.timestamp")}</th>
                            <th className="px-6 py-3 font-medium">{t("audit_logs.table.action")}</th>
                            <th className="px-6 py-3 font-medium">{t("audit_logs.table.actor_target")}</th>
                            <th className="px-6 py-3 font-medium">{t("audit_logs.table.details")}</th>
                            <th className="px-6 py-3 font-medium">{t("audit_logs.table.context")}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-slate-200 dark:divide-slate-800">
                        {Array.from({ length: 5 }).map((_, index) => (
                            <tr className="animate-pulse bg-slate-50 dark:bg-slate-800/30" key={index}>
                                <td className="px-6 py-4"><div className="h-4 bg-slate-200 dark:bg-slate-700 rounded w-24"></div></td>
                                <td className="px-6 py-4"><div className="h-4 bg-slate-200 dark:bg-slate-700 rounded w-20"></div></td>
                                <td className="px-6 py-4"><div className="h-4 bg-slate-200 dark:bg-slate-700 rounded w-32"></div></td>
                                <td className="px-6 py-4"><div className="h-4 bg-slate-200 dark:bg-slate-700 rounded w-48"></div></td>
                                <td className="px-6 py-4"><div className="h-4 bg-slate-200 dark:bg-slate-700 rounded w-16"></div></td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        );
    }

    if (isError) {
        return (
            <div className="px-6 py-8 text-center text-red-500 dark:text-red-400">
                {t("audit_logs.messages.error_loading")}
            </div>
        );
    }

    if (!logs || logs.length === 0) {
        return (
            <div className="px-6 py-12 text-center text-slate-500 dark:text-slate-400">
                {t("audit_logs.messages.no_logs")}
            </div>
        );
    }

    return (
        <div className="overflow-x-auto">
            <table className="w-full text-sm text-left">
                <thead className="text-xs text-slate-500 dark:text-slate-400 bg-slate-50 dark:bg-slate-800/50 uppercase">
                    <tr>
                        <th className="px-6 py-3 font-medium">{t("audit_logs.table.timestamp")}</th>
                        <th className="px-6 py-3 font-medium">{t("audit_logs.table.action")}</th>
                        <th className="px-6 py-3 font-medium">{t("audit_logs.table.actor_target")}</th>
                        <th className="px-6 py-3 font-medium">{t("audit_logs.table.details")}</th>
                        <th className="px-6 py-3 font-medium">{t("audit_logs.table.context")}</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-slate-200 dark:divide-slate-800">
                    {logs.map((log) => (
                        <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors" key={log.id}>
                            <td className="px-6 py-4 whitespace-nowrap text-slate-500 dark:text-slate-400">
                                {formatDateTime(log.created_at, language)}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap">
                                <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${getBadgeColor(log.action)}`}>
                                    {formatActionName(log.action)}
                                </span>
                            </td>
                            <td className="px-6 py-4">
                                <div className="space-y-1">
                                    <div className="flex items-center gap-1.5 text-slate-900 dark:text-slate-200 font-medium">
                                        <User className="h-3.5 w-3.5 text-slate-400" />
                                        <span title={log.actor_id}>
                                            {log.actor_id ? log.actor_id.substring(0, 8) + '...' : t("audit_logs.messages.system")}
                                        </span>
                                    </div>
                                    {log.target_user_id ? (
                                        <div className="text-xs text-slate-500 dark:text-slate-400 flex items-center gap-1.5">
                                            {t("audit_logs.messages.target")} <span className="font-mono bg-slate-100 dark:bg-slate-800 px-1 rounded" title={log.target_user_id}>{log.target_user_id.substring(0, 8)}...</span>
                                        </div>
                                      ) : null}
                                </div>
                            </td>
                            <td className="px-6 py-4 max-w-[300px]">
                                {log.details ? (
                                    <pre className="text-xs bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 p-2 rounded-md overflow-x-auto whitespace-pre-wrap font-mono border border-slate-200 dark:border-slate-700">
                                        {JSON.stringify(log.details, null, 2)}
                                    </pre>
                                ) : (
                                    <span className="text-xs text-slate-400 dark:text-slate-500 italic">{t("audit_logs.messages.no_details")}</span>
                                )}
                            </td>
                            <td className="px-6 py-4 whitespace-nowrap">
                                <div className="space-y-1.5 text-xs text-slate-500 dark:text-slate-400">
                                    {log.ip_address ? (
                                        <div className="flex items-center gap-1.5" title="IP Address">
                                            <Globe className="h-3.5 w-3.5" />
                                            <span className="truncate w-24">{log.ip_address}</span>
                                        </div>
                                      ) : null}
                                    {log.user_agent ? (
                                        <div className="flex items-center gap-1.5" title={log.user_agent}>
                                            <Monitor className="h-3.5 w-3.5" />
                                            <span className="truncate w-24">{log.user_agent}</span>
                                        </div>
                                      ) : null}
                                </div>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
