import React from "react";
import { SearchX, Clock } from "lucide-react";
import { useTranslation } from "react-i18next";
import InfiniteScrollSentinel from "@/components/common/InfiniteScrollSentinel";
import { RawAttendanceLog } from "@/lib/api";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";

interface RawLogsTableProps {
    rawLogsData: RawAttendanceLog[];
    rawLogsLoading: boolean;
    hasNextPage: boolean;
    isFetchingNextPage: boolean;
    fetchNextPage: () => void;
}

export default function RawLogsTable({
    rawLogsData,
    rawLogsLoading,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
}: RawLogsTableProps) {
    const { t } = useTranslation();
    const { formatDate, formatTime } = useDateTimeFormatter();
    
    const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
        hasNextPage,
        isFetchingNextPage,
        fetchNextPage,
    });

    return (
        <div className="overflow-auto flex-1 min-h-0 relative">
            <table className="w-full text-left border-collapse">
                <thead>
                    <tr className="bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm border-b border-slate-100 dark:border-slate-800 sticky top-0 z-10">
                        <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.time")}</th>
                        <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.source")}</th>
                        <th className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.status")}</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
                    {rawLogsLoading ? (
                        Array.from({ length: 5 }).map((_, i) => (
                            <tr key={`skeleton-log-${i}`} className="border-b border-slate-50 dark:border-slate-800/50">
                                <td className="px-8 py-5">
                                    <div className="flex items-center gap-4">
                                        <div className="w-10 h-10 rounded-xl bg-slate-100 dark:bg-slate-800 animate-pulse"></div>
                                        <div className="space-y-1">
                                            <div className="h-4 w-20 bg-slate-200 dark:bg-slate-700 animate-pulse rounded"></div>
                                            <div className="h-3 w-16 bg-slate-100 dark:bg-slate-800 animate-pulse rounded"></div>
                                        </div>
                                    </div>
                                </td>
                                <td className="px-8 py-5">
                                    <div className="h-4 w-24 bg-slate-200 dark:bg-slate-700 animate-pulse rounded"></div>
                                </td>
                                <td className="px-8 py-5 text-right">
                                    <div className="flex justify-end">
                                        <div className="h-4 w-16 bg-emerald-100 dark:bg-emerald-900/20 animate-pulse rounded-full"></div>
                                    </div>
                                </td>
                            </tr>
                        ))
                    ) : rawLogsData.length === 0 ? (
                        <tr>
                            <td colSpan={3} className="px-8 py-20">
                                <div className="flex flex-col items-center justify-center text-center animate-in fade-in zoom-in-95 duration-500 max-w-sm mx-auto">
                                    <div className="w-16 h-16 mb-4 rounded-3xl bg-slate-50 dark:bg-slate-800/80 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner border border-slate-100 dark:border-slate-700/50 rotate-3">
                                        <SearchX size={24} className="-rotate-3" />
                                    </div>
                                    <h3 className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest mb-1">
                                        {t("attendance_logs.messages.no_data")}
                                    </h3>
                                    <p className="text-[10px] font-medium text-slate-500 dark:text-slate-400 max-w-[200px]">
                                        No punches were recorded for this day.
                                    </p>
                                </div>
                            </td>
                        </tr>
                    ) : (
                        rawLogsData.map((log: RawAttendanceLog) => (
                            <tr key={log.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group">
                                <td className="px-8 py-5">
                                    <div className="flex items-center gap-4">
                                        <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800/50"><Clock size={18} /></div>
                                        <div>
                                            <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">{formatDate(log.punch_time)}</div>
                                            <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 font-mono uppercase tracking-tighter">{formatTime(log.punch_time)}</div>
                                        </div>
                                    </div>
                                </td>
                                <td className="px-8 py-5">
                                    <div className="flex flex-col gap-1">
                                        <div className="flex items-center gap-2">
                                            <span className="px-2.5 py-1 bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-slate-200 dark:border-slate-700">{log.source}</span>
                                            {log.device_name && (
                                                <span className="text-[10px] font-bold text-slate-700 dark:text-slate-200 uppercase">{log.device_name}</span>
                                            )}
                                        </div>
                                    </div>
                                </td>
                                <td className="px-8 py-5 text-right">
                                    <span className="px-3 py-1 bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 rounded-full text-[9px] font-bold uppercase tracking-widest border border-emerald-100 dark:border-emerald-800 shadow-sm">{t("attendance_logs.status.recorded")}</span>
                                </td>
                            </tr>
                        ))
                    )}
                    {hasNextPage && (
                        <InfiniteScrollSentinel sentinelRef={loadMoreRef} isFetchingNextPage={isFetchingNextPage} colSpan={3} />
                    )}
                </tbody>
            </table>
        </div>
    );
}
