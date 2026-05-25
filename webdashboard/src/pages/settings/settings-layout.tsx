import { Link, Outlet, useLocation } from "@tanstack/react-router";
import {
  ChevronRight,
  ShieldCheck,
  Settings
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { ROUTES } from "@/lib/routes";

import { NavGroup } from "./components/nav-group";
import { useSettingsNav } from "./hooks/use-settings-nav";

export default function SettingsLayout() {
  const { t } = useTranslation();
  const location = useLocation();
  const {
    accessGroup,
    dangerItem,
    operationsGroup,
    systemGroup,
    workspaceGroup,
  } = useSettingsNav();

  const isActive = (path: string) => {
    if (path === ROUTES.SETTINGS.OVERVIEW && location.pathname === ROUTES.SETTINGS.ROOT) return true;
    return location.pathname.startsWith(path);
  };

  return (
    <div className="flex flex-col lg:flex-row gap-8 min-h-[calc(100vh-10rem)]">
      {/* Sub-navigation Sidebar */}
      <aside className="w-full lg:w-72 flex-shrink-0">
        <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-4 shadow-sm sticky top-24 max-h-[calc(100vh-8rem)] overflow-y-auto custom-scrollbar">
          <div className="px-4 py-3 mb-4 flex items-center gap-3">
            <div className="p-2 bg-blue-50 dark:bg-blue-900/30 rounded-xl">
              <Settings className="w-5 h-5 text-blue-600 dark:text-blue-400" />
            </div>
            <div>
              <Typography className="text-sm font-bold text-slate-800 dark:text-slate-200" variant="h2">
                {t("common.settings.title")}
              </Typography>
              <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5" variant="p">
                {t("settings.layout.control_panel")}
              </Typography>
            </div>
          </div>
          
          <nav>
            {workspaceGroup.length > 0 ? (
              <NavGroup isActive={isActive} items={workspaceGroup} title={t("settings.layout.workspace")} />
            ) : null}
            {accessGroup.length > 0 ? (
              <NavGroup isActive={isActive} items={accessGroup} title={t("settings.layout.access")} />
            ) : null}
            {operationsGroup.length > 0 ? (
              <NavGroup isActive={isActive} items={operationsGroup} title={t("settings.layout.operations")} />
            ) : null}
            {systemGroup.length > 0 ? (
              <NavGroup isActive={isActive} items={systemGroup} title={t("settings.layout.security_system")} />
            ) : null}
            
            {dangerItem ? (
              <div className="pt-4 mt-4 border-t border-slate-100 dark:border-slate-800">
                <Link
                  className={`flex items-start gap-3 px-4 py-3 rounded-2xl transition-all duration-300 group ${
                    isActive(dangerItem.path)
                      ? "bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 shadow-sm border border-rose-100 dark:border-rose-900/50"
                      : "text-slate-500 dark:text-slate-400 hover:bg-rose-50/50 dark:hover:bg-rose-900/10 hover:text-rose-600 dark:hover:text-rose-400 border border-transparent"
                  }`}
                  to={dangerItem.path}
                >
                  <div
                    className={`p-2 rounded-xl transition-colors ${
                      isActive(dangerItem.path)
                        ? "bg-white dark:bg-slate-900 shadow-sm"
                        : "bg-slate-50 dark:bg-slate-800 group-hover:bg-rose-100 dark:group-hover:bg-rose-900/30 text-rose-500"
                    }`}
                  >
                    {dangerItem.icon}
                  </div>
                  <div className="flex flex-col min-w-0">
                    <span className="font-bold text-sm leading-tight">{dangerItem.label}</span>
                    <span className="text-[10px] text-slate-500 dark:text-slate-400 font-medium leading-tight hidden xl:block mt-0.5 group-hover:text-rose-500/70">
                      {dangerItem.description}
                    </span>
                  </div>
                  {isActive(dangerItem.path) ? (
                    <ChevronRight className="w-4 h-4 ml-auto text-rose-400" />
                  ) : null}
                </Link>
              </div>
            ) : null}
          </nav>

          <div className="mt-8 pt-6 border-t border-slate-50 dark:border-slate-800 px-4 pb-2">
            <div className="flex items-center gap-2 text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
              <ShieldCheck className="w-3 h-3" />
              <span>{t("settings.admin_only")}</span>
            </div>
          </div>
        </div>
      </aside>

      {/* Content Area */}
      <main className="flex-1 min-w-0">
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500 bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm p-8 lg:p-10 min-h-full">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
