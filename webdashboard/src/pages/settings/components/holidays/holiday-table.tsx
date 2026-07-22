import { CalendarDays, Pencil, Trash2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { Holiday, LeaveType } from "@/lib/api/types";

import Button from "@/components/ui/button";
import ResponsiveTable from "@/components/ui/responsive-table";
import {
    Table,
    TableHeader,
    TableBody,
    TableRow,
    TableHead,
    TableCell,
} from "@/components/ui/table";
import { Typography } from "@/components/ui/typography";

interface HolidayTableProps {
    holidays: Holiday[];
    isLoading: boolean;
    leaveTypes: LeaveType[];
    onDelete: (id: string) => Promise<void>;
    onEdit: (h: Holiday) => void;
    year: number;
}

function HolidayTableHeader() {
    const { t } = useTranslation();
    return (
        <TableHeader>
            <TableRow>
                <TableHead className="px-4 py-4">{t("holidays.table.date")}</TableHead>
                <TableHead className="px-4 py-4">{t("holidays.table.name")}</TableHead>
                <TableHead className="px-4 py-4">{t("holidays.table.type")}</TableHead>
                <TableHead className="px-4 py-4 text-center">{t("holidays.table.paid")}</TableHead>
                <TableHead className="px-4 py-4 text-center">{t("holidays.table.ot")}</TableHead>
                <TableHead className="px-4 py-4 text-center">{t("holidays.table.credit", "Credit")}</TableHead>
                <TableHead className="px-4 py-4 text-center">{t("holidays.table.threshold", "Ngưỡng")}</TableHead>
                <TableHead className="px-4 py-4">{t("holidays.table.description")}</TableHead>
                <TableHead className="px-4 py-4" />
            </TableRow>
        </TableHeader>
    );
}

function HolidayTableRow({ h, leaveTypes, onDelete, onEdit }: { h: Holiday; leaveTypes: LeaveType[]; onDelete: (id: string) => void; onEdit: (h: Holiday) => void }) {
    const { t } = useTranslation();
    return (
        <TableRow className="group" key={h.id}>
            <TableCell className="px-4 py-4 font-mono text-sm font-bold text-slate-700 dark:text-slate-200">
                {h.date}
            </TableCell>
            <TableCell className="px-4 py-4 font-bold text-slate-900 dark:text-white text-sm">
                {h.name}
            </TableCell>
            <TableCell className="px-4 py-4">
                <span className="px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wide bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 border border-slate-200 dark:border-slate-700">
                    {t(`holidays.fields.types.${h.holiday_type}`)}
                </span>
            </TableCell>
            <TableCell className="px-4 py-4 text-center">
                {h.is_paid ? (
                    <span className="text-emerald-600">✅ {t("holidays.table.paid_status")}</span>
                ) : (
                    <span className="text-amber-600 text-xs font-bold">
                        ⚡ {t("holidays.table.deduct_leave")}
                        {!!h.leave_type_id && leaveTypes.find((lt) => lt.id === h.leave_type_id) && (
                            <span className="block text-[10px] text-slate-400 font-normal">
                                → {leaveTypes.find((lt) => lt.id === h.leave_type_id)?.name}
                            </span>
                        )}
                    </span>
                )}
            </TableCell>
            <TableCell className="px-4 py-4 text-center font-mono font-bold text-sm text-orange-600 dark:text-orange-400">
                {h.ot_multiplier ? `${h.ot_multiplier}×` : "—"}
            </TableCell>
            <TableCell className="px-4 py-4 text-center text-sm">
                {h.is_credit_on_work ? (
                    <span className="text-emerald-600">✅</span>
                ) : (
                    <span className="text-slate-400">❌</span>
                )}
            </TableCell>
            <TableCell className="px-4 py-4 text-center font-mono text-sm text-slate-600 dark:text-slate-400">
                {h.work_threshold_percent != null ? `${h.work_threshold_percent}%` : "—"}
            </TableCell>
            <TableCell className="px-4 py-4 text-xs text-slate-500 dark:text-slate-400 max-w-[200px] truncate">
                {h.description ?? "—"}
            </TableCell>
            <TableCell className="px-4 py-4">
                <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <Button className="p-1.5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-500 hover:text-primary-600 hover:bg-primary-50 transition-colors"
                        onClick={() => { onEdit(h); }}
                        variant="ghost"
                    >
                        <Pencil size={14} />
                    </Button>
                    <Button className="p-1.5 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-500 hover:text-rose-600 hover:bg-rose-50 transition-colors"
                        onClick={() => { onDelete(h.id); }}
                        variant="ghost"
                    >
                        <Trash2 size={14} />
                    </Button>
                </div>
            </TableCell>
        </TableRow>
    );
}

function HolidayMobileCard({
    h,
    leaveTypes,
    onDelete,
    onEdit,
}: {
    h: Holiday;
    leaveTypes: LeaveType[];
    onDelete: (id: string) => void;
    onEdit: (h: Holiday) => void;
}) {
    const { t } = useTranslation();
    return (
        <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-sm transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900">
            {/* Header: Date and name */}
            <div className="flex items-start justify-between gap-3">
                <div className="min-w-0 flex-1">
                    <div className="font-mono text-sm font-bold text-slate-700 dark:text-slate-200">
                        {h.date}
                    </div>
                    <div className="mt-1 truncate text-sm font-bold text-slate-900 dark:text-white">
                        {h.name}
                    </div>
                </div>
                <span className="shrink-0 rounded-full border border-slate-200 bg-slate-100 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-slate-600 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-400">
                    {t(`holidays.fields.types.${h.holiday_type}`)}
                </span>
            </div>

            {/* Details grid */}
            <div className="mt-4 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
                <div>
                    <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">
                        {t("holidays.table.paid")}
                    </div>
                    <div className="mt-1 text-sm">
                        {h.is_paid ? (
                            <span className="text-emerald-600">✅ {t("holidays.table.paid_status")}</span>
                        ) : (
                            <div>
                                <span className="text-xs font-bold text-amber-600">
                                    ⚡ {t("holidays.table.deduct_leave")}
                                </span>
                                {!!h.leave_type_id && leaveTypes.find((lt) => lt.id === h.leave_type_id) && (
                                    <div className="mt-0.5 text-[10px] text-slate-400">
                                        → {leaveTypes.find((lt) => lt.id === h.leave_type_id)?.name}
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                </div>
                <div>
                    <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">
                        {t("holidays.table.ot")}
                    </div>
                    <div className="mt-1 font-mono text-sm font-bold text-orange-600 dark:text-orange-400">
                        {h.ot_multiplier ? `${h.ot_multiplier}×` : "—"}
                    </div>
                </div>
                <div>
                    <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">
                        {t("holidays.table.credit", "Credit")}
                    </div>
                    <div className="mt-1 text-sm">
                        {h.is_credit_on_work ? (
                            <span className="text-emerald-600">✅</span>
                        ) : (
                            <span className="text-slate-400">❌</span>
                        )}
                    </div>
                </div>
                <div>
                    <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">
                        {t("holidays.table.threshold", "Ngưỡng")}
                    </div>
                    <div className="mt-1 font-mono text-sm text-slate-600 dark:text-slate-400">
                        {h.work_threshold_percent != null ? `${h.work_threshold_percent}%` : "—"}
                    </div>
                </div>
            </div>

            {/* Description */}
            {!!h.description && (
                <div className="mt-4 border-t border-slate-100 pt-4 dark:border-slate-800">
                    <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">
                        {t("holidays.table.description")}
                    </div>
                    <div className="mt-1 text-xs text-slate-500 dark:text-slate-400">
                        {h.description}
                    </div>
                </div>
              )}

            {/* Actions */}
            <div className="mt-4 flex justify-end gap-2 border-t border-slate-100 pt-4 dark:border-slate-800">
                <Button
                    className="rounded-lg border border-slate-200 bg-white p-1.5 text-slate-500 transition-colors hover:bg-primary-50 hover:text-primary-600 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-primary-950/30 dark:hover:text-primary-400"
                    onClick={() => { onEdit(h); }}
                    variant="ghost"
                >
                    <Pencil size={14} />
                </Button>
                <Button
                    className="rounded-lg border border-slate-200 bg-white p-1.5 text-slate-500 transition-colors hover:bg-rose-50 hover:text-rose-600 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-rose-950/30 dark:hover:text-rose-400"
                    onClick={() => { onDelete(h.id); }}
                    variant="ghost"
                >
                    <Trash2 size={14} />
                </Button>
            </div>
        </div>
    );
}

export function HolidayTable({ holidays, isLoading, leaveTypes, onDelete, onEdit, year }: HolidayTableProps) {
    const { t } = useTranslation();

    if (isLoading) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] border border-slate-200 dark:border-slate-800 overflow-hidden shadow-sm">
                <div className="p-12 text-center text-slate-400 text-sm font-bold uppercase tracking-widest animate-pulse">
                    {t("common.loading")}
                </div>
            </div>
        );
    }

    if (holidays.length === 0) {
        return (
            <div className="bg-white dark:bg-slate-900 rounded-3xl sm:rounded-[3rem] border border-slate-200 dark:border-slate-800 overflow-hidden shadow-sm">
                <div className="p-12 text-center">
                    <CalendarDays className="w-12 h-12 text-slate-300 dark:text-slate-600 mx-auto mb-3" />
                    <Typography className="text-slate-400 text-sm font-medium" variant="p">
                        {t("holidays.no_holidays", { year })}
                    </Typography>
                </div>
            </div>
        );
    }

    return (
        <ResponsiveTable
            data={holidays}
            getCardKey={(h) => h.id}
            renderCard={(h) => (
                <HolidayMobileCard
                    h={h}
                    leaveTypes={leaveTypes}
                    onDelete={onDelete}
                    onEdit={onEdit}
                />
            )}
            renderDesktopTable={() => (
                <Table>
                    <HolidayTableHeader />
                    <TableBody>
                        {holidays.map((h) => (
                            <HolidayTableRow h={h} key={h.id} leaveTypes={leaveTypes} onDelete={onDelete} onEdit={onEdit} />
                        ))}
                    </TableBody>
                </Table>
            )}
        />
    );
}
