import { useState, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Settings2, Save, Clock, Palette, Globe, Calendar, Languages } from "lucide-react";
import { useToast } from "@/components/Toast";
import { fetchWorkspace, updateWorkspace, getCurrentWorkspaceId, Workspace } from "@/lib/api";
import { Combobox } from "@/components/ui/Combobox";
import { useTheme } from "@/hooks/use-theme";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

type TabType = "localization" | "appearance";

export default function GeneralSettings() {
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

  return <GeneralSettingsContent key={workspace.id} workspace={workspace} />;
}

function GeneralSettingsContent({ workspace }: { workspace: Workspace }) {
  const { t } = useTranslation();
  const wsId = workspace.id;
  const queryClient = useQueryClient();
  const toast = useToast();
  const { setPreviewTheme } = useTheme();

  const [activeTab, setActiveTab] = useState<TabType>("localization");
  const [dateFormat, setDateFormat] = useState(workspace.date_format);
  const [timeFormat, setTimeFormat] = useState(workspace.time_format);
  const [timezone, setTimezone] = useState(workspace.timezone);
  const [themePrimaryColor, setThemePrimaryColor] = useState(workspace.theme_primary_color);
  const [defaultLanguage, setDefaultLanguage] = useState(workspace.default_language);

  useEffect(() => {
    setPreviewTheme(themePrimaryColor);
    return () => setPreviewTheme(null);
  }, [themePrimaryColor, setPreviewTheme]);

  const updateWsMutation = useMutation({
    mutationFn: (data: Partial<Workspace>) => updateWorkspace(wsId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      queryClient.invalidateQueries({ queryKey: ["workspace", wsId] });
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });

  function handleSaveConfig() {
    updateWsMutation.mutate({ date_format: dateFormat, time_format: timeFormat, timezone: timezone, theme_primary_color: themePrimaryColor, default_language: defaultLanguage });
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50"><Settings2 size={24} /></div>
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-slate-900 leading-tight">{t("common.settings.categories.general")}</h1>
            <p className="text-slate-500 mt-1 font-medium">{t("common.workspace_settings.description")}</p>
          </div>
        </div>
        <button onClick={handleSaveConfig} disabled={updateWsMutation.isPending} className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 active:scale-95 disabled:opacity-50"><Save size={18} /> {t("common.workspace_settings.save_config").toUpperCase()}</button>
      </div>

      <div className="flex bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50 shadow-inner">
        <button onClick={() => setActiveTab("localization")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'localization' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><Globe size={14} /> {t("common.settings.tabs.localization")}</button>
        <button onClick={() => setActiveTab("appearance")} className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all", activeTab === 'appearance' ? "bg-white text-primary-600 shadow-sm border border-slate-100" : "text-slate-500 hover:text-slate-700")}><Palette size={14} /> {t("common.settings.tabs.appearance")}</button>
      </div>

      <div className="animate-in slide-in-from-bottom-4 duration-500">
        {activeTab === "localization" && (
          <div className="bg-white p-10 rounded-[2.5rem] border border-slate-100 shadow-sm space-y-10">
            <div className="flex items-center gap-3 border-b border-slate-50 pb-6"><Globe className="w-6 h-6 text-indigo-500" /><h2 className="text-xl font-black text-slate-800 tracking-tight uppercase">{t("common.workspace_settings.general_config")}</h2></div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-10">
              <div className="space-y-3">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.workspace_settings.date_format")}</label>
                <Combobox options={[{ value: "DD/MM/YYYY", label: "DD/MM/YYYY", description: t("common.workspace_settings.date_format_options.vn"), icon: <Calendar size={14} /> }, { value: "MM/DD/YYYY", label: "MM/DD/YYYY", description: t("common.workspace_settings.date_format_options.us"), icon: <Calendar size={14} /> }, { value: "YYYY-MM-DD", label: "YYYY-MM-DD", description: t("common.workspace_settings.date_format_options.iso"), icon: <Calendar size={14} /> }]} value={dateFormat} onChange={setDateFormat} className="bg-slate-50 border-slate-100 h-14" />
              </div>
              <div className="space-y-3">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.workspace_settings.time_format")}</label>
                <Combobox options={[{ value: "24h", label: t("common.workspace_settings.time_format_options.24h"), description: t("common.workspace_settings.time_format_options.example_24h"), icon: <Clock size={14} /> }, { value: "12h", label: t("common.workspace_settings.time_format_options.12h"), description: t("common.workspace_settings.time_format_options.example_12h"), icon: <Clock size={14} /> }]} value={timeFormat} onChange={setTimeFormat} className="bg-slate-50 border-slate-100 h-14" />
              </div>
              <div className="space-y-3">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.workspace_settings.timezone")}</label>
                <Combobox options={[{ value: "Asia/Ho_Chi_Minh", label: "Asia/Ho_Chi_Minh", description: t("common.workspace_settings.timezone_options.vn"), icon: <Globe size={14} /> }, { value: "Asia/Bangkok", label: "Asia/Bangkok", description: t("common.workspace_settings.timezone_options.th"), icon: <Globe size={14} /> }, { value: "UTC", label: t("common.workspace_settings.timezone_options.utc"), icon: <Globe size={14} /> }]} value={timezone} onChange={setTimezone} className="bg-slate-50 border-slate-100 h-14" />
              </div>
              <div className="space-y-3">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.workspace_settings.default_language")}</label>
                <Combobox options={[{ value: "vi", label: t("common.languages.vi"), icon: <Languages size={14} /> }, { value: "en", label: t("common.languages.en"), icon: <Languages size={14} /> }, { value: "zh", label: t("common.languages.zh"), icon: <Languages size={14} /> }]} value={defaultLanguage} onChange={setDefaultLanguage} className="bg-slate-50 border-slate-100 h-14" />
              </div>
            </div>
          </div>
        )}

        {activeTab === "appearance" && (
          <div className="bg-white p-10 rounded-[2.5rem] border border-slate-100 shadow-sm space-y-10">
            <div className="flex items-center gap-3 border-b border-slate-50 pb-6"><Palette className="w-6 h-6 text-pink-500" /><h2 className="text-xl font-black text-slate-800 tracking-tight uppercase">{t("common.workspace_settings.theme_config")}</h2></div>
            <div className="space-y-10">
              <div className="space-y-6">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("common.workspace_settings.primary_color")}</label>
                <div className="flex flex-wrap gap-5 items-center">
                  {[
                    { name: t("common.colors.violet"), color: '#7c3aed' }, { name: t("common.colors.indigo"), color: '#4f46e5' }, { name: t("common.colors.blue"), color: '#2563eb' },
                    { name: t("common.colors.emerald"), color: '#059669' }, { name: t("common.colors.amber"), color: '#d97706' }, { name: t("common.colors.rose"), color: '#e11d48' }, { name: t("common.colors.slate"), color: '#475569' },
                  ].map((preset) => (
                    <button key={preset.color} onClick={() => setThemePrimaryColor(preset.color)} className={cn("w-14 h-14 rounded-2xl transition-all border-4 flex items-center justify-center shadow-lg group active:scale-90", themePrimaryColor === preset.color ? 'border-slate-900 scale-110' : 'border-white hover:scale-105 hover:border-slate-100')} style={{ backgroundColor: preset.color }} title={preset.name}>
                      {themePrimaryColor === preset.color && <div className="w-3 h-3 bg-white rounded-full shadow-2xl animate-in zoom-in duration-300" />}
                    </button>
                  ))}
                  <div className="h-12 w-px bg-slate-100 mx-2" />
                  <div className="flex items-center gap-4 bg-slate-50 p-2.5 rounded-[1.5rem] border border-slate-100 shadow-inner group">
                    <input type="color" value={themePrimaryColor} onChange={(e) => setThemePrimaryColor(e.target.value)} className="w-12 h-12 rounded-xl cursor-pointer bg-transparent border-none p-0 overflow-hidden" />
                    <input type="text" value={themePrimaryColor} onChange={(e) => setThemePrimaryColor(e.target.value)} className="bg-transparent border-none text-sm font-mono font-black text-slate-600 w-24 outline-none uppercase tracking-tighter" />
                  </div>
                </div>
              </div>
              <div className="p-8 bg-slate-50 border border-slate-100 rounded-[2rem] shadow-inner"><p className="text-[11px] font-bold text-slate-400 leading-relaxed italic uppercase tracking-tight opacity-80">{t("common.workspace_settings.theme_desc")}</p></div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
