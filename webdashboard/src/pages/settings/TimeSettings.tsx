import { useState, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Clock, Calculator, Save, ShieldAlert, Settings2, Shield, LayoutGrid, List, Zap } from "lucide-react";
import { useToast } from "@/components/Toast";
import { fetchWorkspace, updateWorkspace, getCurrentWorkspaceId, Workspace } from "@/lib/api";
import { Combobox } from "@/components/ui/Combobox";
import { useTranslation } from "react-i18next";
import WorkingPatternsPage from "@/pages/WorkingPatternsPage";
import AssignmentRulesPage from "@/pages/AssignmentRulesPage";

export default function TimeSettings() {
  const wsId = getCurrentWorkspaceId();

  const { data: workspace, isLoading: isLoadingWs } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => fetchWorkspace(wsId!),
    enabled: !!wsId,
  });

  if (isLoadingWs) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>
    );
  }

  if (!workspace) return null;

  return <TimeSettingsContent key={workspace.id} workspace={workspace} />;
}

type TabType = "general" | "patterns" | "rules";

function TimeSettingsContent({ workspace }: { workspace: Workspace }) {
  const { t } = useTranslation();
  const wsId = workspace.id;
  const queryClient = useQueryClient();
  const toast = useToast();

  const [activeTab, setActiveTab] = useState<TabType>("general");
  const [workUnit, setWorkUnit] = useState(workspace.work_unit_minutes);
  const [roundingPolicy, setRoundingPolicy] = useState(workspace.rounding_policy);
  const [pairingAlgorithm, setPairingAlgorithm] = useState(workspace.pairing_algorithm || "dp_pattern");

  const updateWsMutation = useMutation({
    mutationFn: (data: Partial<Workspace>) => updateWorkspace(wsId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      queryClient.invalidateQueries({ queryKey: ["workspace", wsId] });
    },
    onError: () => {
      toast.error(t("common.error"));
    }
  });

  function handleSaveConfig() {
    updateWsMutation.mutate({
      work_unit_minutes: workUnit,
      rounding_policy: roundingPolicy,
      pairing_algorithm: pairingAlgorithm,
    });
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 tracking-tight">
            {t("common.settings.categories.time")}
          </h1>
          <p className="text-slate-500 text-sm mt-1">
            {t("common.settings.categories.time_desc")}
          </p>
        </div>
        {activeTab === "general" && (
          <button
            onClick={handleSaveConfig}
            disabled={updateWsMutation.isPending}
            className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 disabled:opacity-50 text-sm"
          >
            <Save className="w-4 h-4" />
            {t("common.workspace_settings.save_config")}
          </button>
        )}
      </div>

      {/* Tabs Navigation */}
      <div className="flex items-center gap-1 bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50">
        <button
          onClick={() => setActiveTab("general")}
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${
            activeTab === "general"
              ? "bg-white text-primary-600 shadow-sm border border-slate-100"
              : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <Settings2 size={14} /> {t("common.settings.tabs.general", "General")}
        </button>
        <button
          onClick={() => setActiveTab("patterns")}
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${
            activeTab === "patterns"
              ? "bg-white text-primary-600 shadow-sm border border-slate-100"
              : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <LayoutGrid size={14} /> {t("common.settings.tabs.patterns", "Working Patterns")}
        </button>
        <button
          onClick={() => setActiveTab("rules")}
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${
            activeTab === "rules"
              ? "bg-white text-primary-600 shadow-sm border border-slate-100"
              : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <Zap size={14} /> {t("common.settings.tabs.rules", "Auto-Assignment")}
        </button>
      </div>

      <div className="animate-in fade-in duration-500">
        {activeTab === "general" && (
          <div className="bg-white p-8 rounded-3xl border border-slate-100 shadow-sm">
            <h2 className="text-lg font-bold mb-6 flex items-center gap-2 text-slate-800">
              <Calculator className="w-5 h-5 text-indigo-500" />
              {t("common.workspace_settings.attendance_config")}
            </h2>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                  {t("common.workspace_settings.work_unit")}
                </label>
                <Combobox
                  options={[
                    { value: "1", label: t("common.workspace_settings.work_unit_options.1min"), description: t("common.workspace_settings.work_unit_options.1min_desc"), icon: <Clock className="w-4 h-4" /> },
                    { value: "15", label: t("common.workspace_settings.work_unit_options.15min"), description: t("common.workspace_settings.work_unit_options.15min_desc"), icon: <Clock className="w-4 h-4" /> },
                    { value: "30", label: t("common.workspace_settings.work_unit_options.30min"), description: t("common.workspace_settings.work_unit_options.30min_desc"), icon: <Clock className="w-4 h-4" /> },
                    { value: "60", label: t("common.workspace_settings.work_unit_options.60min"), description: t("common.workspace_settings.work_unit_options.60min_desc"), icon: <Clock className="w-4 h-4" /> },
                  ]}
                  value={workUnit.toString()}
                  onChange={(v) => setWorkUnit(parseInt(v))}
                  className="bg-slate-50 border-slate-200"
                />
              </div>

              <div className="space-y-2">
                <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                  {t("common.workspace_settings.rounding_policy")}
                </label>
                <Combobox
                  options={[
                    { value: "none", label: t("common.workspace_settings.rounding_options.none"), icon: <ShieldAlert className="w-4 h-4" /> },
                    { value: "floor", label: t("common.workspace_settings.rounding_options.floor"), icon: <Calculator className="w-4 h-4" /> },
                    { value: "ceil", label: t("common.workspace_settings.rounding_options.ceil"), icon: <Calculator className="w-4 h-4" /> },
                    { value: "round", label: t("common.workspace_settings.rounding_options.round"), icon: <Calculator className="w-4 h-4" /> },
                  ]}
                  value={roundingPolicy}
                  onChange={setRoundingPolicy}
                  className="bg-slate-50 border-slate-200"
                />
              </div>

              <div className="space-y-2">
                <label className="text-xs font-black text-slate-400 uppercase tracking-widest ml-1">
                  {t("common.workspace_settings.pairing_algorithm")}
                </label>
                <Combobox
                  options={[
                    { value: "dp_pattern", label: t("common.workspace_settings.algorithm_options.dp_pattern"), description: t("common.workspace_settings.algorithm_options.dp_pattern_desc"), icon: <Settings2 className="w-4 h-4" /> },
                    { value: "anchor_first", label: t("common.workspace_settings.algorithm_options.anchor_first"), description: t("common.workspace_settings.algorithm_options.anchor_first_desc"), icon: <Clock className="w-4 h-4" /> },
                    { value: "sequential", label: t("common.workspace_settings.algorithm_options.sequential"), description: t("common.workspace_settings.algorithm_options.sequential_desc"), icon: <Calculator className="w-4 h-4" /> },
                    { value: "fsm", label: t("common.workspace_settings.algorithm_options.fsm"), description: t("common.workspace_settings.algorithm_options.fsm_desc"), icon: <Shield className="w-4 h-4" /> },
                  ]}
                  value={pairingAlgorithm}
                  onChange={setPairingAlgorithm}
                  className="bg-slate-50 border-slate-200"
                />
              </div>
            </div>
          </div>
        )}

        {activeTab === "patterns" && (
          <div className="bg-white rounded-3xl border border-slate-100 overflow-hidden shadow-sm">
            <div className="p-1">
              <WorkingPatternsPage />
            </div>
          </div>
        )}

        {activeTab === "rules" && (
          <div className="bg-white rounded-3xl border border-slate-100 overflow-hidden shadow-sm">
            <div className="p-1">
              <AssignmentRulesPage />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}