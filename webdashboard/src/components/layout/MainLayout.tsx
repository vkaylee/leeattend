import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Outlet, useLocation } from "@tanstack/react-router";
import { Building2, X } from "lucide-react";
import { useWorkspaces } from "@/hooks/use-workspace";
import { getCurrentWorkspaceId } from "@/lib/api";
import { useTheme } from "@/hooks/use-theme";
import { useI18nSync } from "@/hooks/use-i18n-sync";
import Sidebar from "./Sidebar";
import Navbar from "./Navbar";
import WorkspaceSwitcher from "./WorkspaceSwitcher";
import Modal from "../ui/Modal";
import { cn } from "@/lib/utils";
import { ROUTES } from "@/lib/routes";

// Stable module-level arrays for sidebar section matching
const ORG_PATHS: string[] = [ROUTES.MEMBERS, ROUTES.EMPLOYEES, ROUTES.DEPARTMENTS];
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
    setOpenSections(prev => ({
      org: false,
      time: false,
      payroll: false,
      system: false,
      ess: false,
      [section]: !prev[section]
    }));
  };

  const toggleCollapsed = () => {
    setIsCollapsed(prev => {
      const newValue = !prev;
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
      setOpenSections(prev => {
        const next = { ...prev };
        let changed = false;
        if (isOrg && !prev.org) { next.org = true; changed = true; }
        if (isTime && !prev.time) { next.time = true; changed = true; }
        if (isPayroll && !prev.payroll) { next.payroll = true; changed = true; }
        if (isSystem && !prev.system) { next.system = true; changed = true; }
        if (isEss && !prev.ess) { next.ess = true; changed = true; }
        return changed ? next : prev;
      });
      setIsSidebarOpen(false);
    }, 0);

    return () => clearTimeout(timer);
  }, [location.pathname]);

  return (
    <div className="flex min-h-screen bg-background text-foreground font-sans relative overflow-x-hidden">
      {isSidebarOpen && (
        <div
          className="fixed inset-0 bg-slate-900/50 dark:bg-slate-950/60 backdrop-blur-sm z-40 lg:hidden transition-opacity duration-500"
          onClick={() => setIsSidebarOpen(false)}
        />
      )}

      <Sidebar
        isOpen={isSidebarOpen}
        onClose={() => setIsSidebarOpen(false)}
        openSections={openSections}
        toggleSection={toggleSection}
        onShowSwitcher={() => setShowSwitcher(true)}
        isCollapsed={isCollapsed}
      />

      <div className={cn(
        "flex-1 flex flex-col min-w-0 h-screen overflow-hidden relative transition-all duration-300",
        isCollapsed ? "lg:ml-20" : "lg:ml-72"
      )}>
        <Navbar
          currentWorkspaceName={currentWorkspace?.name || "Workspace"}
          onOpenSidebar={() => setIsSidebarOpen(true)}
          isCollapsed={isCollapsed}
          onToggleCollapsed={toggleCollapsed}
        />

        <main className="flex-1 overflow-y-auto p-4 md:p-8 lg:p-10 relative scroll-smooth">
          <div className="max-w-7xl mx-auto">
            <Outlet />
          </div>
        </main>
      </div>

      <Modal
        isOpen={showSwitcher}
        onClose={() => setShowSwitcher(false)}
        title={t("common.switch_workspace")}
        subtitle={t("common.office_branch")}
        icon={<Building2 className="w-6 h-6" />}
        maxWidth="lg"
      >
        <div className="space-y-8">
          <p className="text-sm text-slate-500 dark:text-slate-400 font-medium px-1">{t("common.select_workspace_desc")}</p>
          <WorkspaceSwitcher />
          <div className="pt-6 border-t border-slate-50 dark:border-slate-800 flex justify-end">
            <button
              onClick={() => setShowSwitcher(false)}
              className="px-8 py-3 bg-slate-100 dark:bg-slate-800 text-foreground/70 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-200 dark:hover:bg-slate-700 transition-all"
            >
              {t("common.close")}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
