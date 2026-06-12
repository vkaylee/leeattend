import { Link, Outlet, useLocation } from "@tanstack/react-router";
import {
  ChevronRight,
  ShieldCheck,
  Settings
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { Typography } from "@/components/ui/typography";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

import { NavGroup } from "./components/nav-group";
import { SettingItem, useSettingsNav } from "./hooks/use-settings-nav";

interface SettingsSidebarProps {
  accessGroup: SettingItem[];
  dangerItem: SettingItem | null;
  isActive: (path: string) => boolean;
  isCollapsed: boolean;
  onToggleCollapse: () => void;
  operationsGroup: SettingItem[];
  systemGroup: SettingItem[];
  workspaceGroup: SettingItem[];
}

function SettingsSidebar({
  accessGroup,
  dangerItem,
  isActive,
  isCollapsed,
  onToggleCollapse,
  operationsGroup,
  systemGroup,
  workspaceGroup,
}: SettingsSidebarProps) {
  const { t } = useTranslation();
  return (
    <aside className={cn(
      "w-full flex-shrink-0 transition-all duration-300",
      isCollapsed ? "lg:w-20" : "lg:w-72"
    )}>
      <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-4 shadow-sm sticky top-24 max-h-[calc(100vh-8rem)] overflow-y-auto custom-scrollbar">
        <div className={cn(
          "px-4 py-3 mb-4 flex items-center justify-between gap-3",
          isCollapsed && "flex-col justify-center px-0 gap-2"
        )}>
          <div className="flex items-center gap-3">
            <div className="p-2 bg-blue-50 dark:bg-blue-900/30 rounded-xl">
              <Settings className="w-5 h-5 text-blue-600 dark:text-blue-400" />
            </div>
            {!isCollapsed && (
              <div>
                <Typography className="text-sm font-bold text-slate-800 dark:text-slate-200" variant="h2">
                  {t("common.settings.title")}
                </Typography>
                <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5" variant="p">
                  {t("settings.layout.control_panel")}
                </Typography>
              </div>
            )}
          </div>

          <button
            onClick={onToggleCollapse}
            className={cn(
              "p-1.5 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors",
              isCollapsed ? "mt-1" : "ml-auto"
            )}
            title={isCollapsed ? t("settings.expand_sidebar", "Expand sidebar") : t("settings.collapse_sidebar", "Collapse sidebar")}
          >
            <ChevronRight className={cn("w-4 h-4 transition-transform duration-300", !isCollapsed && "rotate-180")} />
          </button>
        </div>
        
        <nav>
          {workspaceGroup.length > 0 ? (
            <NavGroup isActive={isActive} isCollapsed={isCollapsed} items={workspaceGroup} title={t("settings.layout.workspace")} />
          ) : null}
          {accessGroup.length > 0 ? (
            <NavGroup isActive={isActive} isCollapsed={isCollapsed} items={accessGroup} title={t("settings.layout.access")} />
          ) : null}
          {operationsGroup.length > 0 ? (
            <NavGroup isActive={isActive} isCollapsed={isCollapsed} items={operationsGroup} title={t("settings.layout.operations")} />
          ) : null}
          {systemGroup.length > 0 ? (
            <NavGroup isActive={isActive} isCollapsed={isCollapsed} items={systemGroup} title={t("settings.layout.security_system")} />
          ) : null}
          
          {dangerItem ? (
            <DangerItemLink dangerItem={dangerItem} isActive={isActive} isCollapsed={isCollapsed} />
          ) : null}
        </nav>

        {!isCollapsed ? (
          <div className="mt-8 pt-6 border-t border-slate-50 dark:border-slate-800 px-4 pb-2">
            <div className="flex items-center gap-2 text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
              <ShieldCheck className="w-3 h-3" />
              <span>{t("settings.admin_only")}</span>
            </div>
          </div>
        ) : (
          <div className="mt-4 pt-4 border-t border-slate-50 dark:border-slate-800 flex justify-center text-slate-300 dark:text-slate-600 pb-2" title={t("settings.admin_only")}>
            <ShieldCheck className="w-5 h-5" />
          </div>
        )}
      </div>
    </aside>
  );
}

interface DangerItemLinkProps {
  dangerItem: SettingItem;
  isActive: (path: string) => boolean;
  isCollapsed: boolean;
}

function DangerItemLink({ dangerItem, isActive, isCollapsed }: DangerItemLinkProps) {
  const active = isActive(dangerItem.path);
  return (
    <div className={cn("pt-4 mt-4 border-t border-slate-100 dark:border-slate-800", isCollapsed && "pt-2 mt-2")}>
      <Link
        className={cn(
          "flex items-center gap-3 rounded-2xl transition-all duration-300 group",
          isCollapsed ? "justify-center p-2" : "px-4 py-3 items-start",
          active
            ? "bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 shadow-sm border border-rose-100 dark:border-rose-900/50"
            : "text-slate-500 dark:text-slate-400 hover:bg-rose-50/50 dark:hover:bg-rose-900/10 hover:text-rose-600 dark:hover:text-rose-400 border border-transparent"
        )}
        to={dangerItem.path}
        title={isCollapsed ? dangerItem.label : undefined}
      >
        <div
          className={cn(
            "p-2 rounded-xl transition-colors",
            active
              ? "bg-white dark:bg-slate-900 shadow-sm"
              : "bg-slate-50 dark:bg-slate-800 group-hover:bg-rose-100 dark:group-hover:bg-rose-900/30 text-rose-500"
          )}
        >
          {dangerItem.icon}
        </div>
        {!isCollapsed && (
          <div className="flex flex-col min-w-0">
            <span className="font-bold text-sm leading-tight">{dangerItem.label}</span>
            <span className="text-[10px] text-slate-500 dark:text-slate-400 font-medium leading-tight hidden xl:block mt-0.5 group-hover:text-rose-500/70">
              {dangerItem.description}
            </span>
          </div>
        )}
        {!isCollapsed && active ? <ChevronRight className="w-4 h-4 ml-auto text-rose-400" /> : null}
      </Link>
    </div>
  );
}

export default function SettingsLayout() {
  const location = useLocation();
  const {
    accessGroup,
    dangerItem,
    operationsGroup,
    systemGroup,
    workspaceGroup,
  } = useSettingsNav();

  const [isCollapsed, setIsCollapsed] = React.useState(false);

  React.useEffect(() => {
    const collapsed = localStorage.getItem("settings_sidebar_collapsed") === "true";
    if (collapsed) {
      setIsCollapsed(true);
    }
  }, []);

  const toggleCollapse = () => {
    setIsCollapsed((prev) => {
      const next = !prev;
      localStorage.setItem("settings_sidebar_collapsed", String(next));
      return next;
    });
  };

  const isActive = (path: string) => {
    if (path === ROUTES.SETTINGS.GENERAL && location.pathname === ROUTES.SETTINGS.ROOT) return true;
    return location.pathname.startsWith(path);
  };

  const isWorkflowEditor = location.pathname.includes("/workspace/settings/workflows/new") ||
    /\/workspace\/settings\/workflows\/[^/]+\/edit/.test(location.pathname);

  return (
    <div className={isWorkflowEditor ? "min-h-[calc(100vh-10rem)]" : "flex flex-col lg:flex-row gap-8 min-h-[calc(100vh-10rem)]"}>
      {/* Sub-navigation Sidebar */}
      {!isWorkflowEditor ? (
        <SettingsSidebar
          accessGroup={accessGroup}
          dangerItem={dangerItem}
          isActive={isActive}
          isCollapsed={isCollapsed}
          onToggleCollapse={toggleCollapse}
          operationsGroup={operationsGroup}
          systemGroup={systemGroup}
          workspaceGroup={workspaceGroup}
        />
      ) : null}

      {/* Content Area */}
      <main className="flex-1 min-w-0">
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500 bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm p-8 lg:p-10 min-h-full">
          <ErrorBoundary key={location.pathname}>
            <Outlet />
          </ErrorBoundary>
        </div>
      </main>
    </div>
  );
}
