import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { Sliders, ChevronDown } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import Button from "@/components/ui/button";
import { ErrorBoundary } from "@/components/common/error-boundary";
import { fetchMyPermissions } from "@/lib/api";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";
import { SettingsSidebar } from "@/pages/settings/components/settings-sidebar";

const SETTINGS_PERMISSIONS = [
  "organization:settings:read",
  "members:roles:assign",
  "members:roles:manage",
] as const;

export const Route = createFileRoute(
  "/_authenticated/_organization/settings"
)({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!SETTINGS_PERMISSIONS.some((permission) => permissions.includes(permission))) {
        throw redirect({ to: ROUTES.HOME });
      }
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: ROUTES.HOME });
    }
  },
  component: SettingsPageLayout,
});

function SettingsPageLayout() {
  const { t } = useTranslation();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  return (
    <div className="flex-1 flex flex-col h-full min-h-0 overflow-hidden">
      {/* Mobile Selector Bar */}
      <div className="flex md:hidden items-center justify-between p-3 border-b border-border bg-card shrink-0">
        <Button
          aria-expanded={mobileMenuOpen}
          aria-label={mobileMenuOpen ? t("common.close", "Đóng") : t("settings.categories_menu", "Danh mục")}
          className="flex items-center justify-between w-full h-auto p-3 rounded-2xl bg-white dark:bg-slate-900 border border-slate-200/80 dark:border-slate-800/80 shadow-2xs text-xs font-semibold text-slate-800 dark:text-slate-100 hover:bg-slate-50 dark:hover:bg-slate-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 transition-all duration-200 normal-case tracking-normal"
          data-testid="settings-mobile-menu-toggle"
          onClick={() => setMobileMenuOpen((prev) => !prev)}
          type="button"
          variant="outline"
        >
          <div className="flex items-center gap-2.5 min-w-0">
            <div className="w-7 h-7 rounded-lg bg-primary-50 dark:bg-primary-950/70 border border-primary-200/60 dark:border-primary-800/60 flex items-center justify-center shrink-0 text-primary-600 dark:text-primary-400">
              <Sliders className="w-3.5 h-3.5" />
            </div>
            <span className="truncate">{t("common.settings.title", "Cài đặt")}</span>
          </div>
          <span className="flex items-center gap-1.5 text-xs font-semibold px-2.5 py-1 rounded-lg bg-slate-100 dark:bg-slate-800 border border-slate-200/60 dark:border-slate-700/60 text-slate-700 dark:text-slate-300">
            {mobileMenuOpen ? t("common.close", "Đóng") : t("settings.categories_menu", "Danh mục")}
            <ChevronDown className={cn("w-3.5 h-3.5 text-slate-400 transition-transform duration-200", mobileMenuOpen && "rotate-180")} />
          </span>
        </Button>
      </div>

      <div className="flex-1 flex flex-col md:flex-row min-h-0 h-full overflow-hidden">
        {/* Left Column: Settings Sidebar */}
        <div
          className={cn(
            "w-full md:w-72 lg:w-80 shrink-0 min-h-0 h-full border-r border-border bg-card/40",
            !mobileMenuOpen && "hidden md:block"
          )}
          data-testid="settings-sidebar-panel"
        >
          <SettingsSidebar onItemClick={() => setMobileMenuOpen(false)} />
        </div>

        {/* Right Column: Active Setting Detail Pane */}
        <div className={cn("flex-1 min-w-0 h-full overflow-y-auto bg-background p-6 md:p-8 lg:p-10", mobileMenuOpen && "hidden md:block")}>
          <ErrorBoundary>
            <Outlet />
          </ErrorBoundary>
        </div>
      </div>
    </div>
  );
}
