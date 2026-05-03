import type { TFunction } from "i18next";

import { CheckCircle2, XCircle, ShieldCheck, MessageSquare, Sun, Moon, Timer } from "lucide-react";
import React from "react";

import type { fetchWorkspaceLeaveRequests } from "@/lib/api";
import type { Employee, LeaveType } from "@/lib/api/types";

import { cn } from "@/lib/utils";

type LeaveRequestItem = Awaited<ReturnType<typeof fetchWorkspaceLeaveRequests>>[0];

export interface LeaveRequestRowProps {
  emp?: Employee;
  formatDate: (d: string) => string;
  leaveTypes: LeaveType[];
  onApprove: (id: string, status: string) => void;
  request: LeaveRequestItem;
  t: TFunction;
}

export function LeaveRequestRow({ request, emp, leaveTypes, formatDate, t, onApprove }: LeaveRequestRowProps) {
  return (
    <tr className="hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-all group">
      <td className="px-10 py-6">
        <div className="flex items-center gap-5">
          <div className="w-14 h-14 bg-primary-50 dark:bg-primary-950 rounded-2xl flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold text-xl shadow-inner border border-primary-100 dark:border-primary-800/50 uppercase group-hover:scale-110 transition-transform">
            {emp?.full_name[0] ?? "?"}
          </div>
          <div>
            <div className="font-bold text-primary-700 dark:text-primary-300 text-sm tracking-tight uppercase leading-none mb-1.5">{emp?.full_name ?? "N/A"}</div>
            <div className="text-[10px] text-primary-400 font-mono font-bold uppercase tracking-widest opacity-70">{emp?.employee_id_number}</div>
          </div>
        </div>
      </td>
      <td className="px-8 py-6"><span className="px-3 py-1 bg-slate-100 dark:bg-slate-700 text-primary-700 dark:text-primary-300 rounded-lg text-[9px] font-bold uppercase tracking-widest border border-primary-200 dark:border-primary-800/50 shadow-sm">{(leaveTypes.find(tOption => tOption.id === request.leave_type_id)?.name ?? "Unknown").toUpperCase()}</span></td>
      <td className="px-8 py-6">
        <div className="text-xs font-bold text-primary-700 dark:text-primary-300 uppercase tracking-tight mb-1.5">
          {formatDate(request.start_date)} — {formatDate(request.end_date)}
          {request.duration_type !== "full_day" ? (
            <span className={cn("ml-2 px-2 py-0.5 rounded-md text-[8px] font-bold uppercase tracking-wider inline-flex items-center gap-1 border",
              request.duration_type === "first_half" ? "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-200 dark:border-amber-800" :
              request.duration_type === "second_half" ? "bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-200 dark:border-blue-800" :
              "bg-cyan-50 dark:bg-cyan-950 text-cyan-600 dark:text-cyan-300 border-cyan-200 dark:border-cyan-800"
            )}>
              {request.duration_type === "first_half" ? <><Sun size={8} /> {t("leave_requests.duration.first_half")}</> : null}
              {request.duration_type === "second_half" ? <><Moon size={8} /> {t("leave_requests.duration.second_half")}</> : null}
              {request.duration_type === "custom_hours" ? <><Timer size={8} /> {request.start_time?.slice(0,5) ?? ""}–{request.end_time?.slice(0,5) ?? ""}</> : null}
            </span>
          ) : null}
        </div>
        <div className="bg-slate-50 dark:bg-slate-800/50 p-3 rounded-xl border border-primary-100 dark:border-primary-800/50 shadow-inner group-hover:bg-white dark:hover:bg-slate-900 transition-colors">
          <p className="text-[10px] text-primary-400 font-bold leading-relaxed line-clamp-1 italic opacity-80 flex gap-2"><MessageSquare className="text-slate-300 dark:text-slate-600 mt-0.5 shrink-0" size={12} /> "{request.reason ?? t("common.no_description")}"</p>
        </div>
      </td>
      <td className="px-8 py-6 text-center">
        <span className={cn("px-4 py-1.5 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm inline-block", 
          request.status === "pending" ? "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800" : 
          request.status === "approved" ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" :
          request.status === "pre_approved" ? "bg-teal-50 dark:bg-teal-950 text-teal-600 dark:text-teal-300 border-teal-100 dark:border-teal-800" :
          "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800"
        )}>
          {request.status === "pending" ? t("common.status_pending").toUpperCase() : request.status === "approved" ? t("common.status_approved").toUpperCase() : request.status === "pre_approved" ? <span className="flex items-center gap-1"><ShieldCheck size={10} />{t("common.status_pre_approved").toUpperCase()}</span> : t("common.status_rejected").toUpperCase()}
        </span>
      </td>
      <td className="px-10 py-6 text-right">
        {request.status === "pending" ? (
          <div className="flex justify-end gap-3 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-x-2 group-hover:translate-x-0">
            <button className="p-3 bg-white dark:bg-slate-900 text-emerald-400 hover:text-emerald-600 dark:text-emerald-300 rounded-xl shadow-md border border-primary-100 dark:border-primary-800/50 transition-all active:scale-90" onClick={() => { onApprove(request.id, "approved"); }} title={t("common.approve")}><CheckCircle2 size={20} /></button>
            <button className="p-3 bg-white dark:bg-slate-900 text-rose-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-primary-100 dark:border-primary-800/50 transition-all active:scale-90" onClick={() => { onApprove(request.id, "rejected"); }} title={t("common.reject")}><XCircle size={20} /></button>
          </div>
        ) : null}
      </td>
    </tr>
  );
}
