import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { LeaveTypeManager } from "@/components/leave/LeaveTypeManager";
import { LeavePolicyManager } from "@/components/leave/LeavePolicyManager";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";
import { Tabs } from "@/components/ui/Tabs";
import { CalendarDays, ShieldCheck } from "lucide-react";
import { cn } from "@/lib/utils";

type TabType = "types" | "policies";

export default function LeaveSettings() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const [activeTab, setActiveTab] = useState<TabType>("types");

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
        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as TabType)}>
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
          </div>

          {activeTab === "types" && (
            <div className="animate-in fade-in slide-in-from-bottom-2 duration-500">
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
            <div className="animate-in fade-in slide-in-from-bottom-2 duration-500">
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
        </Tabs>
      </div>
    </div>
  );
}
