import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Settings2, Save, Clock, Palette, Globe, Calendar, Languages, Sun, Moon, Monitor, ImagePlus, Trash2, Type, Building2, UploadCloud } from "lucide-react";
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import type { ThemeMode } from "@/hooks/use-dark-mode";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { LABEL_BASE } from "@/components/ui/form-field";
import { Skeleton } from "@/components/ui/skeleton";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useDarkModeContext } from "@/context/use-dark-mode-context";
import { useTheme } from "@/hooks/use-theme";
import { fetchWorkspace, updateWorkspace, getCurrentWorkspaceId, Workspace } from "@/lib/api";
import { cn } from "@/lib/utils";

type TabType = "localization" | "appearance";

export default function GeneralSettings() {
  const wsId = getCurrentWorkspaceId();

  const { data: workspace, isLoading: isLoadingWs } = useQuery({
    queryKey: ["workspace", wsId],
    queryFn: () => {
      if (!wsId) throw new Error("Workspace ID is required");
      return fetchWorkspace(wsId);
    },
    enabled: !!wsId,
  });

  if (isLoadingWs) {
    return (
      <div className="space-y-6">
        <Skeleton className="h-[200px] w-full rounded-[2.5rem]" />
        <Skeleton className="h-[400px] w-full rounded-[2.5rem]" />
      </div>
    );
  }

  if (!workspace) return null;

  return <GeneralSettingsContent key={workspace.id} workspace={workspace} />;
}

// eslint-disable-next-line max-lines-per-function
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
  const [wsName, setWsName] = useState(workspace.name);
  const [logoUrl, setLogoUrl] = useState(workspace.logo_url ?? "");
  const [tagline, setTagline] = useState(workspace.tagline ?? "");

  useEffect(() => {
    setPreviewTheme(themePrimaryColor);
    return () => { setPreviewTheme(null); };
  }, [themePrimaryColor, setPreviewTheme]);

  const updateWsMutation = useMutation({
    mutationFn: (data: Partial<Workspace>) => updateWorkspace(wsId, data),
    onSuccess: () => {
      toast.success(t("common.success"));
      void queryClient.invalidateQueries({ queryKey: ["workspace", wsId] });
      void queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });

  function handleSaveConfig() {
    updateWsMutation.mutate({ 
      name: wsName,
      date_format: dateFormat, 
      time_format: timeFormat, 
      timezone: timezone, 
      theme_primary_color: themePrimaryColor, 
      default_language: defaultLanguage,
      logo_url: logoUrl || undefined,
      tagline: tagline || undefined,
    });
  }

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><Settings2 size={24} /></div>
          <div>
            <Typography className="text-2xl font-bold tracking-tight text-slate-850 dark:text-slate-100 normal-case mb-0.5" data-testid="general-settings-title" variant="h3">{t("common.settings.categories.general")}</Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium normal-case tracking-normal" variant="small">{t("common.workspace_settings.description")}</Typography>
          </div>
        </div>
        <Button data-testid="save-workspace-config-btn" icon={<Save size={18} />} loading={updateWsMutation.isPending} onClick={handleSaveConfig} variant="ghost">{t("common.workspace_settings.save_config").toUpperCase()}</Button>
      </div>

      <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
        <Button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'localization' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} data-testid="tab-localization" onClick={() => { setActiveTab("localization"); }} variant="ghost"><Globe size={14} /> {t("common.settings.tabs.localization")}</Button>
        <Button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", activeTab === 'appearance' ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700")} data-testid="tab-appearance" onClick={() => { setActiveTab("appearance"); }} variant="ghost"><Palette size={14} /> {t("common.settings.tabs.appearance")}</Button>
      </div>

      <div className="animate-in slide-in-from-bottom-4 duration-500">
        {activeTab === "localization" ? (
          <div className="bg-white dark:bg-slate-900 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm space-y-10">
            <div className="flex items-center gap-3 border-b border-slate-50 dark:border-slate-800 pb-6"><Globe className="w-6 h-6 text-primary-500" /><Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h2">{t("common.workspace_settings.general_config")}</Typography></div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-10">
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.workspace_settings.date_format")}</label>
                <Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" onChange={setDateFormat} options={[{ value: "DD/MM/YYYY", label: "DD/MM/YYYY", description: t("common.workspace_settings.date_format_options.vn"), icon: <Calendar size={14} /> }, { value: "MM/DD/YYYY", label: "MM/DD/YYYY", description: t("common.workspace_settings.date_format_options.us"), icon: <Calendar size={14} /> }, { value: "YYYY-MM-DD", label: "YYYY-MM-DD", description: t("common.workspace_settings.date_format_options.iso"), icon: <Calendar size={14} /> }]} value={dateFormat} />
              </div>
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.workspace_settings.time_format")}</label>
                <Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" onChange={setTimeFormat} options={[{ value: "24h", label: t("common.workspace_settings.time_format_options.24h"), description: t("common.workspace_settings.time_format_options.example_24h"), icon: <Clock size={14} /> }, { value: "12h", label: t("common.workspace_settings.time_format_options.12h"), description: t("common.workspace_settings.time_format_options.example_12h"), icon: <Clock size={14} /> }]} value={timeFormat} />
              </div>
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.workspace_settings.timezone")}</label>
                <Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" data-testid="timezone-select" onChange={setTimezone} options={[{ value: "Asia/Ho_Chi_Minh", label: "Asia/Ho_Chi_Minh", description: t("common.workspace_settings.timezone_options.vn"), icon: <Globe size={14} /> }, { value: "Asia/Bangkok", label: "Asia/Bangkok", description: t("common.workspace_settings.timezone_options.th"), icon: <Globe size={14} /> }, { value: "UTC", label: t("common.workspace_settings.timezone_options.utc"), icon: <Globe size={14} /> }]} value={timezone} />
              </div>
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.workspace_settings.default_language")}</label>
                <Combobox className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 h-14" onChange={setDefaultLanguage} options={[{ value: "vi", label: t("common.languages.vi"), icon: <Languages size={14} /> }, { value: "en", label: t("common.languages.en"), icon: <Languages size={14} /> }, { value: "zh", label: t("common.languages.zh"), icon: <Languages size={14} /> }]} value={defaultLanguage} />
              </div>
            </div>
          </div>
        ) : null}

        {activeTab === "appearance" ? (
          <div className="bg-white dark:bg-slate-900 p-10 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm space-y-10">
            {/* Branding Section */}
            <div className="flex items-center gap-3 border-b border-slate-50 dark:border-slate-800 pb-6">
              <Building2 className="w-6 h-6 text-indigo-500" />
              <Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h2">
                {t("common.workspace_settings.branding", { defaultValue: "Branding" })}
              </Typography>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-10">
              {/* Workspace Name */}
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.workspace_settings.workspace_name", { defaultValue: "Workspace Name" })}</label>
                <div className="relative">
                  <div className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400">
                    <Building2 size={18} />
                  </div>
                  <input
                    className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-2xl text-sm font-medium focus:border-primary-500 focus:ring-4 focus:ring-primary-500/10 transition-all text-slate-700 dark:text-slate-200 outline-none"
                    data-testid="workspace-name-input"
                    onChange={(e) => { setWsName(e.target.value); }}
                    placeholder={t("common.workspace_settings.workspace_name_placeholder", { defaultValue: "e.g., LeeAttend Corp" })}
                    type="text"
                    value={wsName}
                  />
                </div>
              </div>

              {/* Tagline */}
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.workspace_settings.tagline", { defaultValue: "Tagline" })}</label>
                <div className="relative">
                  <div className="absolute left-4 top-1/2 -translate-y-1/2 text-slate-400">
                    <Type size={18} />
                  </div>
                  <input
                    className="w-full h-14 pl-12 pr-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-700 rounded-2xl text-sm font-medium focus:border-primary-500 focus:ring-4 focus:ring-primary-500/10 transition-all text-slate-700 dark:text-slate-200 outline-none"
                    data-testid="workspace-tagline-input"
                    onChange={(e) => { setTagline(e.target.value); }}
                    placeholder={t("common.workspace_settings.tagline_placeholder", { defaultValue: "e.g., Human Resources Management" })}
                    type="text"
                    value={tagline}
                  />
                </div>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-10">
              {/* Logo Section */}
              <div className="space-y-3">
                <label className={LABEL_BASE}>{t("common.workspace_settings.workspace_logo", { defaultValue: "Workspace Logo" })}</label>
                {logoUrl ? (
                  <div className="relative group rounded-2xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/50 flex flex-col items-center justify-center p-6 h-40">
                    <div 
                      aria-label="Workspace Logo"
                      className="h-20 w-full bg-contain bg-no-repeat bg-center drop-shadow-sm"
                      role="img"
                      style={{ backgroundImage: `url(${logoUrl})` }}
                    />
                    <div className="absolute inset-0 bg-white/80 dark:bg-slate-900/80 backdrop-blur-sm opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-3 rounded-2xl">
                      <Button className="p-3 bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-300 rounded-xl shadow-sm hover:text-primary-600 hover:border-primary-100 border border-slate-200 dark:border-slate-700 transition-colors" 
                        data-testid="edit-logo-btn"
                        onClick={() => {
                          const url = prompt(t("common.workspace_settings.enter_logo_url", { defaultValue: "Enter image URL:" }), logoUrl);
                          if (url !== null) setLogoUrl(url);
                        }}
                        title={t("common.edit", { defaultValue: "Edit" })}
                        variant="ghost"
                      >
                        <ImagePlus size={18} />
                      </Button>
                      <Button className="p-3 bg-white dark:bg-slate-800 text-rose-500 rounded-xl shadow-sm hover:bg-rose-50 border border-slate-200 dark:border-slate-700 hover:border-rose-100 transition-colors" 
                        data-testid="delete-logo-btn"
                        onClick={() => { setLogoUrl(""); }}
                        title={t("common.delete", { defaultValue: "Delete" })}
                        variant="ghost"
                      >
                        <Trash2 size={18} />
                      </Button>
                    </div>
                  </div>
                ) : (
                  <Button className="w-full h-40 rounded-2xl border-2 border-dashed border-slate-200 dark:border-slate-700 hover:border-primary-400 dark:hover:border-primary-500 bg-slate-50 dark:bg-slate-800/30 hover:bg-primary-50/50 dark:hover:bg-primary-900/10 transition-all flex flex-col items-center justify-center gap-3 group" 
                    data-testid="upload-logo-btn"
                    onClick={() => {
                      const url = prompt(t("common.workspace_settings.enter_logo_url", { defaultValue: "Enter image URL:" }));
                      if (url) setLogoUrl(url);
                    }}
                    variant="ghost"
                  >
                    <div className="w-12 h-12 rounded-full bg-white dark:bg-slate-800 shadow-sm flex items-center justify-center text-slate-400 group-hover:text-primary-500 group-hover:scale-110 transition-all">
                      <UploadCloud size={24} />
                    </div>
                    <div className="text-center">
                      <Typography className="text-sm font-bold text-slate-600 dark:text-slate-300 group-hover:text-primary-600 dark:group-hover:text-primary-400" variant="p">{t("common.workspace_settings.upload_logo", { defaultValue: "Click to set logo URL" })}</Typography>
                      <Typography className="text-[10px] text-slate-400 font-medium uppercase tracking-wider mt-1" variant="p">{t("common.workspace_settings.logo_reqs", { defaultValue: "SVG, PNG, JPG. Max 2MB." })}</Typography>
                    </div>
                  </Button>
                )}
              </div>
            </div>

            <div className="w-full h-px bg-slate-100 dark:bg-slate-800 my-10" />

            <div className="flex items-center gap-3 border-b border-slate-50 dark:border-slate-800 pb-6"><Palette className="w-6 h-6 text-pink-500" /><Typography className="text-xl font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h2">{t("common.workspace_settings.theme_config")}</Typography></div>
            <div className="space-y-10">
              <div className="space-y-6">
                <label className={LABEL_BASE}>{t("common.workspace_settings.primary_color")}</label>
                <div className="flex flex-wrap gap-5 items-center">
                  {[
                    { name: t("common.colors.emerald"), color: '#10b981' }, { name: t("common.colors.blue"), color: '#2563eb' },
                    { name: t("common.colors.emerald"), color: '#059669' }, { name: t("common.colors.amber"), color: '#d97706' }, { name: t("common.colors.rose"), color: '#e11d48' }, { name: t("common.colors.slate"), color: '#475569' },
                  ].map((preset) => (
                    <Button className={cn("w-14 h-14 rounded-2xl transition-all border-4 flex items-center justify-center shadow-lg group active:scale-90", themePrimaryColor === preset.color ? 'border-slate-900 scale-110' : 'border-white hover:scale-105 hover:border-slate-100 dark:border-slate-800')} key={preset.color} onClick={() => { setThemePrimaryColor(preset.color); }} style={{ backgroundColor: preset.color }} title={preset.name} variant="ghost">
                      {themePrimaryColor === preset.color && <div className="w-3 h-3 bg-white dark:bg-slate-900 rounded-full shadow-2xl animate-in zoom-in duration-300" />}
                    </Button>
                  ))}
                  <div className="h-12 w-px bg-slate-100 dark:bg-slate-700 mx-2" />
                  <div className="flex items-center gap-4 bg-slate-50 dark:bg-slate-800 p-2.5 rounded-[1.5rem] border border-slate-100 dark:border-slate-800 shadow-inner group">
                    <input className="w-12 h-12 rounded-xl cursor-pointer bg-transparent border-none p-0 overflow-hidden" onChange={(e) => { setThemePrimaryColor(e.target.value); }} type="color" value={themePrimaryColor} />
                    <input className="bg-transparent border-none text-sm font-mono font-bold text-slate-600 dark:text-slate-300 w-24 outline-none tracking-tighter" onChange={(e) => { setThemePrimaryColor(e.target.value); }} type="text" value={themePrimaryColor} />
                  </div>
                </div>
              </div>
              <div className="p-8 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[2rem] shadow-inner"><Typography className="text-[11px] font-bold text-slate-500 dark:text-slate-400 leading-relaxed italic uppercase tracking-tight opacity-80" variant="p">{t("common.workspace_settings.theme_desc")}</Typography></div>

              {/* Dark Mode Preference */}
              <DarkModeSelector />
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}

/** Dark mode preference selector with light/system/dark options */
function DarkModeSelector() {
  const { t } = useTranslation();
  const { mode, setMode } = useDarkModeContext();

  const options: { value: ThemeMode; icon: React.ReactNode; label: string; desc: string }[] = [
    { value: 'light', icon: <Sun size={20} />, label: t('common.theme.light', 'Light'), desc: t('common.theme.light_desc', 'Always use light mode') },
    { value: 'system', icon: <Monitor size={20} />, label: t('common.theme.system', 'System'), desc: t('common.theme.system_desc', 'Follow system preference') },
    { value: 'dark', icon: <Moon size={20} />, label: t('common.theme.dark', 'Dark'), desc: t('common.theme.dark_desc', 'Always use dark mode') },
  ];

  return (
    <div className="space-y-6">
      <label className={LABEL_BASE}>{t('common.theme.mode', 'Dark Mode')}</label>
      <div className="grid grid-cols-3 gap-4" data-testid="dark-mode-selector">
        {options.map((opt) => (
          <Button className={cn(
              "flex flex-col items-center gap-3 p-6 rounded-2xl border-2 transition-all active:scale-95",
              mode === opt.value
                ? "border-primary-500 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 shadow-lg shadow-primary-100 dark:shadow-black/30"
                : "border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 text-slate-400 dark:text-slate-500 hover:border-slate-200 dark:hover:border-slate-700 hover:text-slate-600 dark:hover:text-slate-300"
            )}
            data-testid={`dark-mode-option-${opt.value}`}
            key={opt.value}
            onClick={() => { setMode(opt.value); }}
            variant="ghost"
          >
            {opt.icon}
            <span className="text-xs font-bold uppercase tracking-widest">{opt.label}</span>
            <span className="text-[10px] text-slate-500 dark:text-slate-400 font-medium">{opt.desc}</span>
          </Button>
        ))}
      </div>
    </div>
  );
}

