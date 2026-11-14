import { Link, useLocation } from "@tanstack/react-router";
import {
  ChevronLeft,
  ChevronRight,
  Clock,
  X,
} from "lucide-react";
import React, { useEffect } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useFilteredMenuItems } from "@/hooks/use-nav-menu-items";
import { useOrganization } from "@/hooks/use-organization";
import { cn } from "@/lib/utils";

export interface SidebarProps {
  isCollapsed: boolean;
  isMobileOpen?: boolean;
  onMobileClose?: () => void;
  onToggleCollapse: () => void;
}

export function Sidebar({
  isCollapsed,
  onToggleCollapse,
  isMobileOpen = false,
  onMobileClose,
}: SidebarProps) {
  const { t } = useTranslation(["common", "settings"]);
  const location = useLocation();
  const { organization } = useOrganization();
  const { dashboard, settings, groups } = useFilteredMenuItems();

  // Keyboard shortcut: Cmd+B or Ctrl+B to toggle sidebar
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isInput =
        ["INPUT", "TEXTAREA"].includes((e.target as HTMLElement).tagName) ||
        (e.target as HTMLElement).isContentEditable;
      if (isInput) return;

      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "b") {
        e.preventDefault();
        onToggleCollapse();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => { window.removeEventListener("keydown", handleKeyDown); };
  }, [onToggleCollapse]);

  const isItemActive = (to: string, exact?: boolean) => {
    if (exact) {
      return location.pathname === to;
    }
    return location.pathname === to || location.pathname.startsWith(`${to}/`);
  };

  const navContent = (
    <div className="flex h-full flex-col justify-between overflow-hidden">
      {/* ── Brand Header ── */}
      <div
        className={cn(
          "flex h-14 items-center border-b border-border px-3 shrink-0 transition-all",
          isCollapsed ? "justify-center" : "justify-between gap-2 px-4"
        )}
      >
        <Link
          className="flex items-center gap-2.5 overflow-hidden group outline-none focus-visible:ring-2 focus-visible:ring-primary rounded-lg p-1"
          data-testid="sidebar-logo"
          to={dashboard.to}
        >
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground font-bold shadow-xs">
            <Clock className="h-4 w-4" />
          </div>
          {!isCollapsed && (
            <div className="flex flex-col truncate text-left">
              <span className="text-sm font-semibold tracking-tight text-foreground">
                LeeAttend
              </span>
              <span className="text-[11px] text-muted-foreground truncate font-medium">
                {organization?.name ?? "LeeAttend"}
              </span>
            </div>
          )}
        </Link>

        {/* Mobile close button */}
        {!!isMobileOpen && !!onMobileClose && (
          <Button
            aria-label="Close navigation"
            className="h-8 w-8 p-0 text-muted-foreground hover:text-foreground md:hidden"
            data-testid="sidebar-mobile-close-btn"
            onClick={onMobileClose}
            size="sm"
            variant="ghost"
          >
            <X className="h-4 w-4" />
          </Button>
        )}
      </div>

      {/* ── Navigation Items ── */}
      <nav
        aria-label="Sidebar Navigation"
        className="flex-1 space-y-4 overflow-y-auto px-2 py-3 scrollbar-thin scrollbar-thumb-muted-foreground/20"
        data-testid="sidebar-nav-container"
      >
        {/* Dashboard Link */}
        <div>
          <Link
            activeOptions={{ exact: true }}
            aria-label={dashboard.label}
            className={cn(
              "flex items-center gap-3 rounded-lg px-2.5 py-2 text-sm font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-primary",
              isCollapsed && "justify-center px-0 h-9 w-9 mx-auto",
              isItemActive(dashboard.to, dashboard.exact)
                ? "bg-primary text-primary-foreground shadow-xs font-semibold"
                : "text-muted-foreground hover:bg-muted hover:text-foreground"
            )}
            data-testid={dashboard.testId}
            title={isCollapsed ? dashboard.label : undefined}
            to={dashboard.to}
          >
            {!!dashboard.icon && (
              <dashboard.icon
                aria-hidden="true"
                className={cn("h-4 w-4 shrink-0", isCollapsed && "h-5 w-5")}
              />
            )}
            {!isCollapsed && <span className="truncate">{dashboard.label}</span>}
          </Link>
        </div>

        {/* Groups */}
        {groups.map((group) => (
          <div className="space-y-1" data-testid={group.testId} key={group.section}>
            {!isCollapsed ? (
              <div className="px-2 pt-2 pb-1 text-[11px] font-semibold tracking-wider text-muted-foreground/70 uppercase select-none">
                {group.label}
              </div>
            ) : (
              <div className="my-1 border-t border-border/40 mx-2" />
            )}

            <div className="space-y-0.5">
              {group.items.map((item) => {
                const active = isItemActive(item.to, item.exact);
                return (
                  <Link
                    aria-label={item.label}
                    className={cn(
                      "flex items-center gap-3 rounded-lg px-2.5 py-2 text-sm font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-primary",
                      isCollapsed && "justify-center px-0 h-9 w-9 mx-auto",
                      active
                        ? "bg-primary/10 text-primary font-semibold"
                        : "text-muted-foreground hover:bg-muted hover:text-foreground"
                    )}
                    data-testid={item.testId}
                    key={item.to}
                    onClick={onMobileClose}
                    title={isCollapsed ? item.label : undefined}
                    to={item.to}
                  >
                    {!!item.icon && (
                      <item.icon
                        aria-hidden="true"
                        className={cn(
                          "h-4 w-4 shrink-0",
                          isCollapsed && "h-5 w-5",
                          active && "text-primary"
                        )}
                      />
                    )}
                    {!isCollapsed && <span className="truncate">{item.label}</span>}
                  </Link>
                );
              })}
            </div>
          </div>
        ))}
      </nav>

      {/* ── Bottom Section: Settings & Toggle ── */}
      <div className="border-t border-border p-2 space-y-1 shrink-0 bg-card/40">
        {settings.to && (
          <Link
            aria-label={settings.label}
            className={cn(
              "flex items-center gap-3 rounded-lg px-2.5 py-2 text-sm font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-primary",
              isCollapsed && "justify-center px-0 h-9 w-9 mx-auto",
              isItemActive(settings.to)
                ? "bg-primary/10 text-primary font-semibold"
                : "text-muted-foreground hover:bg-muted hover:text-foreground"
            )}
            data-testid={settings.testId}
            onClick={onMobileClose}
            title={isCollapsed ? settings.label : undefined}
            to={settings.to}
          >
            {!!settings.icon && (
              <settings.icon
                aria-hidden="true"
                className={cn("h-4 w-4 shrink-0", isCollapsed && "h-5 w-5")}
              />
            )}
            {!isCollapsed && <span className="truncate">{settings.label}</span>}
          </Link>
        )}

        {/* Desktop Collapse Toggle */}
        <Button
          aria-label={
            isCollapsed
              ? t("common.expand_sidebar", "Expand sidebar (⌘B)")
              : t("common.collapse_sidebar", "Collapse sidebar (⌘B)")
          }
          className={cn(
            "w-full text-muted-foreground hover:text-foreground hover:bg-muted transition-colors hidden md:flex items-center",
            isCollapsed ? "justify-center px-0 h-9" : "justify-between px-2.5 h-9"
          )}
          data-testid="sidebar-toggle-btn"
          onClick={onToggleCollapse}
          size="sm"
          type="button"
          variant="ghost"
        >
          {isCollapsed ? (
            <ChevronRight className="h-4 w-4" />
          ) : (
            <>
              <span className="text-xs font-medium">
                {t("common.collapse", "Thu gọn")}
              </span>
              <div className="flex items-center gap-1">
                <kbd className="rounded border border-border bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground font-mono">
                  ⌘B
                </kbd>
                <ChevronLeft className="h-4 w-4" />
              </div>
            </>
          )}
        </Button>
      </div>
    </div>
  );

  return (
    <>
      {/* Desktop Persistent Sidebar */}
      <aside
        aria-label="Application Sidebar"
        className={cn(
          "hidden md:flex flex-col border-r border-border bg-card transition-all duration-200 ease-in-out shrink-0 select-none h-screen",
          isCollapsed ? "w-16" : "w-64"
        )}
        data-collapsed={isCollapsed}
        data-testid="sidebar-container"
      >
        {navContent}
      </aside>

      {/* Mobile Drawer (with backdrop) */}
      {!!isMobileOpen && (
        <div
          aria-modal="true"
          className="fixed inset-0 z-50 flex md:hidden"
          data-testid="sidebar-mobile-drawer"
          role="dialog"
        >
          {/* Backdrop */}
          <div
            aria-hidden="true"
            className="fixed inset-0 bg-black/60 backdrop-blur-xs transition-opacity animate-in fade-in duration-200"
            data-testid="sidebar-mobile-backdrop"
            onClick={onMobileClose}
          />
          {/* Drawer content */}
          <div className="relative flex w-72 max-w-[80vw] flex-col bg-card border-r border-border shadow-2xl animate-in slide-in-from-left duration-200">
            {navContent}
          </div>
        </div>
      )}
    </>
  );
}
export default Sidebar;
