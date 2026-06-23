import { Outlet, useLocation } from "@tanstack/react-router";
import { Building2 } from "lucide-react";
import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";

import { ErrorBoundary } from "@/components/common/error-boundary";
import { Typography } from "@/components/ui/typography";
import { useI18nSync } from "@/hooks/use-i18n-sync";
import { useTheme } from "@/hooks/use-theme";
import { useWorkspaces } from "@/hooks/use-workspace";
import { getCurrentWorkspaceId } from "@/lib/api";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

import Button from "../ui/button";
import Modal from "../ui/modal";
import Navbar from "./navbar";
import Sidebar from "./sidebar";
import WorkspaceSwitcher from "./workspace-switcher";



// Stable module-level arrays for sidebar section matching
const ORG_PATHS: string[] = [ROUTES.PEOPLE, ROUTES.EMPLOYEES, ROUTES.DEPARTMENTS];
const TIME_PATHS: string[] = [ROUTES.ATTENDANCE_LOGS, ROUTES.LEAVE_REQUESTS, ROUTES.ATTENDANCE_ADJUSTMENTS, ROUTES.REPORTS, ROUTES.LEAVE_BALANCES, ROUTES.SHIFT_SWAPS];
const SHIFTS_PATHS: string[] = [ROUTES.SHIFTS, ROUTES.SHIFTS_RULES];
const PAYROLL_PATHS: string[] = [ROUTES.PAYROLL];

function SidebarOverlay({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  if (!isOpen) return null;
  return (
    <div
      className="fixed inset-0 bg-slate-900/50 dark:bg-slate-950/60 backdrop-blur-sm z-40 lg:hidden transition-opacity duration-500"
      onClick={onClose}
      onKeyDown={(e) => {
        if (e.key === 'Escape' || e.key === 'Enter' || e.key === ' ') onClose();
      }}
      role="presentation"
    />
  );
}

function WorkspaceSwitcherModal({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  const { t } = useTranslation();
  return (
    <Modal
      icon={<Building2 className="w-6 h-6" />}
      isOpen={isOpen}
      maxWidth="lg"
      onClose={onClose}
      subtitle={t("common.office_branch")}
      title={t("common.switch_workspace")}
    >
      <div className="space-y-8">
        <Typography className="text-sm text-slate-500 dark:text-slate-400 font-medium px-1" variant="p">{t("common.select_workspace_desc")}</Typography>
        <WorkspaceSwitcher />
        <div className="pt-6 border-t border-slate-50 dark:border-slate-800 flex justify-end">
          <Button className="px-8 py-3 bg-slate-100 dark:bg-slate-800 text-foreground/70 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-slate-200 dark:hover:bg-slate-700 transition-all"
            onClick={onClose}
            variant="ghost"
          >
            {t("common.close")}
          </Button>
        </div>
      </div>
    </Modal>
  );
}

function useSidebarState(locationPathname: string) {
  const getInitialOpenSections = () => {
    return {
      org: ORG_PATHS.includes(locationPathname),
      time: TIME_PATHS.includes(locationPathname) || locationPathname === ROUTES.HOME,
      shifts: SHIFTS_PATHS.includes(locationPathname),
      payroll: PAYROLL_PATHS.includes(locationPathname),
      system: locationPathname.startsWith(ROUTES.SETTINGS.ROOT) || locationPathname.startsWith("/system/") || locationPathname.startsWith("/infra/"),
      ess: locationPathname.startsWith("/ess/"),
    };
  };

  const [openSections, setOpenSections] = useState<Record<string, boolean>>(getInitialOpenSections());
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [isCollapsed, setIsCollapsed] = useState(() => {
    const saved = localStorage.getItem("sidebar-collapsed");
    return saved === "true";
  });

  const toggleSection = (section: string) => {
    setOpenSections(previous => ({
      org: false,
      time: false,
      shifts: false,
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
    const isOrg = ORG_PATHS.includes(locationPathname);
    const isTime = TIME_PATHS.includes(locationPathname);
    const isShifts = SHIFTS_PATHS.includes(locationPathname);
    const isPayroll = PAYROLL_PATHS.includes(locationPathname);
    const isSystem = locationPathname.startsWith(ROUTES.SETTINGS.ROOT) || locationPathname.startsWith("/system/") || locationPathname.startsWith("/infra/");
    const isEss = locationPathname.startsWith("/ess/");

    const timer = setTimeout(() => {
      setOpenSections(previous => {
        const next = { ...previous };
        let hasChanged = false;
        
        const update = (key: string, val: boolean) => {
          if (val && !previous[key]) {
            next[key] = true;
          }
        };
        
        update('org', isOrg);
        update('time', isTime);
        update('shifts', isShifts);
        update('payroll', isPayroll);
        update('system', isSystem);
        update('ess', isEss);
        
        hasChanged = Object.keys(next).some(key => next[key] !== previous[key]);
        return hasChanged ? next : previous;
      });
      setIsSidebarOpen(false);
    }, 0);

    return () => { clearTimeout(timer); };
  }, [locationPathname]);

  return {
    isSidebarOpen, setIsSidebarOpen,
    isCollapsed, toggleCollapsed,
    openSections, toggleSection
  };
}

export default function MainLayout() {
  useTheme();
  useI18nSync();
  const location = useLocation();

  const [showSwitcher, setShowSwitcher] = useState(false);
  
  const {
    isSidebarOpen, setIsSidebarOpen,
    isCollapsed, toggleCollapsed,
    openSections, toggleSection
  } = useSidebarState(location.pathname);

  const { workspaces } = useWorkspaces();
  const currentWsId = getCurrentWorkspaceId();
  const currentWorkspace = workspaces.find(ws => ws.id === currentWsId);

  const isOrgChart = location.pathname.endsWith("/org-chart");

  return (
    <div className="flex min-h-screen bg-background text-foreground font-sans relative overflow-x-hidden">
      <SidebarOverlay isOpen={isSidebarOpen} onClose={() => { setIsSidebarOpen(false); }} />

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
          currentWorkspaceName={currentWorkspace?.name ?? "Workspace"}
          isCollapsed={isCollapsed}
          onOpenSidebar={() => { setIsSidebarOpen(true); }}
          onToggleCollapsed={toggleCollapsed}
        />

        <main className={cn("flex-1 relative scroll-smooth", isOrgChart ? "overflow-hidden p-0" : "overflow-y-auto p-4 md:p-6 lg:p-8")}>
          <div className={cn(isOrgChart ? "w-full h-full animate-in fade-in duration-500" : "max-w-7xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500 fill-mode-both")} key={location.pathname}>
            <ErrorBoundary>
              <Outlet />
            </ErrorBoundary>
          </div>
        </main>
      </div>

      <WorkspaceSwitcherModal isOpen={showSwitcher} onClose={() => { setShowSwitcher(false); }} />
    </div>
  );
}
