import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { LeaveTypeManager } from "@/components/leave/LeaveTypeManager";
import { LeavePolicyManager } from "@/components/leave/LeavePolicyManager";
import { AssignPolicyModal } from "@/components/leave/AssignPolicyModal";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";
import { CalendarDays, ShieldCheck, UserPlus, Info } from "lucide-react";
import { cn } from "@/lib/utils";
import { useQuery } from "@tanstack/react-query";
import { fetchLeavePolicyAssignments } from "@/lib/api";
import { format } from "date-fns";


type TabType = "types" | "policies" | "assignments";

export default function LeaveSettings() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [activeTab, setActiveTab] = useState<TabType>("types");
  const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);

  const { data: assignments = [], isLoading: assignmentsLoading } = useQuery({
    queryKey: ["leave-assignments", wsId],
    queryFn: () => fetchLeavePolicyAssignments(wsId!),
    enabled: !!wsId && activeTab === "assignments"
  });


  if (!wsId) return null;

  return (
    <div className="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <header className="flex flex-col gap-2">
        <div className="flex items-center gap-4">
          <div className="bg-primary-600 p-3 rounded-2xl shadow-xl shadow-primary-100 rotate-3">
            <CalendarDays className="text-white w-6 h-6" />
          </div>
          <div>
            <h2 className="text-3xl font-bold tracking-tighter text-slate-800 uppercase leading-none">
              {t("settings.leave.title")}
            </h2>
            <p className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.25em] mt-1 opacity-60">
              {t("settings.leave.desc")}
            </p>
          </div>
        </div>
      </header>

      <div className="bg-white rounded-3xl border border-slate-100 p-8 shadow-sm">
        <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner mb-8">
          <button
            onClick={() => setActiveTab("types")}
            data-testid="leave-tab-types"
            className={cn(
              "flex items-center gap-3 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all duration-300",
              activeTab === "types"
                ? "bg-white text-primary-600 shadow-sm border border-slate-100"
                : "text-slate-500 hover:text-slate-800"
            )}
          >
            <CalendarDays className="w-4 h-4" />
            {t("settings.leave.tabs.types")}
          </button>
          <button
            onClick={() => setActiveTab("policies")}
            data-testid="leave-tab-policies"
            className={cn(
              "flex items-center gap-3 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all duration-300",
              activeTab === "policies"
                ? "bg-white text-primary-600 shadow-sm border border-slate-100"
                : "text-slate-500 hover:text-slate-800"
            )}
          >
            <ShieldCheck className="w-4 h-4" />
            {t("settings.leave.tabs.policies")}
          </button>
          <button
            onClick={() => setActiveTab("assignments")}
            data-testid="leave-tab-assignments"
            className={cn(
              "flex items-center gap-3 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all duration-300",
              activeTab === "assignments"
                ? "bg-white text-primary-600 shadow-sm border border-slate-100"
                : "text-slate-500 hover:text-slate-800"
            )}
          >
            <UserPlus className="w-4 h-4" />
            {t("settings.leave.tabs.assignments")}
          </button>
        </div>

        <div className="animate-in fade-in slide-in-from-bottom-2 duration-500">
          {activeTab === "types" && (
            <div>
              <div className="mb-8 border-b border-slate-50 pb-6">
                <h3 className="text-xl font-bold text-slate-800 tracking-tight uppercase">
                  {t("settings.leave.tabs.types")}
                </h3>
                <p className="text-xs font-bold text-slate-500 mt-1 uppercase tracking-tight">
                  {t("settings.leave.tabs.types_desc")}
                </p>
              </div>
              <LeaveTypeManager wsId={wsId} />
            </div>
          )}

          {activeTab === "policies" && (
            <div>
              <div className="mb-8 border-b border-slate-50 pb-6">
                <h3 className="text-xl font-bold text-slate-800 tracking-tight uppercase">
                  {t("settings.leave.tabs.policies")}
                </h3>
                <p className="text-xs font-bold text-slate-500 mt-1 uppercase tracking-tight">
                  {t("settings.leave.tabs.policies_desc")}
                </p>
              </div>
              <LeavePolicyManager wsId={wsId} />
            </div>
          )}

          {activeTab === "assignments" && (
            <div>
              <div className="mb-8 border-b border-slate-50 pb-6 flex justify-between items-end">
                <div>
                  <h3 className="text-xl font-bold text-slate-800 tracking-tight uppercase">
                    {t("settings.leave.tabs.assignments")}
                  </h3>
                  <p className="text-xs font-bold text-slate-500 mt-1 uppercase tracking-tight">
                    {t("settings.leave.tabs.assignments_desc")}
                  </p>
                </div>
                <button
                  onClick={() => setIsAssignModalOpen(true)}
                  data-testid="assign-policy-btn" className="flex items-center gap-2 rounded-xl bg-primary-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 shadow-lg shadow-primary-100"
                >
                  <UserPlus className="w-4 h-4" />
                  {t("settings.leave.actions.assign")}
                </button>
              </div>
              <div className="bg-amber-50 border border-amber-100 p-8 rounded-[2.5rem] flex items-start gap-6">
                <Info className="text-amber-600 w-6 h-6 mt-1 shrink-0" />
                <div className="space-y-2">
                  <p className="font-bold text-amber-900 uppercase tracking-widest text-xs">{t("common.important_note")}</p>
                  <p className="text-sm text-amber-800 leading-relaxed">
                    {t("settings.leave.messages.assignment_note")}
                  </p>
                </div>
              </div>
              <div className="bg-white rounded-3xl border border-slate-100 overflow-hidden shadow-sm mt-8 animate-in slide-in-from-bottom-4 duration-700">
                <table className="w-full text-left border-collapse">
                  <thead className="bg-slate-50 border-b border-slate-100">
                    <tr>
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("settings.leave.table.policy")}</th>
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("settings.leave.table.type")}</th>
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("settings.leave.table.assignee")}</th>
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{t("settings.leave.table.effective_date")}</th>
                      <th className="px-6 py-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest text-right">{t("settings.leave.table.created_at")}</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-50">
                    {assignmentsLoading ? (
                      <tr><td colSpan={5} className="p-8 text-center text-slate-400 italic font-bold text-xs uppercase tracking-widest">{t("settings.leave.messages.loading")}</td></tr>
                    ) : assignments.length === 0 ? (
                      <tr><td colSpan={5} className="p-12 text-center text-slate-400 italic font-bold text-xs uppercase tracking-widest opacity-50">{t("settings.leave.messages.no_assignments")}</td></tr>
                    ) : (
                      assignments.map((a: any) => (
                        <tr key={a.id} className="hover:bg-slate-50/50 transition-colors group">
                          <td className="px-6 py-5 font-bold text-slate-700 text-xs tracking-tight">{a.policy_name}</td>
                          <td className="px-6 py-5 text-[10px] font-bold text-slate-500 uppercase tracking-widest">{a.leave_type_name}</td>
                          <td className="px-6 py-5">
                            <span className={cn("px-2.5 py-1 rounded-lg text-[9px] font-bold uppercase tracking-widest border shadow-sm", 
                              a.assignee_type === "workspace" ? "bg-purple-50 text-purple-600 border-purple-100" :
                              a.assignee_type === "department" ? "bg-blue-50 text-blue-600 border-blue-100" :
                              "bg-emerald-50 text-emerald-600 border-emerald-100"
                            )}>
                              {a.assignee_type}
                            </span>
                          </td>
                          <td className="px-6 py-5 text-xs font-mono text-slate-600 font-bold">
                             {format(new Date(a.effective_from), "MMM dd, yyyy")} 
                             <span className="text-slate-300 mx-2">→</span>
                             {a.effective_to ? format(new Date(a.effective_to), "MMM dd, yyyy") : <span className="text-slate-400 italic">{t("common.forever")}</span>}
                          </td>
                          <td className="px-6 py-5 text-[10px] font-mono text-slate-400 text-right uppercase">
                            {format(new Date(a.created_at), "MMM dd HH:mm")}
                          </td>
                        </tr>
                      ))
                    )}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      </div>

      <AssignPolicyModal 
        isOpen={isAssignModalOpen}
        onClose={() => setIsAssignModalOpen(false)}
        wsId={wsId}
      />
    </div>
  );
}
