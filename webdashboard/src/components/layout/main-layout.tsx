import { Outlet, useLocation } from "@tanstack/react-router";
import { Building2 } from "lucide-react";
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { useI18nSync } from "@/hooks/use-i18n-sync";
import { useTheme } from "@/hooks/use-theme";
import { useWorkspaces } from "@/hooks/use-workspace";
import { getCurrentWorkspaceId } from "@/lib/api";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

import Modal from "../ui/modal";
import Navbar from "./navbar";
import Sidebar from "./sidebar";
import WorkspaceSwitcher from "./workspace-switcher";



// Stable module-level arrays for sidebar section matching
const ORG_PATHS: string[] = [ROUTES.PEOPLE, ROUTES.EMPLOYEES, ROUTES.DEPARTMENTS];
const TIME_PATHS: string[] = [ROUTES.ATTENDANCE_LOGS, ROUTES.LEAVE_REQUESTS, ROUTES.ATTENDANCE_ADJUSTMENTS, ROUTES.REPORTS, ROUTES.LEAVE_BALANCES, ROUTES.SHIFT_SWAPS];
const PAYROLL_PATHS: string[] = [ROUTES.SHIFTS, ROUTES.PAYROLL];

export default function MainLayout() {
  const { t } = useTranslation();
  useTheme();
  useI18nSync();
  const location = useLocation();

  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [showSwitcher, setShowSwitcher] = useState(false);
  const [isCollapsed, setIsCollapsed] = useState(() => {
    const saved = localStorage.getItem("sidebar-collapsed");
    return saved === "true";
  });

  const getInitialOpenSections = () => {
    const path = location.pathname;
    return {
      org: ORG_PATHS.includes(path),
      time: TIME_PATHS.includes(path) || path === ROUTES.HOME,
      payroll: PAYROLL_PATHS.includes(path),
      system: path.startsWith(ROUTES.SETTINGS.ROOT),
      ess: path.startsWith("/ess/"),
    };
  };

  const [openSections, setOpenSections] = useState<Record<string, boolean>>(getInitialOpenSections());

  const { workspaces } = useWorkspaces();
  const currentWsId = getCurrentWorkspaceId();
  const currentWorkspace = workspaces.find(ws => ws.id === currentWsId);

  const toggleSection = (section: string) => {
    setOpenSections(previous => ({
      org: false,
      time: false,
      payroll: false,
      system: false,
      ess: false,
      [section]: !previous[section]
    }));
  };

  const toggleCollapsed = () => {
    setIsCollapsed(previous => {
      const newValue = !previous;
      localStorage.setItem("sidebar-collapsed", String(newValue));
      return newValue;
    });
  };

  useEffect(() => {
    const path = location.pathname;
    const isOrg = ORG_PATHS.includes(path);
    const isTime = TIME_PATHS.includes(path);
    const isPayroll = PAYROLL_PATHS.includes(path);
    const isSystem = path.startsWith(ROUTES.SETTINGS.ROOT);
    const isEss = path.startsWith("/ess/");

    const timer = setTimeout(() => {
      setOpenSections(previous => {
        const next = { ...previous };
        let changed = false;
        if (isOrg && !previous.org) { next.org = true; changed = true; }
        if (isTime && !previous.time) { next.time = true; changed = true; }
        if (isPayroll && !previous.payroll) { next.payroll = true; changed = true; }
        if (isSystem && !previous.system) { next.system = true; changed = true; }
        if (isEss && !previous.ess) { next.ess = true; changed = true; }
        return changed ? next : previous;
      });
      setIsSidebarOpen(false);
    }, 0);

    return () => { clearTimeout(timer); };
  }, [location.pathname]);

  return (
    <div className="flex min-h-screen bg-background text-foreground font-sans relative overflow-x-hidden">
      {!!isSidebarOpen && (
        <div
          className="fixed inset-0 bg-slate-900/50 dark:bg-slate-950/60 backdrop-blur-sm z-40 lg:hidden transition-opacity duration-500"
          onClick={() => { setIsSidebarOpen(false); }}
          onKeyDown={(e) => {
            if (e.key === 'Escape' || e.key === 'Enter' || e.key === ' ') {
              setIsSidebarOpen(false);
            }
          }}
          role="presentation"
        />
      )}

      <Sidebar
        isCollapsed={isCollapsed}
        isOpen={isSidebarOpen}
        onClose={() => { setIsSidebarOpen(false); }}
        onShowSwitcher={() => { setShowSwitcher(true); }}
        openSections={openSections}
        toggleSection={toggleSection}
      />

      <div className={cn(
        "flex-1 flex flex-col min-w-0 h-screen overflow-hidden relative transition-all duration-300",
        isCollapsed ? "lg:ml-20" : "lg:ml-72"
      )}>
        <Navbar
          currentWorkspaceName={currentWorkspace?.name || "Workspace"}
          isCollapsed={isCollapsed}
          onOpenSidebar={() => { setIsSidebarOpen(true); }}
          onToggleCollapsed={toggleCollapsed}
        />

        <main className="flex-1 overflow-y-auto p-4 md:p-8 lg:p-10 relative scroll-smooth">
          <div className="max-w-7xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500 fill-mode-both" key={location.pathname}>
            <Outlet />
          </div>
        </main>
      </div>

      <Modal
        icon={<Building2 className="w-6 h-6" />}
        isOpen={showSwitcher}
        maxWidth="lg"
        onClose={() => { setShowSwitcher(false); }}
        subtitle={t("common.office_branch")}
        title={t("common.switch_workspace")}
      >
        <div className="space-y-8">
          <p className="text-sm text-slate-500 dark:text-slate-400 font-medium px-1">{t("common.select_workspace_desc")}</p>
          <WorkspaceSwitcher />
          <div className="pt-6 border-t border-slate-50 dark:border-slate-800 flex justify-end">
            <button
              className="px-8 py-3 bg-slate-100 dark:bg-slate-800 text-foreground/70 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-200 dark:hover:bg-slate-700 transition-all"
              onClick={() => { setShowSwitcher(false); }}
            >
              {t("common.close")}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
