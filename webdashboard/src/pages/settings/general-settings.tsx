import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Settings2, Save, Clock, Palette, Globe, Calendar, Languages, Sun, Moon, Monitor, ImagePlus, Trash2, Type, Building2, UploadCloud } from "lucide-react";
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import type { ThemeMode } from "@/hooks/use-dark-mode";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
const SETTINGS_LABEL = "text-xs font-semibold text-slate-700 dark:text-slate-300 ml-0.5";
import { SettingsSegmentedControl } from "@/components/ui/settings/settings-segmented-control";
import { Skeleton } from "@/components/ui/skeleton";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useDarkModeContext } from "@/context/use-dark-mode-context";
import { useTheme } from "@/hooks/use-theme";
import { fetchWorkspace, updateWorkspace, getStandaloneOrganizationId, Workspace } from "@/lib/api";
import { cn } from "@/lib/utils";

type TabType = "localization" | "appearance";

export default function GeneralSettings() {
  const wsId = getStandaloneOrganizationId();

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
        <Skeleton className="h-40 w-full rounded-2xl" />
        <Skeleton className="h-80 w-full rounded-2xl" />
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
    onMutate: async (data) => {
      const queryKey = ["workspace", wsId];
      await queryClient.cancelQueries({ queryKey: ["workspace"] });
      const previousWorkspace = queryClient.getQueryData<Workspace>(queryKey);
      const mergeWorkspace = (current: Workspace | undefined): Workspace => ({ ...current, ...data } as Workspace);
      queryClient.setQueriesData<Workspace>({ queryKey: ["workspace"] }, mergeWorkspace);
      return { previousWorkspace };
    },
    onError: (_error, _data, context) => {
      queryClient.setQueryData(["workspace", wsId], context?.previousWorkspace);
    },
    onSuccess: (updatedWorkspace, data) => {
      toast.success(t("common.success"));
      const mergeWorkspace = (current: Workspace | undefined): Workspace => ({ ...current, ...data, ...updatedWorkspace } as Workspace);
      queryClient.setQueriesData<Workspace>({ queryKey: ["workspace"] }, mergeWorkspace);
      void queryClient.invalidateQueries({ queryKey: ["workspace", wsId] });
      void queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });

  function handleSaveConfig() {
    updateWsMutation.mutate({ 
      date_format: dateFormat,
      time_format: timeFormat,
      timezone,
      theme_primary_color: themePrimaryColor,
      name: wsName,
      default_language: defaultLanguage,
      logo_url: logoUrl || undefined,
      tagline: tagline || undefined,
    });
  }

  return (
    <div className="space-y-6 animate-in fade-in duration-300">
      {/* Apple HIG Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-2 border-b border-slate-200/60 dark:border-slate-800/60">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-primary-50 dark:bg-primary-950/70 flex items-center justify-center text-primary-600 dark:text-primary-400 border border-primary-100/50 dark:border-primary-900/50 shadow-2xs">
            <Settings2 size={20} />
          </div>
          <div>
            <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100" data-testid="general-settings-title" variant="h1">
              {t("common.settings.categories.general")}
            </Typography>
            <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium" variant="small">
              {t("common.workspace_settings.description")}
            </Typography>
          </div>
        </div>
        <Button
          className="rounded-xl px-5 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal"
          data-testid="save-workspace-config-btn"
          icon={<Save size={15} />}
          loading={updateWsMutation.isPending}
          onClick={handleSaveConfig}
          variant="primary"
        >
          {t("common.workspace_settings.save_config", { defaultValue: "Lưu thay đổi" })}
        </Button>
      </div>

      {/* Apple HIG Segmented Control */}
      <div className="flex items-center">
        <div className="inline-flex items-center gap-1 rounded-xl bg-slate-100/90 dark:bg-slate-800/80 p-1 border border-slate-200/60 dark:border-slate-700/60" role="tablist">
          <button
            aria-selected={activeTab === "localization"}
            className={cn(
              "flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 select-none",
              activeTab === "localization"
                ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
            )}
            data-testid="tab-localization"
            onClick={() => setActiveTab("localization")}
            role="tab"
            type="button"
          >
            <Globe size={14} />
            <span>{t("common.settings.tabs.localization")}</span>
          </button>
          <button
            aria-selected={activeTab === "appearance"}
            className={cn(
              "flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 select-none",
              activeTab === "appearance"
                ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
            )}
            data-testid="tab-appearance"
            onClick={() => setActiveTab("appearance")}
            role="tab"
            type="button"
          >
            <Palette size={14} />
            <span>{t("common.settings.tabs.appearance")}</span>
          </button>
        </div>
      </div>

      <div className="animate-in fade-in duration-300">
        {activeTab === "localization" ? (
          <div className="bg-white dark:bg-slate-900/90 p-6 sm:p-8 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 shadow-2xs space-y-6">
            <div className="flex items-center gap-2.5 border-b border-slate-100 dark:border-slate-800/80 pb-4">
              <Globe className="w-5 h-5 text-primary-500" />
              <Typography className="text-base font-bold text-slate-800 dark:text-slate-100" variant="h4">
                {t("common.workspace_settings.general_config")}
              </Typography>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className={SETTINGS_LABEL}>{t("common.workspace_settings.date_format")}</label>
                <Combobox className="bg-slate-50 dark:bg-slate-800/80 border-slate-200/70 dark:border-slate-700/70 h-11 rounded-xl normal-case tracking-normal text-xs font-medium px-4" onChange={setDateFormat} options={[{ value: "DD/MM/YYYY", label: "DD/MM/YYYY", description: t("common.workspace_settings.date_format_options.vn"), icon: <Calendar size={14} /> }, { value: "MM/DD/YYYY", label: "MM/DD/YYYY", description: t("common.workspace_settings.date_format_options.us"), icon: <Calendar size={14} /> }, { value: "YYYY-MM-DD", label: "YYYY-MM-DD", description: t("common.workspace_settings.date_format_options.iso"), icon: <Calendar size={14} /> }]} value={dateFormat} />
              </div>
              <div className="space-y-2">
                <label className={SETTINGS_LABEL}>{t("common.workspace_settings.time_format")}</label>
                <Combobox className="bg-slate-50 dark:bg-slate-800/80 border-slate-200/70 dark:border-slate-700/70 h-11 rounded-xl normal-case tracking-normal text-xs font-medium px-4" onChange={setTimeFormat} options={[{ value: "24h", label: t("common.workspace_settings.time_format_options.24h"), description: t("common.workspace_settings.time_format_options.example_24h"), icon: <Clock size={14} /> }, { value: "12h", label: t("common.workspace_settings.time_format_options.12h"), description: t("common.workspace_settings.time_format_options.example_12h"), icon: <Clock size={14} /> }]} value={timeFormat} />
              </div>
              <div className="space-y-2">
                <label className={SETTINGS_LABEL}>{t("common.workspace_settings.timezone")}</label>
                <Combobox className="bg-slate-50 dark:bg-slate-800/80 border-slate-200/70 dark:border-slate-700/70 h-11 rounded-xl normal-case tracking-normal text-xs font-medium px-4" data-testid="timezone-select" onChange={setTimezone} options={[{ value: "Asia/Ho_Chi_Minh", label: "(GMT+07:00) Asia/Ho Chi Minh", description: t("common.workspace_settings.timezone_options.vn"), icon: <Globe size={14} /> }, { value: "Asia/Bangkok", label: "(GMT+07:00) Asia/Bangkok", description: t("common.workspace_settings.timezone_options.th"), icon: <Globe size={14} /> }, { value: "UTC", label: "(GMT+00:00) UTC", description: t("common.workspace_settings.timezone_options.utc"), icon: <Globe size={14} /> }]} value={timezone} />
              </div>
              <div className="space-y-2">
                <label className={SETTINGS_LABEL}>{t("common.workspace_settings.default_language")}</label>
                <Combobox className="bg-slate-50 dark:bg-slate-800/80 border-slate-200/70 dark:border-slate-700/70 h-11 rounded-xl normal-case tracking-normal text-xs font-medium px-4" onChange={setDefaultLanguage} options={[{ value: "vi", label: t("common.languages.vi"), icon: <Languages size={14} /> }, { value: "en", label: t("common.languages.en"), icon: <Languages size={14} /> }, { value: "zh", label: t("common.languages.zh"), icon: <Languages size={14} /> }]} value={defaultLanguage} />
              </div>
            </div>
          </div>
        ) : null}

        {activeTab === "appearance" ? (
          <div className="space-y-6">
            {/* Branding Inset Card */}
            <div className="bg-white dark:bg-slate-900/90 p-6 sm:p-8 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 shadow-2xs space-y-6">
              <div className="flex items-center gap-2.5 border-b border-slate-100 dark:border-slate-800/80 pb-4">
                <Building2 className="w-5 h-5 text-indigo-500" />
                <Typography className="text-base font-bold text-slate-800 dark:text-slate-100" variant="h4">
                  {t("common.workspace_settings.branding", { defaultValue: "Nhận diện thương hiệu" })}
                </Typography>
              </div>
              
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <label className={SETTINGS_LABEL}>{t("common.workspace_settings.workspace_name", { defaultValue: "Tên Workspace" })}</label>
                  <div className="relative">
                    <div className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400">
                      <Building2 size={16} />
                    </div>
                    <input
                      className="w-full h-11 pl-10 pr-4 bg-slate-50 dark:bg-slate-800/80 border border-slate-200/70 dark:border-slate-700/70 rounded-xl text-xs font-medium focus:border-primary-500 focus:ring-2 focus:ring-primary-500/20 transition-all text-slate-700 dark:text-slate-200 outline-none"
                      data-testid="workspace-name-input"
                      onChange={(e) => { setWsName(e.target.value); }}
                      placeholder={t("common.workspace_settings.workspace_name_placeholder", { defaultValue: "e.g., LeeAttend Corp" })}
                      type="text"
                      value={wsName}
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <label className={SETTINGS_LABEL}>{t("common.workspace_settings.tagline", { defaultValue: "Khẩu hiệu / Mô tả" })}</label>
                  <div className="relative">
                    <div className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400">
                      <Type size={16} />
                    </div>
                    <input
                      className="w-full h-11 pl-10 pr-4 bg-slate-50 dark:bg-slate-800/80 border border-slate-200/70 dark:border-slate-700/70 rounded-xl text-xs font-medium focus:border-primary-500 focus:ring-2 focus:ring-primary-500/20 transition-all text-slate-700 dark:text-slate-200 outline-none"
                      data-testid="workspace-tagline-input"
                      onChange={(e) => { setTagline(e.target.value); }}
                      placeholder={t("common.workspace_settings.tagline_placeholder", { defaultValue: "e.g., Human Resources Management" })}
                      type="text"
                      value={tagline}
                    />
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <label className={SETTINGS_LABEL}>{t("common.workspace_settings.workspace_logo", { defaultValue: "Logo Workspace" })}</label>
                  {logoUrl ? (
                    <div className="relative group rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-800/50 flex flex-col items-center justify-center p-4 h-32">
                      <div 
                        aria-label="Workspace Logo"
                        className="h-16 w-full bg-contain bg-no-repeat bg-center drop-shadow-2xs"
                        role="img"
                        style={{ backgroundImage: `url(${logoUrl})` }}
                      />
                      <div className="absolute inset-0 bg-white/80 dark:bg-slate-900/80 backdrop-blur-xs opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2 rounded-xl">
                        <Button className="p-2 bg-white dark:bg-slate-800 text-slate-600 dark:text-slate-300 rounded-lg shadow-2xs hover:text-primary-600 border border-slate-200 dark:border-slate-700 transition-colors" 
                          data-testid="edit-logo-btn"
                          onClick={() => {
                            const url = prompt(t("common.workspace_settings.enter_logo_url", { defaultValue: "Enter image URL:" }), logoUrl);
                            if (url !== null) setLogoUrl(url);
                          }}
                          title={t("common.edit", { defaultValue: "Edit" })}
                          variant="ghost"
                        >
                          <ImagePlus size={16} />
                        </Button>
                        <Button className="p-2 bg-white dark:bg-slate-800 text-rose-500 rounded-lg shadow-2xs hover:bg-rose-50 border border-slate-200 dark:border-slate-700 transition-colors" 
                          data-testid="delete-logo-btn"
                          onClick={() => { setLogoUrl(""); }}
                          title={t("common.delete", { defaultValue: "Delete" })}
                          variant="ghost"
                        >
                          <Trash2 size={16} />
                        </Button>
                      </div>
                    </div>
                  ) : (
                    <Button className="w-full h-32 rounded-xl border-2 border-dashed border-slate-200 dark:border-slate-700 hover:border-primary-400 dark:hover:border-primary-500 bg-slate-50 dark:bg-slate-800/30 hover:bg-primary-50/50 dark:hover:bg-primary-900/10 transition-all flex flex-col items-center justify-center gap-2 group" 
                      data-testid="upload-logo-btn"
                      onClick={() => {
                        const url = prompt(t("common.workspace_settings.enter_logo_url", { defaultValue: "Enter image URL:" }));
                        if (url) setLogoUrl(url);
                      }}
                      variant="ghost"
                    >
                      <div className="w-9 h-9 rounded-full bg-white dark:bg-slate-800 shadow-2xs flex items-center justify-center text-slate-400 group-hover:text-primary-500 transition-all">
                        <UploadCloud size={18} />
                      </div>
                      <div className="text-center">
                        <Typography className="text-xs font-semibold text-slate-600 dark:text-slate-300 group-hover:text-primary-600" variant="p">{t("common.workspace_settings.upload_logo", { defaultValue: "Nhấp để cài đặt logo URL" })}</Typography>
                      </div>
                    </Button>
                  )}
                </div>
              </div>
            </div>

            {/* Theme & Palette Inset Card */}
            <div className="bg-white dark:bg-slate-900/90 p-6 sm:p-8 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 shadow-2xs space-y-6">
              <div className="flex items-center gap-2.5 border-b border-slate-100 dark:border-slate-800/80 pb-4">
                <Palette className="w-5 h-5 text-pink-500" />
                <Typography className="text-base font-bold text-slate-800 dark:text-slate-100" variant="h4">
                  {t("common.workspace_settings.theme_config")}
                </Typography>
              </div>
              <div className="space-y-6">
                <div className="space-y-3">
                  <label className={SETTINGS_LABEL}>{t("common.workspace_settings.primary_color")}</label>
                  <div className="flex flex-wrap gap-3 items-center">
                    {[
                      { name: t("common.colors.emerald"), color: '#10b981' }, { name: t("common.colors.blue"), color: '#2563eb' },
                      { name: t("common.colors.emerald"), color: '#059669' }, { name: t("common.colors.amber"), color: '#d97706' }, { name: t("common.colors.rose"), color: '#e11d48' }, { name: t("common.colors.slate"), color: '#475569' },
                    ].map((preset) => (
                      <Button className={cn("w-10 h-10 rounded-xl transition-all border-2 flex items-center justify-center shadow-2xs group active:scale-95", themePrimaryColor === preset.color ? 'border-slate-900 dark:border-white scale-105' : 'border-transparent hover:scale-105')} key={preset.color} onClick={() => { setThemePrimaryColor(preset.color); }} style={{ backgroundColor: preset.color }} title={preset.name} variant="ghost">
                        {themePrimaryColor === preset.color && <div className="w-2.5 h-2.5 bg-white dark:bg-slate-900 rounded-full shadow-xs" />}
                      </Button>
                    ))}
                    <div className="h-8 w-px bg-slate-200 dark:bg-slate-700 mx-1" />
                    <div className="flex items-center gap-2 bg-slate-50 dark:bg-slate-800 px-3 py-1.5 rounded-xl border border-slate-200/80 dark:border-slate-700/80 shadow-2xs">
                      <input className="w-6 h-6 rounded-md cursor-pointer bg-transparent border-none p-0" onChange={(e) => { setThemePrimaryColor(e.target.value); }} type="color" value={themePrimaryColor} />
                      <input className="bg-transparent border-none text-xs font-mono font-semibold text-slate-600 dark:text-slate-300 w-20 outline-none" onChange={(e) => { setThemePrimaryColor(e.target.value); }} type="text" value={themePrimaryColor} />
                    </div>
                  </div>
                </div>

                {/* Dark Mode Preference */}
                <DarkModeSelector />
              </div>
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

  const options: { mode: ThemeMode; label: string; icon: React.ReactNode }[] = [
    { mode: "light", label: t("common.theme.light", { defaultValue: "Sáng" }), icon: <Sun size={15} /> },
    { mode: "dark", label: t("common.theme.dark", { defaultValue: "Tối" }), icon: <Moon size={15} /> },
    { mode: "system", label: t("common.theme.system", { defaultValue: "Hệ thống" }), icon: <Monitor size={15} /> },
  ];

  return (
    <div className="space-y-2 pt-2 border-t border-slate-100 dark:border-slate-800/80">
      <label className={SETTINGS_LABEL}>{t("common.workspace_settings.dark_mode_preference", { defaultValue: "Chế độ hiển thị" })}</label>
      <div className="flex gap-2">
        {options.map((opt) => (
          <Button
            className={cn(
              "flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold transition-all shadow-2xs border",
              mode === opt.mode
                ? "bg-white dark:bg-slate-800 border-primary-500 text-primary-600 dark:text-primary-400 font-bold"
                : "bg-slate-50 dark:bg-slate-900 border-slate-200/70 dark:border-slate-800 text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800/50"
            )}
            key={opt.mode}
            onClick={() => setMode(opt.mode)}
            variant="ghost"
          >
            {opt.icon}
            <span>{opt.label}</span>
          </Button>
        ))}
      </div>
    </div>
  );
}
