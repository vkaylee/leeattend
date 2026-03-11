import { useState, useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Clock, Calculator, Save, ShieldAlert, Settings2, LayoutGrid, List, Zap } from "lucide-react";
import { cn } from "@/lib/utils";
import { useToast } from "@/components/ui/Toast";
import Button from "@/components/ui/Button";
import { LABEL_BASE } from "@/components/ui/FormField";
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



  const [searchParams, setSearchParams] = useSearchParams();
  const activeTab = (searchParams.get("tab") as TabType) || "general";

  const handleTabChange = (tab: TabType) => {
    setSearchParams({ tab });
  };

  const [workUnit, setWorkUnit] = useState(workspace.work_unit_minutes);

  const [roundingPolicy, setRoundingPolicy] = useState(workspace.rounding_policy);



  const [earlyArrivalStrategy, setEarlyArrivalStrategy] = useState(workspace.early_arrival_strategy || "block");



  const updateWsMutation = useMutation({

    mutationFn: (data: Partial<Workspace>) => updateWorkspace(wsId, data),

    onSuccess: () => {

      toast.success(t("common.success"));

      queryClient.invalidateQueries({ queryKey: ["workspace", wsId] });

    },

  });



  function handleSaveConfig() {

    updateWsMutation.mutate({

      work_unit_minutes: workUnit,

      rounding_policy: roundingPolicy,

      early_arrival_strategy: earlyArrivalStrategy as any

    });

  }





  return (

    <div className="space-y-8 animate-in fade-in duration-500">

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">

        <div className="flex items-center gap-4">

          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><Clock size={24} /></div>

          <div>

            <h1 className="text-3xl font-bold tracking-tight text-slate-700 dark:text-slate-200 leading-tight">{t("common.settings.categories.time")}</h1>

            <p className="text-slate-500 dark:text-slate-400 mt-1 font-medium">{t("common.settings.categories.time_desc")}</p>

          </div>

        </div>

        {activeTab === "general" && (

          <Button onClick={handleSaveConfig} loading={updateWsMutation.isPending} icon={<Save size={18} />}>{t("common.workspace_settings.save_config").toUpperCase()}</Button>

        )}

      </div>



      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">

        <button onClick={() => handleTabChange("general")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'general' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Settings2 size={14} /> {t("common.settings.tabs.general")}</button>

        <button onClick={() => handleTabChange("patterns")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'patterns' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><LayoutGrid size={14} /> {t("common.settings.tabs.patterns")}</button>

        <button onClick={() => handleTabChange("rules")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'rules' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")}><Zap size={14} /> {t("common.settings.tabs.rules")}</button>

      </div>



      <div className="animate-in slide-in-from-bottom-4 duration-500">

        {activeTab === "general" && (

          <div className="bg-white dark:bg-slate-900 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm space-y-10">

            <div className="flex items-center gap-3 border-b border-slate-50 dark:border-slate-800 pb-6"><Calculator className="w-6 h-6 text-indigo-500" /><h2 className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase">{t("common.workspace_settings.attendance_config")}</h2></div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-10">

              <div className="space-y-3">

                <label className={LABEL_BASE}>{t("common.workspace_settings.work_unit")}</label>

                <Combobox options={[{ value: "1", label: t("common.workspace_settings.work_unit_options.1min"), description: t("common.workspace_settings.work_unit_options.1min_desc"), icon: <Clock size={14} /> }, { value: "15", label: t("common.workspace_settings.work_unit_options.15min"), description: t("common.workspace_settings.work_unit_options.15min_desc"), icon: <Clock size={14} /> }, { value: "30", label: t("common.workspace_settings.work_unit_options.30min"), description: t("common.workspace_settings.work_unit_options.30min_desc"), icon: <Clock size={14} /> }, { value: "60", label: t("common.workspace_settings.work_unit_options.60min"), description: t("common.workspace_settings.work_unit_options.60min_desc"), icon: <Clock size={14} /> }]} value={workUnit.toString()} onChange={(v) => setWorkUnit(parseInt(v))} className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" />

              </div>

              <div className="space-y-3">

                <label className={LABEL_BASE}>{t("common.workspace_settings.rounding_policy")}</label>

                <Combobox options={[{ value: "none", label: t("common.workspace_settings.rounding_options.none"), icon: <ShieldAlert size={14} /> }, { value: "floor", label: t("common.workspace_settings.rounding_options.floor"), icon: <Calculator size={14} /> }, { value: "ceil", label: t("common.workspace_settings.rounding_options.ceil"), icon: <Calculator size={14} /> }, { value: "round", label: t("common.workspace_settings.rounding_options.round"), icon: <Calculator size={14} /> }]} value={roundingPolicy} onChange={setRoundingPolicy} className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" />

              </div>





              <div className="space-y-3 md:col-span-2">



                <label className={LABEL_BASE}>{t("common.workspace_settings.early_arrival_strategy")}</label>



                <Combobox



                  options={[



                    { value: "block", label: t("common.workspace_settings.early_arrival_options.block"), description: t("common.workspace_settings.early_arrival_options.block_desc"), icon: <ShieldAlert size={14} /> },



                    { value: "allow", label: t("common.workspace_settings.early_arrival_options.allow"), description: t("common.workspace_settings.early_arrival_options.allow_desc"), icon: <Zap size={14} /> },



                    { value: "clamp", label: t("common.workspace_settings.early_arrival_options.clamp"), description: t("common.workspace_settings.early_arrival_options.clamp_desc"), icon: <Settings2 size={14} /> }



                  ]}



                  value={earlyArrivalStrategy}



                  onChange={(v) => setEarlyArrivalStrategy(v as any)}



                  className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14"



                />



              </div>





            </div>



          </div>

        )}



        {activeTab === "patterns" && (

          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4"><WorkingPatternsPage /></div>

        )}



        {activeTab === "rules" && (

          <div className="bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden shadow-sm p-4"><AssignmentRulesPage /></div>

        )}

      </div>

    </div>

  );

}
