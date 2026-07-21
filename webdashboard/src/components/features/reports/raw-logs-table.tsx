import { SearchX, Clock } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import ResponsiveTable from "@/components/ui/responsive-table";
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from "@/components/ui/table";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { RawAttendanceLog } from "@/lib/api";

interface RawLogsTableProps {
    fetchNextPage: () => void;
    hasNextPage: boolean;
    isFetchingNextPage: boolean;
    rawLogsData: RawAttendanceLog[];
    rawLogsLoading: boolean;
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
    const mobileLoadMoreRef = useInfiniteScroll<HTMLDivElement>({
        hasNextPage,
        isFetchingNextPage,
        fetchNextPage,
    });

    return (
        <ResponsiveTable<RawAttendanceLog>
            data={rawLogsData}
            emptyState={{
                description: "No punches were recorded for this day.",
                icon: <SearchX className="h-10 w-10" />,
                title: t("attendance_logs.messages.no_data"),
            }}
            getCardKey={(log) => log.id}
            isLoading={rawLogsLoading}
            mobileFooter={(
                <div className="flex min-h-10 items-center justify-center" ref={mobileLoadMoreRef}>
                    {isFetchingNextPage ? <span className="text-[10px] font-bold uppercase tracking-widest text-slate-400">{t("common.loading")}</span> : null}
                </div>
            )}
            renderCard={(log) => (
                <article className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
                    <div className="flex items-start gap-3">
                        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-primary-100 bg-primary-50 text-primary-600 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-400">
                            <Clock size={18} />
                        </div>
                        <div className="min-w-0 flex-1">
                            <div className="text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100">{formatDate(log.punch_time)}</div>
                            <div className="mt-1 font-mono text-xs font-bold text-slate-500 dark:text-slate-400">{formatTime(log.punch_time)}</div>
                        </div>
                        <span className="shrink-0 rounded-full border border-emerald-100 bg-emerald-50 px-3 py-1 text-[9px] font-bold uppercase tracking-widest text-emerald-600 shadow-sm dark:border-emerald-800 dark:bg-emerald-950 dark:text-emerald-300">{t("attendance_logs.status.recorded")}</span>
                    </div>
                    <div className="mt-4 flex flex-wrap items-center gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
                        <span className="rounded-lg border border-slate-200 bg-slate-100 px-2.5 py-1 text-[9px] font-bold uppercase tracking-widest text-slate-600 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300">{log.source}</span>
                        {log.device_name ? <span className="truncate text-[10px] font-bold uppercase text-slate-500 dark:text-slate-400">{log.device_name}</span> : null}
                    </div>
                </article>
            )}
            renderDesktopTable={() => (
                <div className="flex-1 min-h-0 relative overflow-auto">
                    <Table>
                        <TableHeader className="bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm border-b border-slate-100 dark:border-slate-800 sticky top-0 z-10">
                            <TableRow>
                                <TableHead className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.time")}</TableHead>
                                <TableHead className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("common.source")}</TableHead>
                                <TableHead className="px-8 py-4 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("common.status")}</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {rawLogsLoading ? (
                                Array.from({ length: 5 }).map((_, index) => (
                                    <TableRow className="border-b border-slate-50 dark:border-slate-800/50" key={`skeleton-log-${index}`}>
                                        <TableCell className="px-8 py-5">
                                            <div className="flex items-center gap-4">
                                                <div className="w-10 h-10 rounded-xl bg-slate-100 dark:bg-slate-800 animate-pulse"></div>
                                                <div className="space-y-1">
                                                    <div className="h-4 w-20 bg-slate-200 dark:bg-slate-700 animate-pulse rounded"></div>
                                                    <div className="h-3 w-16 bg-slate-100 dark:bg-slate-800 animate-pulse rounded"></div>
                                                </div>
                                            </div>
                                        </TableCell>
                                        <TableCell className="px-8 py-5">
                                            <div className="h-4 w-24 bg-slate-200 dark:bg-slate-700 animate-pulse rounded"></div>
                                        </TableCell>
                                        <TableCell className="px-8 py-5 text-right">
                                            <div className="flex justify-end">
                                                <div className="h-4 w-16 bg-emerald-100 dark:bg-emerald-900/20 animate-pulse rounded-full"></div>
                                            </div>
                                        </TableCell>
                                    </TableRow>
                                ))
                            ) : rawLogsData.length === 0 ? (
                                <TableRow>
                                    <TableCell className="px-8 py-20" colSpan={3}>
                                        <div className="flex flex-col items-center justify-center text-center animate-in fade-in zoom-in-95 duration-500 max-w-sm mx-auto">
                                            <div className="w-16 h-16 mb-4 rounded-3xl bg-slate-50 dark:bg-slate-800/80 flex items-center justify-center text-slate-300 dark:text-slate-600 shadow-inner border border-slate-100 dark:border-slate-700/50 rotate-3">
                                                <SearchX className="-rotate-3" size={24} />
                                            </div>
                                            <Typography className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-widest mb-1" variant="h3">
                                                {t("attendance_logs.messages.no_data")}
                                            </Typography>
                                            <Typography className="text-[10px] font-medium text-slate-500 dark:text-slate-400 max-w-[200px]" variant="p">
                                                No punches were recorded for this day.
                                            </Typography>
                                        </div>
                                    </TableCell>
                                </TableRow>
                            ) : (
                                rawLogsData.map((log: RawAttendanceLog) => (
                                    <TableRow className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors group" key={log.id}>
                                        <TableCell className="px-8 py-5">
                                            <div className="flex items-center gap-4">
                                                <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800/50"><Clock size={18} /></div>
                                                <div>
                                                    <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase">{formatDate(log.punch_time)}</div>
                                                    <div className="text-[10px] font-bold text-slate-500 dark:text-slate-400 font-mono uppercase tracking-tighter">{formatTime(log.punch_time)}</div>
                                                </div>
                                            </div>
                                        </TableCell>
                                        <TableCell className="px-8 py-5">
                                            <div className="flex flex-col gap-1">
                                                <div className="flex items-center gap-2">
                                                    <span className="px-2.5 py-1 bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-slate-200 dark:border-slate-700">{log.source}</span>
                                                    {log.device_name ? <span className="text-[10px] font-bold text-slate-700 dark:text-slate-200 uppercase">{log.device_name}</span> : null}
                                                </div>
                                            </div>
                                        </TableCell>
                                        <TableCell className="px-8 py-5 text-right">
                                            <span className="px-3 py-1 bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 rounded-full text-[9px] font-bold uppercase tracking-widest border border-emerald-100 dark:border-emerald-800 shadow-sm">{t("attendance_logs.status.recorded")}</span>
                                        </TableCell>
                                    </TableRow>
                                ))
                            )}
                            {hasNextPage ? <InfiniteScrollSentinel colSpan={3} isFetchingNextPage={isFetchingNextPage} sentinelRef={loadMoreRef} /> : null}
                        </TableBody>
                    </Table>
                </div>
            )}
        />
    );
}

