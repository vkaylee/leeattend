import React, { useState } from "react";
import { useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { fetchAuditLogs, AuditLog } from "@/lib/api";
import { useTranslation } from "react-i18next";
import { ChevronLeft, ChevronRight, Activity, Globe, Monitor, User } from "lucide-react";

const formatDateTime = (dateStr: string) => {
    return new Intl.DateTimeFormat('en-US', {
        dateStyle: 'medium',
        timeStyle: 'medium',
    }).format(new Date(dateStr));
};

export default function AuditLogPage() {
    const { workspaceId } = useParams<{ workspaceId: string }>();
    const [page, setPage] = useState(1);
    const limit = 20;

    const [actionFilter, setActionFilter] = useState("");
    const [startDateFilter, setStartDateFilter] = useState("");
    const [endDateFilter, setEndDateFilter] = useState("");

    const { data, isLoading, isError } = useQuery({
        queryKey: ["audit_logs", workspaceId, page, actionFilter, startDateFilter, endDateFilter],
        queryFn: () => fetchAuditLogs(workspaceId!, {
            page,
            limit,
            action: actionFilter || undefined,
            startDate: startDateFilter ? new Date(startDateFilter).toISOString() : undefined,
            endDate: endDateFilter ? new Date(endDateFilter).toISOString() : undefined,
        }),
        enabled: !!workspaceId,
    });

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

    return (
        <div className="space-y-6 max-w-7xl mx-auto">
            <div>
                <h2 className="text-3xl font-bold tracking-tight text-slate-900 dark:text-white">Audit Logs</h2>
                <p className="text-slate-500 dark:text-slate-400 mt-1">
                    Track security and administrative events across your workspace.
                </p>
            </div>

            <div className="bg-white dark:bg-slate-900 rounded-xl border border-slate-200 dark:border-slate-800 overflow-hidden shadow-sm">
                <div className="border-b border-slate-200 dark:border-slate-800 px-6 py-4 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                    <div className="flex items-center gap-2">
                        <Activity className="h-5 w-5 text-slate-400" />
                        <h3 className="font-medium text-slate-900 dark:text-white">Security Events</h3>
                    </div>

                    <div className="flex flex-wrap items-center gap-3">
                        <select
                            value={actionFilter}
                            onChange={(e) => { setActionFilter(e.target.value); setPage(1); }}
                            className="text-sm bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500"
                        >
                            <option value="">All Actions</option>
                            <optgroup label="User & Auth">
                                <option value="user_login">User Login</option>
                                <option value="user_create">User Create</option>
                                <option value="user_update">User Update</option>
                                <option value="user_delete">User Delete</option>
                            </optgroup>
                            <optgroup label="Roles & Permissions">
                                <option value="role_assign">Role Assign</option>
                                <option value="role_revoke">Role Revoke</option>
                                <option value="permission_override_set">Permission Set</option>
                            </optgroup>
                            <optgroup label="Domain">
                                <option value="leave_request_create">Leave Request Create</option>
                                <option value="leave_request_approve">Leave Request Approve</option>
                                <option value="attendance_log_create">Attendance Log Create</option>
                                <option value="shift_swap_request">Shift Swap Request</option>
                            </optgroup>
                        </select>
                        <div className="flex items-center gap-2">
                            <input
                                type="date"
                                value={startDateFilter}
                                onChange={(e) => { setStartDateFilter(e.target.value); setPage(1); }}
                                className="text-sm bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500"
                            />
                            <span className="text-slate-400">-</span>
                            <input
                                type="date"
                                value={endDateFilter}
                                onChange={(e) => { setEndDateFilter(e.target.value); setPage(1); }}
                                className="text-sm bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500"
                            />
                        </div>
                    </div>
                </div>

                <div className="overflow-x-auto">
                    <table className="w-full text-sm text-left">
                        <thead className="text-xs text-slate-500 dark:text-slate-400 bg-slate-50 dark:bg-slate-800/50 uppercase">
                            <tr>
                                <th className="px-6 py-3 font-medium">Timestamp</th>
                                <th className="px-6 py-3 font-medium">Action</th>
                                <th className="px-6 py-3 font-medium">Actor / Target</th>
                                <th className="px-6 py-3 font-medium">Details</th>
                                <th className="px-6 py-3 font-medium">Context</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-200 dark:divide-slate-800">
                            {isLoading ? (
                                Array.from({ length: 5 }).map((_, i) => (
                                    <tr key={i} className="animate-pulse bg-slate-50 dark:bg-slate-800/30">
                                        <td className="px-6 py-4"><div className="h-4 bg-slate-200 dark:bg-slate-700 rounded w-24"></div></td>
                                        <td className="px-6 py-4"><div className="h-4 bg-slate-200 dark:bg-slate-700 rounded w-20"></div></td>
                                        <td className="px-6 py-4"><div className="h-4 bg-slate-200 dark:bg-slate-700 rounded w-32"></div></td>
                                        <td className="px-6 py-4"><div className="h-4 bg-slate-200 dark:bg-slate-700 rounded w-48"></div></td>
                                        <td className="px-6 py-4"><div className="h-4 bg-slate-200 dark:bg-slate-700 rounded w-16"></div></td>
                                    </tr>
                                ))
                            ) : isError ? (
                                <tr>
                                    <td colSpan={5} className="px-6 py-8 text-center text-red-500 dark:text-red-400">
                                        Error loading audit logs.
                                    </td>
                                </tr>
                            ) : !data || data.data.length === 0 ? (
                                <tr>
                                    <td colSpan={5} className="px-6 py-12 text-center text-slate-500 dark:text-slate-400">
                                        No audit logs found.
                                    </td>
                                </tr>
                            ) : (
                                data.data.map((log: AuditLog) => (
                                    <tr key={log.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors">
                                        <td className="px-6 py-4 whitespace-nowrap text-slate-500 dark:text-slate-400">
                                            {formatDateTime(log.created_at)}
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
                                                        {log.actor_id ? log.actor_id.substring(0, 8) + '...' : 'System'}
                                                    </span>
                                                </div>
                                                {log.target_user_id && (
                                                    <div className="text-xs text-slate-500 dark:text-slate-400 flex items-center gap-1.5">
                                                        Target: <span title={log.target_user_id} className="font-mono bg-slate-100 dark:bg-slate-800 px-1 rounded">{log.target_user_id.substring(0, 8)}...</span>
                                                    </div>
                                                )}
                                            </div>
                                        </td>
                                        <td className="px-6 py-4 max-w-[300px]">
                                            {log.details ? (
                                                <pre className="text-xs bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 p-2 rounded-md overflow-x-auto whitespace-pre-wrap font-mono border border-slate-200 dark:border-slate-700">
                                                    {JSON.stringify(log.details, null, 2)}
                                                </pre>
                                            ) : (
                                                <span className="text-xs text-slate-400 dark:text-slate-500 italic">No details</span>
                                            )}
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap">
                                            <div className="space-y-1.5 text-xs text-slate-500 dark:text-slate-400">
                                                {log.ip_address && (
                                                    <div className="flex items-center gap-1.5" title="IP Address">
                                                        <Globe className="h-3.5 w-3.5" />
                                                        <span className="truncate w-24">{log.ip_address}</span>
                                                    </div>
                                                )}
                                                {log.user_agent && (
                                                    <div className="flex items-center gap-1.5" title={log.user_agent}>
                                                        <Monitor className="h-3.5 w-3.5" />
                                                        <span className="truncate w-24">{log.user_agent}</span>
                                                    </div>
                                                )}
                                            </div>
                                        </td>
                                    </tr>
                                ))
                            )}
                        </tbody>
                    </table>
                </div>

                {data && data.total > 0 && (
                    <div className="border-t border-slate-200 dark:border-slate-800 px-6 py-4 flex items-center justify-between bg-slate-50/50 dark:bg-slate-800/10">
                        <div className="text-sm text-slate-500 dark:text-slate-400">
                            Showing <span className="font-medium text-slate-900 dark:text-white">{((page - 1) * limit) + 1}</span> to <span className="font-medium text-slate-900 dark:text-white">{Math.min(page * limit, data.total)}</span> of <span className="font-medium text-slate-900 dark:text-white">{data.total}</span> entries
                        </div>
                        <div className="flex items-center gap-2">
                            <button
                                onClick={() => setPage(p => Math.max(1, p - 1))}
                                disabled={page === 1 || isLoading}
                                className="flex items-center justify-center h-8 px-3 rounded-md border border-slate-200 dark:border-slate-700 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                            >
                                <ChevronLeft className="h-4 w-4 mr-1" />
                                Previous
                            </button>
                            <button
                                onClick={() => setPage(p => p + 1)}
                                disabled={page * limit >= data.total || isLoading}
                                className="flex items-center justify-center h-8 px-3 rounded-md border border-slate-200 dark:border-slate-700 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                            >
                                Next
                                <ChevronRight className="h-4 w-4 ml-1" />
                            </button>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
