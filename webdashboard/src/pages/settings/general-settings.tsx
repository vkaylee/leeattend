import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Settings2, Save, Clock, Palette, Globe, Calendar, Languages, Sun, Moon, Monitor, Type, Building2 } from "lucide-react";
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import type { ThemeMode } from "@/hooks/use-dark-mode";

import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { SettingsHeader } from "@/components/ui/settings/settings-header";
const SETTINGS_LABEL = "text-xs font-semibold text-slate-700 dark:text-slate-300 ml-0.5";
import { Skeleton } from "@/components/ui/skeleton";
import { useToast } from "@/components/ui/toast";
import { Typography } from "@/components/ui/typography";
import { useDarkModeContext } from "@/context/use-dark-mode-context";
import { useTheme } from "@/hooks/use-theme";
import { fetchOrganization, updateOrganization, Organization } from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";
import { cn } from "@/lib/utils";

import { OrganizationLogoField } from "./components/organization-logo-field";
import { ThemeColorPicker } from "./components/theme-color-picker";


type TabType = "localization" | "appearance";

export default function GeneralSettings() {


  const { data: organization, isLoading: isLoadingOrganization } = useQuery({
    queryKey: createQueryKey("organization"),
    queryFn: () => {

      return fetchOrganization();
    },
  });

  if (isLoadingOrganization) {
    return (
      <div className="space-y-6">
        <Skeleton className="h-40 w-full rounded-2xl" />
        <Skeleton className="h-80 w-full rounded-2xl" />
      </div>
    );
  }

  if (!organization) return null;

  return <GeneralSettingsContent organization={organization} />;
}

// eslint-disable-next-line max-lines-per-function
function GeneralSettingsContent({ organization }: { organization: Organization }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const toast = useToast();
  const { setPreviewTheme } = useTheme();

  const [activeTab, setActiveTab] = useState<TabType>("localization");
  const [dateFormat, setDateFormat] = useState(organization.date_format);
  const [timeFormat, setTimeFormat] = useState(organization.time_format);
  const [timezone, setTimezone] = useState(organization.timezone);
  const [themePrimaryColor, setThemePrimaryColor] = useState(organization.theme_primary_color);
  const [defaultLanguage, setDefaultLanguage] = useState(organization.default_language);
  const [organizationName, setOrganizationName] = useState(organization.name);
  const [logoUrl, setLogoUrl] = useState(organization.logo_url ?? "");
  const [tagline, setTagline] = useState(organization.tagline ?? "");

  useEffect(() => {
    setPreviewTheme(themePrimaryColor);
    return () => { setPreviewTheme(null); };
  }, [themePrimaryColor, setPreviewTheme]);

  const updateOrganizationMutation = useMutation({
    mutationFn: (data: Partial<Organization>) => updateOrganization(data),
    onMutate: async (data) => {
      const queryKey = ["organization"];
      await queryClient.cancelQueries({ queryKey });
      const previousOrganization = queryClient.getQueryData<Organization>(queryKey);
      queryClient.setQueryData<Organization>(queryKey, (current) => current ? { ...current, ...data } : current);
      return { previousOrganization };
    },
    onError: (_error, _data, context) => {
      queryClient.setQueryData(["organization"], context?.previousOrganization);
    },
    onSuccess: (updatedOrganization, data) => {
      toast.success(t("common.success"));
      queryClient.setQueryData<Organization>(["organization"], (current) => current ? { ...current, ...data, ...updatedOrganization } : updatedOrganization);
      void queryClient.invalidateQueries({ queryKey: createQueryKey("organization") });
    },
  });

  function handleSaveConfig() {
    updateOrganizationMutation.mutate({
      date_format: dateFormat,
      time_format: timeFormat,
      timezone,
      theme_primary_color: themePrimaryColor,
      name: organizationName,
      default_language: defaultLanguage,
      logo_url: logoUrl || undefined,
      tagline: tagline || undefined,
    });
  }

  return (
    <div className="space-y-6 animate-in fade-in duration-300">
      {/* Apple HIG Header via SettingsHeader */}
      <SettingsHeader
        actions={
          <Button
            className="rounded-xl px-5 py-2 text-xs font-semibold shadow-xs normal-case tracking-normal"
            data-testid="save-organization-config-btn"
            icon={<Save size={15} />}
            loading={updateOrganizationMutation.isPending}
            onClick={handleSaveConfig}
            variant="primary"
          >
            {t("common.organization_settings.save_config", { defaultValue: "Lưu thay đổi" })}
          </Button>
        }
        description={t("common.organization_settings.description")}
        icon={<Settings2 size={20} />}
        title={
          <Typography className="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100" data-testid="general-settings-title" variant="h1">
            {t("common.settings.categories.general")}
          </Typography>
        }
      />

      {/* Apple HIG Segmented Control with mobile horizontal scroll */}
      <div className="flex max-w-full items-center pb-1">
        <div className="grid w-full grid-cols-1 gap-1 rounded-xl border border-slate-200/60 bg-slate-100/90 p-1 dark:border-slate-700/60 dark:bg-slate-800/80 sm:inline-flex sm:w-auto sm:grid-cols-none" role="tablist">
          <Button
            aria-selected={activeTab === "localization"}
            className={cn(
              "flex min-w-0 items-center justify-center gap-2 rounded-lg px-4 py-2 text-xs font-semibold transition-all duration-150 select-none sm:whitespace-nowrap",
              activeTab === "localization"
                ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
            )}
            data-testid="tab-localization"
            onClick={() => { setActiveTab("localization"); }}
            role="tab"
            size="sm"
            type="button"
            variant="ghost"
          >
            <Globe size={14} />
            <span>{t("common.settings.tabs.localization")}</span>
          </Button>
          <Button
            aria-selected={activeTab === "appearance"}
            className={cn(
              "flex min-w-0 items-center justify-center gap-2 rounded-lg px-4 py-2 text-xs font-semibold transition-all duration-150 select-none sm:whitespace-nowrap",
              activeTab === "appearance"
                ? "bg-white dark:bg-slate-700 text-slate-900 dark:text-slate-100 shadow-2xs font-bold"
                : "text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200"
            )}
            data-testid="tab-appearance"
            onClick={() => { setActiveTab("appearance"); }}
            role="tab"
            size="sm"
            type="button"
            variant="ghost"
          >
            <Palette size={14} />
            <span>{t("common.settings.tabs.appearance")}</span>
          </Button>
        </div>
      </div>

      <div className="animate-in fade-in duration-300">
        {activeTab === "localization" ? (
          <div className="bg-white dark:bg-slate-900/90 p-6 sm:p-8 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 shadow-2xs space-y-6">
            <div className="flex items-center gap-2.5 border-b border-slate-100 dark:border-slate-800/80 pb-4">
              <Globe className="w-5 h-5 text-primary-500" />
              <Typography className="text-base font-bold text-slate-800 dark:text-slate-100" variant="h4">
                {t("common.organization_settings.general_config")}
              </Typography>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label className={SETTINGS_LABEL}>{t("common.organization_settings.date_format")}</label>
                <Combobox className="bg-slate-50 dark:bg-slate-800/80 border-slate-200/70 dark:border-slate-700/70 h-11 rounded-xl normal-case tracking-normal text-xs font-medium px-4" onChange={setDateFormat} options={[{ value: "DD/MM/YYYY", label: "DD/MM/YYYY", description: t("common.organization_settings.date_format_options.vn"), icon: <Calendar size={14} /> }, { value: "MM/DD/YYYY", label: "MM/DD/YYYY", description: t("common.organization_settings.date_format_options.us"), icon: <Calendar size={14} /> }, { value: "YYYY-MM-DD", label: "YYYY-MM-DD", description: t("common.organization_settings.date_format_options.iso"), icon: <Calendar size={14} /> }]} value={dateFormat} />
              </div>
              <div className="space-y-2">
                <label className={SETTINGS_LABEL}>{t("common.organization_settings.time_format")}</label>
                <Combobox className="bg-slate-50 dark:bg-slate-800/80 border-slate-200/70 dark:border-slate-700/70 h-11 rounded-xl normal-case tracking-normal text-xs font-medium px-4" onChange={setTimeFormat} options={[{ value: "24h", label: t("common.organization_settings.time_format_options.24h"), description: t("common.organization_settings.time_format_options.example_24h"), icon: <Clock size={14} /> }, { value: "12h", label: t("common.organization_settings.time_format_options.12h"), description: t("common.organization_settings.time_format_options.example_12h"), icon: <Clock size={14} /> }]} value={timeFormat} />
              </div>
              <div className="space-y-2">
                <label className={SETTINGS_LABEL}>{t("common.organization_settings.timezone")}</label>
                <Combobox className="bg-slate-50 dark:bg-slate-800/80 border-slate-200/70 dark:border-slate-700/70 h-11 rounded-xl normal-case tracking-normal text-xs font-medium px-4" data-testid="timezone-select" onChange={setTimezone} options={[{ value: "Asia/Ho_Chi_Minh", label: "(GMT+07:00) Asia/Ho Chi Minh", description: t("common.organization_settings.timezone_options.vn"), icon: <Globe size={14} /> }, { value: "Asia/Bangkok", label: "(GMT+07:00) Asia/Bangkok", description: t("common.organization_settings.timezone_options.th"), icon: <Globe size={14} /> }, { value: "UTC", label: "(GMT+00:00) UTC", description: t("common.organization_settings.timezone_options.utc"), icon: <Globe size={14} /> }]} value={timezone} />
              </div>
              <div className="space-y-2">
                <label className={SETTINGS_LABEL}>{t("common.organization_settings.default_language")}</label>
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
                  {t("common.organization_settings.branding", { defaultValue: "Nhận diện thương hiệu" })}
                </Typography>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <label className={SETTINGS_LABEL}>{t("common.organization_settings.organization_name", { defaultValue: "Tên tổ chức" })}</label>
                  <div className="relative">
                    <div className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400">
                      <Building2 size={16} />
                    </div>
                    <input
                      className="w-full h-11 pl-10 pr-4 bg-slate-50 dark:bg-slate-800/80 border border-slate-200/70 dark:border-slate-700/70 rounded-xl text-xs font-medium focus:border-primary-500 focus:ring-2 focus:ring-primary-500/20 transition-all text-slate-700 dark:text-slate-200 outline-none"
                      data-testid="organization-name-input"
                      onChange={(e) => { setOrganizationName(e.target.value); }}
                      placeholder={t("common.organization_settings.organization_name_placeholder", { defaultValue: "e.g., LeeAttend Corp" })}
                      type="text"
                      value={organizationName}
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <label className={SETTINGS_LABEL}>{t("common.organization_settings.tagline", { defaultValue: "Khẩu hiệu / Mô tả" })}</label>
                  <div className="relative">
                    <div className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400">
                      <Type size={16} />
                    </div>
                    <input
                      className="w-full h-11 pl-10 pr-4 bg-slate-50 dark:bg-slate-800/80 border border-slate-200/70 dark:border-slate-700/70 rounded-xl text-xs font-medium focus:border-primary-500 focus:ring-2 focus:ring-primary-500/20 transition-all text-slate-700 dark:text-slate-200 outline-none"
                      data-testid="organization-tagline-input"
                      onChange={(e) => { setTagline(e.target.value); }}
                      placeholder={t("common.organization_settings.tagline_placeholder", { defaultValue: "e.g., Human Resources Management" })}
                      type="text"
                      value={tagline}
                    />
                  </div>
                </div>
              </div>

              <OrganizationLogoField
                labelClassName={SETTINGS_LABEL}
                logoUrl={logoUrl}
                onLogoChange={setLogoUrl}
              />
            </div>

            {/* Theme & Palette Inset Card */}
            <div className="bg-white dark:bg-slate-900/90 p-6 sm:p-8 rounded-2xl border border-slate-200/80 dark:border-slate-800/80 shadow-2xs space-y-6">
              <div className="flex items-center gap-2.5 border-b border-slate-100 dark:border-slate-800/80 pb-4">
                <Palette className="w-5 h-5 text-pink-500" />
                <Typography className="text-base font-bold text-slate-800 dark:text-slate-100" variant="h4">
                  {t("common.organization_settings.theme_config")}
                </Typography>
              </div>
              <div className="space-y-6">
                <ThemeColorPicker
                  labelClassName={SETTINGS_LABEL}
                  onThemeColorChange={setThemePrimaryColor}
                  themePrimaryColor={themePrimaryColor}
                />

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
      <label className={SETTINGS_LABEL}>{t("common.organization_settings.dark_mode_preference", { defaultValue: "Chế độ hiển thị" })}</label>
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
            onClick={() => { setMode(opt.mode); }}
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
