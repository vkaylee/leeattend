import { ArrowRight, CalendarDays, Check, X } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

import StatusBadge from "@/components/ui/status-badge";
import { Typography } from "@/components/ui/typography";

import { ShiftSwapRequest } from "../../lib/api/shift-swaps";

interface ShiftSwapListItemProps {
  formatDate: (d: string) => string;
  onApprove: (id: string) => void;
  onReject: (id: string) => void;
  swap: ShiftSwapRequest;
  t: TFunction;
}

export function ShiftSwapListItem({
  formatDate,
  onApprove,
  onReject,
  swap,
  t,
}: ShiftSwapListItemProps) {
  return (
    <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group">
      {/* Date */}
      <td className="px-10 py-6">
        <div className="inline-flex items-center gap-2 bg-slate-50 dark:bg-slate-800 px-4 py-2 rounded-xl border border-slate-100 dark:border-slate-800 shadow-inner">
          <CalendarDays
            className="text-primary-600 dark:text-primary-400"
            size={14}
          />
          <span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-tighter">
            {formatDate(swap.swap_date)}
          </span>
        </div>
      </td>

      {/* Requester */}
      <td className="px-8 py-6">
        <div>
          <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1">
            {swap.requester_name ?? "—"}
          </div>
          <div className="text-[10px] text-primary-600 dark:text-primary-400 font-bold">
            {swap.requester_shift_name ?? t("shift_swaps.day_off")}
          </div>
        </div>
      </td>

      {/* Arrow */}
      <td className="px-4 py-6 text-center">
        <ArrowRight
          className="text-slate-300 dark:text-slate-600 mx-auto"
          size={16}
        />
      </td>

      {/* Target */}
      <td className="px-8 py-6">
        <div>
          <div className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight uppercase leading-none mb-1">
            {swap.target_name ?? "—"}
          </div>
          <div className="text-[10px] text-primary-600 dark:text-primary-400 font-bold">
            {swap.target_shift_name ?? t("shift_swaps.day_off")}
          </div>
        </div>
      </td>

      {/* Reason */}
      <td className="px-8 py-6">
        <Typography className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-tight line-clamp-2 italic opacity-70 leading-relaxed" variant="p">
          {swap.reason ?? t("common.no_notes")}
        </Typography>
      </td>

      {/* Status */}
      <td className="px-8 py-6 text-center">
        <StatusBadge
          variant={
            swap.status === "pending_admin"
              ? "warning"
              : swap.status === "approved"
                ? "success"
                : "danger"
          }
        >
          {t(`shift_swaps.status.${swap.status}`)}
        </StatusBadge>
      </td>

      {/* Actions */}
      <td className="px-10 py-6 text-right">
        {swap.status === "pending_admin" && (
          <div className="flex items-center justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300">
            <button
              className="p-3 bg-emerald-600 text-white hover:bg-emerald-700 rounded-xl shadow-lg shadow-emerald-100 transition-all active:scale-90"
              onClick={() => {
                onApprove(swap.id);
              }}
              title={t("common.approve")}
              type="button"
            >
              <Check size={18} />
            </button>
            <button
              className="p-3 bg-white dark:bg-slate-900 text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950 rounded-xl shadow-md border border-rose-100 dark:border-rose-800 transition-all active:scale-90"
              onClick={() => {
                onReject(swap.id);
              }}
              title={t("common.reject")}
              type="button"
            >
              <X size={18} />
            </button>
          </div>
        )}
      </td>
    </tr>
  );
}
