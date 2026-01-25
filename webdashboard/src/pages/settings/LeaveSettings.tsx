import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { LeaveTypeManager } from "@/components/leave/LeaveTypeManager";
import { LeavePolicyManager } from "@/components/leave/LeavePolicyManager";
import { AssignPolicyModal } from "@/components/leave/AssignPolicyModal";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";
import { Tabs } from "@/components/ui/Tabs";
import { CalendarDays, ShieldCheck, UserPlus, Info } from "lucide-react";
import { cn } from "@/lib/utils";

type TabType = "types" | "policies" | "assignments";

export default function LeaveSettings() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [activeTab, setActiveTab] = useState<TabType>("types");
  const [isAssignModalOpen, setIsAssignModalOpen] = useState(false);

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
            className={cn(
              "flex items-center gap-3 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all duration-300",
              activeTab === "assignments"
                ? "bg-white text-primary-600 shadow-sm border border-slate-100"
                : "text-slate-500 hover:text-slate-800"
            )}
          >
            <UserPlus className="w-4 h-4" />
            {t("settings.leave.tabs.assignments", "Assignments")}
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
                    Policy Assignments
                  </h3>
                  <p className="text-xs font-bold text-slate-500 mt-1 uppercase tracking-tight">
                    Assign policies to employees or departments
                  </p>
                </div>
                <button
                  onClick={() => setIsAssignModalOpen(true)}
                  className="flex items-center gap-2 rounded-xl bg-primary-600 px-6 py-2.5 text-[10px] font-bold text-white uppercase tracking-widest hover:bg-primary-700 shadow-lg shadow-primary-100"
                >
                  <UserPlus className="w-4 h-4" />
                  Assign Policy
                </button>
              </div>
              <div className="bg-amber-50 border border-amber-100 p-8 rounded-[2.5rem] flex items-start gap-6">
                <Info className="text-amber-600 w-6 h-6 mt-1 shrink-0" />
                <div className="space-y-2">
                  <p className="font-bold text-amber-900 uppercase tracking-widest text-xs">Important Note</p>
                  <p className="text-sm text-amber-800 leading-relaxed">
                    Policy assignments define how leave is calculated for each person. Changes will be applied during the next scheduled accrual run or can be forced via the "Accrual Run" tool in General Settings.
                  </p>
                </div>
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
