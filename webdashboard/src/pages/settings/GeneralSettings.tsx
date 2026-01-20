import { useState, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Settings2, Save, Clock, Palette, Globe, Calendar, Languages } from "lucide-react";
import { useToast } from "@/components/Toast";
import { fetchWorkspace, updateWorkspace, getCurrentWorkspaceId, Workspace } from "@/lib/api";
import { Combobox } from "@/components/ui/Combobox";
import { useTheme } from "@/hooks/use-theme";
import { useTranslation } from "react-i18next";

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
    onError: () => {
      toast.error(t("common.error"));
    }
  });

  function handleSaveConfig() {
    updateWsMutation.mutate({
      date_format: dateFormat,
      time_format: timeFormat,
      timezone: timezone,
      theme_primary_color: themePrimaryColor,
      default_language: defaultLanguage,
    });
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 tracking-tight">
            {t("common.settings.categories.general")}
          </h1>
          <p className="text-slate-500 text-sm mt-1">
            {t("common.workspace_settings.description")}
          </p>
        </div>
        <button
          onClick={handleSaveConfig}
          disabled={updateWsMutation.isPending}
          className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 disabled:opacity-50 text-sm"
        >
          <Save className="w-4 h-4" />
          {t("common.workspace_settings.save_config")}
        </button>
      </div>

      {/* Tabs Navigation */}
      <div className="flex items-center gap-1 bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50">
        <button
          onClick={() => setActiveTab("localization")}
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${
            activeTab === "localization"
              ? "bg-white text-primary-600 shadow-sm border border-slate-100"
              : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <Globe size={14} /> {t("common.settings.tabs.localization", "Localization")}
        </button>
        <button
          onClick={() => setActiveTab("appearance")}
          className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${
            activeTab === "appearance"
              ? "bg-white text-primary-600 shadow-sm border border-slate-100"
              : "text-slate-500 hover:text-slate-700"
          }`}
        >
          <Palette size={14} /> {t("common.settings.tabs.appearance", "Appearance")}
        </button>
      </div>

      <div className="animate-in fade-in duration-500">
        {activeTab === "localization" && (
          <div className="bg-white p-8 rounded-[32px] border border-slate-100 shadow-sm">
            <h2 className="text-lg font-bold mb-6 flex items-center gap-2 text-slate-800">
              <Globe className="w-5 h-5 text-indigo-500" />
              {t("common.workspace_settings.general_config")}
            </h2>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">
                  {t("common.workspace_settings.date_format")}
                </label>
                <Combobox
                  options={[
                    { value: "DD/MM/YYYY", label: "DD/MM/YYYY", description: t("common.workspace_settings.date_format_options.vn"), icon: <Calendar className="w-4 h-4" /> },
                    { value: "MM/DD/YYYY", label: "MM/DD/YYYY", description: t("common.workspace_settings.date_format_options.us"), icon: <Calendar className="w-4 h-4" /> },
                    { value: "YYYY-MM-DD", label: "YYYY-MM-DD", description: t("common.workspace_settings.date_format_options.iso"), icon: <Calendar className="w-4 h-4" /> },
                  ]}
                  value={dateFormat}
                  onChange={setDateFormat}
                  className="bg-slate-50 border-slate-200"
                />
              </div>

              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">
                  {t("common.workspace_settings.time_format")}
                </label>
                <Combobox
                  options={[
                    { value: "24h", label: t("common.workspace_settings.time_format_options.24h"), description: t("common.workspace_settings.time_format_options.example_24h"), icon: <Clock className="w-4 h-4" /> },
                    { value: "12h", label: t("common.workspace_settings.time_format_options.12h"), description: t("common.workspace_settings.time_format_options.example_12h"), icon: <Clock className="w-4 h-4" /> },
                  ]}
                  value={timeFormat}
                  onChange={setTimeFormat}
                  className="bg-slate-50 border-slate-200"
                />
              </div>

              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">
                  {t("common.workspace_settings.timezone")}
                </label>
                <Combobox
                  options={[
                    { value: "Asia/Ho_Chi_Minh", label: "Asia/Ho_Chi_Minh", description: t("common.workspace_settings.timezone_options.vn"), icon: <Globe className="w-4 h-4" /> },
                    { value: "Asia/Bangkok", label: "Asia/Bangkok", description: t("common.workspace_settings.timezone_options.th"), icon: <Globe className="w-4 h-4" /> },
                    { value: "UTC", label: t("common.workspace_settings.timezone_options.utc"), icon: <Globe className="w-4 h-4" /> },
                  ]}
                  value={timezone}
                  onChange={setTimezone}
                  className="bg-slate-50 border-slate-200"
                />
              </div>

              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">
                  {t("common.workspace_settings.default_language")}
                </label>
                <Combobox
                  options={[
                    { value: "vi", label: t("common.languages.vi"), icon: <Languages className="w-4 h-4" /> },
                    { value: "en", label: t("common.languages.en"), icon: <Languages className="w-4 h-4" /> },
                    { value: "zh", label: t("common.languages.zh"), icon: <Languages className="w-4 h-4" /> },
                  ]}
                  value={defaultLanguage}
                  onChange={setDefaultLanguage}
                  className="bg-slate-50 border-slate-200"
                />
              </div>
            </div>
          </div>
        )}

        {activeTab === "appearance" && (
          <div className="bg-white p-8 rounded-[32px] border border-slate-100 shadow-sm">
            <h2 className="text-lg font-bold mb-6 flex items-center gap-2 text-slate-800">
              <Palette className="w-5 h-5 text-pink-500" />
              {t("common.workspace_settings.theme_config")}
            </h2>
            <div className="space-y-8">
              <div className="space-y-4">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">
                  {t("common.workspace_settings.primary_color")}
                </label>
                <div className="flex flex-wrap gap-4 items-center">
                  {[
                    { name: t("common.colors.violet"), color: '#7c3aed' },
                    { name: t("common.colors.indigo"), color: '#4f46e5' },
                    { name: t("common.colors.blue"), color: '#2563eb' },
                    { name: t("common.colors.emerald"), color: '#059669' },
                    { name: t("common.colors.amber"), color: '#d97706' },
                    { name: t("common.colors.rose"), color: '#e11d48' },
                    { name: t("common.colors.slate"), color: '#475569' },
                  ].map((preset) => (
                    <button
                      key={preset.color}
                      onClick={() => setThemePrimaryColor(preset.color)}
                      className={`w-12 h-12 rounded-2xl transition-all border-2 flex items-center justify-center ${
                        themePrimaryColor === preset.color ? 'border-slate-900 scale-110 shadow-lg' : 'border-transparent hover:scale-105 shadow-sm'
                      }`}
                      style={{ backgroundColor: preset.color }}
                      title={preset.name}
                    >
                      {themePrimaryColor === preset.color && <div className="w-2.5 h-2.5 bg-white rounded-full shadow-sm" />}
                    </button>
                  ))}
                  
                  <div className="h-10 w-[1px] bg-slate-100 mx-2" />
                  
                  <div className="flex items-center gap-3 bg-slate-50 p-2 rounded-2xl border border-slate-100 shadow-inner">
                    <input 
                      type="color" 
                      value={themePrimaryColor} 
                      onChange={(e) => setThemePrimaryColor(e.target.value)}
                      className="w-10 h-10 rounded-xl cursor-pointer bg-transparent border-none p-0"
                    />
                    <input 
                      type="text" 
                      value={themePrimaryColor} 
                      onChange={(e) => setThemePrimaryColor(e.target.value)}
                      className="bg-transparent border-none text-xs font-mono font-black text-slate-600 w-24 outline-none uppercase"
                    />
                  </div>
                </div>
              </div>
              
              <div className="p-6 bg-slate-50 border border-slate-100 rounded-3xl">
                <p className="text-xs text-slate-500 leading-relaxed italic">
                  {t("common.workspace_settings.theme_desc", "This color will be the primary brand color for all members in this workspace.")}
                </p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
