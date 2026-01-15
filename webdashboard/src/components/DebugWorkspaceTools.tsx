import { useState } from "react";
import { useTranslation } from "react-i18next";
import { AlertTriangle, RefreshCcw, FileText, Calendar, Clock, Banknote } from "lucide-react";
import { clearWorkspaceData, getCurrentWorkspaceId } from "../lib/api";
import { useToast } from "./Toast";

export default function DebugWorkspaceTools() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId();
  const toast = useToast();
  const [loading, setLoading] = useState<string | null>(null);

  const handleClear = async (group: string, label: string) => {
    if (!wsId) return;
    if (!confirm(t("debug_tools.messages.confirm_clear", { label }))) {
      return;
    }

    setLoading(group);
    try {
      await clearWorkspaceData(wsId, group);
      toast.success(t("debug_tools.messages.clear_success", { label }));
    } catch (e: any) {
      toast.error(`${t("debug_tools.messages.clear_error")} ${e.message}`);
    } finally {
      setLoading(null);
    }
  };

  const tools = [
    { id: "attendance", label: t("common.attendance"), icon: Clock, color: "text-amber-500", bg: "bg-amber-50" },
    { id: "schedules", label: t("common.schedule"), icon: Calendar, color: "text-blue-500", bg: "bg-primary-50" },
    { id: "leave", label: t("common.leave"), icon: FileText, color: "text-emerald-500", bg: "bg-emerald-50" },
    { id: "payroll", label: t("common.payroll_and_contracts"), icon: Banknote, color: "text-purple-500", bg: "bg-purple-50" },
    { id: "full_reset", label: t("debug_tools.full_reset_label"), icon: AlertTriangle, color: "text-rose-500", bg: "bg-rose-50" },
  ];

  return (
    <div className="bg-white p-8 rounded-3xl border border-rose-100 shadow-sm">
      <div className="mb-6">
        <h2 className="text-xl font-bold flex items-center gap-2 text-rose-600">
          <RefreshCcw className="w-5 h-5" />
          {t("debug_tools.title")}
        </h2>
        <p className="text-slate-500 text-sm mt-1">
          {t("debug_tools.subtitle")} 
          <span className="font-bold text-rose-500 ml-1">{t("debug_tools.warning")}</span>
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {tools.map((tool) => (
          <button
            key={tool.id}
            onClick={() => handleClear(tool.id, tool.label)}
            disabled={!!loading}
            className={`flex flex-col items-start p-4 rounded-2xl border border-slate-100 hover:border-rose-200 transition-all text-left group disabled:opacity-50`}
          >
            <div className={`p-3 rounded-xl ${tool.bg} ${tool.color} mb-3 group-hover:scale-110 transition-transform`}>
              <tool.icon className="w-5 h-5" />
            </div>
            <span className="font-bold text-slate-700">{tool.label}</span>
            <span className="text-[10px] text-slate-400 font-medium uppercase tracking-wider mt-1">
              {loading === tool.id ? t("common.deleting") : t("debug_tools.click_to_clear")}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
