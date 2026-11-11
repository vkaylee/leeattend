/* eslint-disable react/forbid-elements */
import { Link } from "@tanstack/react-router";
import { ChevronDown, ChevronLeft, ChevronRight, ExternalLink, X, User, FolderOpen, FileText, Milestone, DollarSign, History, ShieldCheck, Maximize2, Minimize2 } from "lucide-react";
import { createElement, useCallback, useEffect, useRef, useState, type ComponentType } from "react";
import type { TFunction } from "i18next";

import type { Employee } from "@/lib/api";

import Button from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

export type DetailTabType = "profile" | "documents" | "contracts" | "phases" | "salary" | "compensation" | "attendance-policy";

export interface TabDefinition {
  id: DetailTabType;
  i18nKey: string;
  icon: ComponentType<{ size?: number; className?: string }>;
}

export const EMPLOYEE_DRAWER_TAB_CONFIG: readonly TabDefinition[] = [
  { id: "profile", i18nKey: "employees.details.tabs.profile", icon: User },
  { id: "documents", i18nKey: "employees.details.tabs.documents", icon: FolderOpen },
  { id: "contracts", i18nKey: "employees.details.tabs.contracts", icon: FileText },
  { id: "phases", i18nKey: "employees.details.tabs.phases", icon: Milestone },
  { id: "salary", i18nKey: "employees.details.tabs.salary", icon: DollarSign },
  { id: "compensation", i18nKey: "employees.details.tabs.compensation", icon: History },
  { id: "attendance-policy", i18nKey: "employees.details.tabs.attendance_policy", icon: ShieldCheck },
] as const;

export interface DrawerHeaderProps {
    activeDetailTab: DetailTabType;
    employee: Employee;
    isExpanded?: boolean;
    onClose: () => void;
    onToggleExpand?: () => void;
    setActiveDetailTab: (tab: DetailTabType) => void;
    t: TFunction;
}

// ── Scrollable tab nav with overflow indicators ─────────────
function DesktopTabNav({
  activeDetailTab,
  setActiveDetailTab,
  tabs,
  t,
}: {
  activeDetailTab: DetailTabType;
  setActiveDetailTab: (tab: DetailTabType) => void;
  tabs: { id: DetailTabType; label: string; icon: ComponentType<{ size?: number; className?: string }> }[];
  t: TFunction;
}) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(false);

  const updateScrollState = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    setCanScrollLeft(el.scrollLeft > 4);
    setCanScrollRight(el.scrollLeft + el.clientWidth < el.scrollWidth - 4);
  }, []);

  // Scroll arrows visibility
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    updateScrollState();
    el.addEventListener("scroll", updateScrollState, { passive: true });
    const ro = new ResizeObserver(updateScrollState);
    ro.observe(el);
    return () => { el.removeEventListener("scroll", updateScrollState); ro.disconnect(); };
  }, [updateScrollState]);

  // Auto-scroll active tab into view when it changes
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const activeBtn = el.querySelector(`[data-testid="employees-tab-${activeDetailTab}"]`);
    activeBtn?.scrollIntoView({ behavior: "smooth", block: "nearest", inline: "nearest" });
  }, [activeDetailTab]);

  const scroll = (dir: "left" | "right") => {
    scrollRef.current?.scrollBy({ left: dir === "left" ? -240 : 240, behavior: "smooth" });
  };

  return (
    <div className="hidden md:flex items-center gap-1">
      <button
        aria-hidden={!canScrollLeft}
        aria-label={t("common.scroll_left", "Scroll tabs left")}
        className={cn(
          "shrink-0 size-9 rounded-[14px] flex items-center justify-center transition-all border",
          canScrollLeft
            ? "bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 text-slate-500 hover:text-primary-600 shadow-sm cursor-pointer"
            : "border-transparent text-transparent pointer-events-none",
        )}
        data-testid="employees-tab-scroll-left"
        onClick={() => { scroll("left"); }}
        tabIndex={canScrollLeft ? 0 : -1}
        type="button"
      >
        <ChevronLeft size={16} />
      </button>

      <div className="relative flex-1 min-w-0">
        {/* Gradient fades */}
        <div
          aria-hidden
          className={cn(
            "pointer-events-none absolute inset-y-0 left-0 w-8 z-10 bg-gradient-to-r from-white dark:from-slate-900/50 to-transparent transition-opacity duration-200",
            canScrollLeft ? "opacity-100" : "opacity-0",
          )}
        />
        <div
          aria-hidden
          className={cn(
            "pointer-events-none absolute inset-y-0 right-0 w-8 z-10 bg-gradient-to-l from-white dark:from-slate-900/50 to-transparent transition-opacity duration-200",
            canScrollRight ? "opacity-100" : "opacity-0",
          )}
        />

        <div
          aria-label={t("employees.details.tabs_nav", "Employee sections")}
          className="flex overflow-x-auto scrollbar-hide bg-white dark:bg-slate-900/50 p-1.5 rounded-[22px] border border-slate-200 dark:border-slate-700/50 shadow-inner w-full scroll-smooth"
          data-testid="employees-tab-scroll-container"
          ref={scrollRef}
          role="tablist"
        >
          {tabs.map(tab => (
            <Button
              aria-controls={`employees-tabpanel-${tab.id}`}
              aria-selected={activeDetailTab === tab.id}
              className={cn(
                "px-5 py-2.5 shrink-0 whitespace-nowrap rounded-[18px] text-xs font-semibold uppercase tracking-wider transition-colors duration-200 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 h-auto shadow-none border border-transparent",
                activeDetailTab === tab.id
                  ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900 font-bold"
                  : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-transparent",
              )}
              data-testid={`employees-tab-${tab.id}`}
              id={`employees-tab-${tab.id}`}
              key={tab.id}
              onClick={() => { setActiveDetailTab(tab.id); }}
              role="tab"
              type="button"
              variant="ghost"
            >
              {tab.label.toUpperCase()}
            </Button>
          ))}
        </div>
      </div>

      <button
        aria-hidden={!canScrollRight}
        aria-label={t("common.scroll_right", "Scroll tabs right")}
        className={cn(
          "shrink-0 size-9 rounded-[14px] flex items-center justify-center transition-all border",
          canScrollRight
            ? "bg-white dark:bg-slate-900 border-slate-200 dark:border-slate-700 text-slate-500 hover:text-primary-600 shadow-sm cursor-pointer"
            : "border-transparent text-transparent pointer-events-none",
        )}
        data-testid="employees-tab-scroll-right"
        onClick={() => { scroll("right"); }}
        tabIndex={canScrollRight ? 0 : -1}
        type="button"
      >
        <ChevronRight size={16} />
      </button>
    </div>
  );
}

export default function DrawerHeader({
    employee,
    onClose,
    activeDetailTab,
    setActiveDetailTab,
    isExpanded,
    onToggleExpand,
    t
}: DrawerHeaderProps) {
    const tabs = EMPLOYEE_DRAWER_TAB_CONFIG.map(tab => ({
        id: tab.id,
        label: t(tab.i18nKey),
        icon: tab.icon,
    }));
    const activeTabObj = tabs.find(tab => tab.id === activeDetailTab) ?? tabs[0];

    return (
        <div className="sticky top-0 z-20 px-6 py-5 sm:px-8 sm:py-6 border-b border-slate-50 dark:border-slate-800 bg-slate-50/95 dark:bg-slate-800/95 backdrop-blur-sm shadow-sm">
            <div className="flex justify-between items-start mb-6">
                <div className="flex items-center gap-4 sm:gap-6">
                    <div className="w-14 h-14 sm:w-16 sm:h-16 bg-primary-600 text-white rounded-[1.5rem] flex items-center justify-center font-bold text-2xl sm:text-3xl shadow-xl shadow-primary-100 dark:shadow-black/30 border-4 border-white rotate-3 group hover:rotate-0 transition-transform duration-500 uppercase shrink-0">{employee.full_name[0]}</div>
                    <div>
                        <Typography className="text-2xl sm:text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none mb-2" variant="h2">{employee.full_name}</Typography>
                        <div className="flex items-center gap-3">
                            <span className="text-primary-600 dark:text-primary-400 font-mono text-xs font-bold tracking-widest uppercase bg-white dark:bg-slate-900 px-3 py-1 rounded-full border border-primary-100 dark:border-primary-800 shadow-sm">{employee.employee_id_number}</span>
                            <div className="w-1.5 h-1.5 bg-slate-200 dark:bg-slate-600 rounded-full" />
                            <span className={cn("text-[9px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full border shadow-sm", employee.active_employment_id ? 'bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800' : 'bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60')}>{employee.active_employment_id ? t("employees.table.active").toUpperCase() : t("employees.table.terminated").toUpperCase()}</span>
                        </div>
                    </div>
                </div>
                <div className="flex items-center gap-2">
                    <Link params={{ employeeId: employee.id }} to="/employees/$employeeId">
                        <Button
                            aria-label={t("employees.details.open_full_page", "Mở toàn trang")}
                            className="p-3 bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-[1.25rem] transition-all shadow-md active:scale-90 border border-slate-100 dark:border-slate-800 text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:hover:text-primary-400 h-auto w-auto shadow-none flex items-center justify-center"
                            data-testid="open-full-page"
                            size="icon"
                            type="button"
                            variant="ghost"
                        >
                            <ExternalLink size={20} />
                        </Button>
                    </Link>
                    {onToggleExpand && (
                        <Button
                            aria-label={isExpanded ? t("common.minimize", "Thu nhỏ") : t("common.maximize", "Phóng to")}
                            className="p-3 bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-[1.25rem] transition-all shadow-md active:scale-90 border border-slate-100 dark:border-slate-800 text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:hover:text-primary-400 h-auto w-auto shadow-none hidden sm:flex items-center justify-center"
                            data-testid="toggle-drawer-expand"
                            onClick={onToggleExpand}
                            size="icon"
                            type="button"
                            variant="ghost"
                        >
                            {isExpanded ? <Minimize2 size={20} /> : <Maximize2 size={20} />}
                        </Button>
                    )}
                    <Button aria-label={t("common.close")} className="p-3 bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-[1.25rem] transition-all shadow-md active:scale-90 border border-slate-100 dark:border-slate-800 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 h-auto w-auto shadow-none" onClick={onClose} size="icon" type="button" variant="ghost"><X size={24} /></Button>
                </div>
            </div>

            {/* Desktop Navigation Tabs */}
            <DesktopTabNav activeDetailTab={activeDetailTab} setActiveDetailTab={setActiveDetailTab} t={t} tabs={tabs} />

            {/* Mobile Navigation Dropdown */}
            <div className="flex md:hidden w-full relative">
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button
                            className="w-full flex items-center justify-between px-6 py-4 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700/50 rounded-[18px] shadow-sm text-xs font-semibold uppercase tracking-wider text-slate-700 dark:text-slate-200 transition-all hover:bg-slate-50 dark:hover:bg-slate-800 h-auto"
                            type="button"
                            variant="ghost"
                        >
                            <span className="flex items-center gap-3">
                                {createElement(activeTabObj.icon, { size: 14, className: "text-primary-500" })}
                                {activeTabObj.label.toUpperCase()}
                            </span>
                            <ChevronDown className="text-slate-400 dark:text-slate-500 transition-transform duration-200" size={14} />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="w-[calc(100vw-5rem)] max-w-md max-h-[300px] overflow-y-auto rounded-2xl shadow-xl">
                        {tabs.map((tab) => (
                            <DropdownMenuItem
                                className={cn(
                                    "flex items-center gap-3 px-5 py-3 rounded-xl text-xs font-semibold uppercase tracking-wider transition-all",
                                    activeDetailTab === tab.id
                                        ? "bg-primary-50 dark:bg-primary-950/50 text-primary-600 dark:text-primary-400"
                                        : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
                                )}
                                key={tab.id}
                                onClick={() => {
                                    setActiveDetailTab(tab.id);
                                }}
                            >
                                {createElement(tab.icon, {
                                    size: 12,
                                    className: cn(
                                        activeDetailTab === tab.id
                                            ? "text-primary-500"
                                            : "text-slate-400 dark:text-slate-500"
                                    ),
                                })}
                                {tab.label.toUpperCase()}
                            </DropdownMenuItem>
                        ))}
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>
        </div>
    );
}
