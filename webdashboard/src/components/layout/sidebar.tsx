import type { ReactNode } from "react";

import { Link, useLocation } from "@tanstack/react-router";
import { ChevronLeft, ChevronRight, Clock, X } from "lucide-react";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useFocusTrap } from "@/hooks/use-focus-trap";
import { useFilteredMenuItems, type MenuGroupType, type MenuLinkType } from "@/hooks/use-nav-menu-items";
import { useOrganization } from "@/hooks/use-organization";
import { cn } from "@/lib/utils";

export interface SidebarProps {
  isCollapsed: boolean;
  isMobileOpen?: boolean;
  onMobileClose?: () => void;
  onToggleCollapse: () => void;
}

interface SidebarLinkProps {
  active: boolean;
  highlightIcon?: boolean;
  isCollapsed: boolean;
  item: MenuLinkType;
  onClick?: () => void;
}

interface SidebarContentProps {
  collapseLabel: string;
  collapseText: string;
  dashboard: MenuLinkType;
  expandLabel: string;
  groups: MenuGroupType[];
  isCollapsed: boolean;
  isMobileOpen: boolean;
  onMobileClose?: () => void;
  onToggleCollapse: () => void;
  organizationName: string;
  pathname: string;
  settings: MenuLinkType;
}

const SIDEBAR_LINK_CLASSES =
  "flex items-center gap-3 rounded-lg px-2.5 py-2 text-sm font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-primary";

function isItemActive(pathname: string, to: string, exact?: boolean) {
  return exact ? pathname === to : pathname === to || pathname.startsWith(`${to}/`);
}

function useSidebarKeyboardShortcut(onToggleCollapse: () => void) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const target = event.target;
      const isInput =
        target instanceof HTMLElement &&
        (["INPUT", "TEXTAREA"].includes(target.tagName) || target.isContentEditable);
      if (isInput) return;

      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "b") {
        event.preventDefault();
        onToggleCollapse();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => { window.removeEventListener("keydown", handleKeyDown); };
  }, [onToggleCollapse]);
}

function SidebarLink({ active, highlightIcon = true, isCollapsed, item, onClick }: SidebarLinkProps) {
  const Icon = item.icon;

  return (
    <Link
      aria-label={item.label}
      className={cn(
        SIDEBAR_LINK_CLASSES,
        isCollapsed && "justify-center px-0 h-9 w-9 mx-auto",
        active
          ? "bg-primary/10 text-primary font-semibold"
          : "text-muted-foreground hover:bg-muted hover:text-foreground"
      )}
      data-testid={item.testId}
      onClick={onClick}
      title={isCollapsed ? item.label : undefined}
      to={item.to}
    >
      {Icon ? (
        <Icon
          aria-hidden="true"
          className={cn(
            "h-4 w-4 shrink-0",
            isCollapsed && "h-5 w-5",
            highlightIcon && active && "text-primary"
          )}
        />
      ) : null}
      {!isCollapsed && <span className="truncate">{item.label}</span>}
    </Link>
  );
}

function SidebarBrand({
  dashboard,
  isCollapsed,
  isMobileOpen,
  onMobileClose,
  organizationName,
}: Pick<SidebarContentProps, "dashboard" | "isCollapsed" | "isMobileOpen" | "onMobileClose" | "organizationName">) {
  return (
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
            <span className="text-sm font-semibold tracking-tight text-foreground">LeeAttend</span>
            <span className="text-[11px] text-muted-foreground truncate font-medium">
              {organizationName}
            </span>
          </div>
        )}
      </Link>

      {isMobileOpen && onMobileClose ? (
        <Button
          aria-label="Close navigation"
          className="h-8 w-8 p-0 text-muted-foreground hover:text-foreground lg:hidden"
          data-testid="sidebar-mobile-close-btn"
          onClick={onMobileClose}
          size="sm"
          variant="ghost"
        >
          <X className="h-4 w-4" />
        </Button>
      ) : null}
    </div>
  );
}

function SidebarNavigation({
  dashboard,
  groups,
  isCollapsed,
  onMobileClose,
  pathname,
}: Pick<SidebarContentProps, "dashboard" | "groups" | "isCollapsed" | "onMobileClose" | "pathname">) {
  return (
    <nav
      aria-label="Sidebar Navigation"
      className="flex-1 space-y-4 overflow-y-auto px-2 py-3 scrollbar-thin scrollbar-thumb-muted-foreground/20"
      data-testid="sidebar-nav-container"
    >
      <div>
        <Link
          activeOptions={{ exact: true }}
          aria-label={dashboard.label}
          className={cn(
            SIDEBAR_LINK_CLASSES,
            isCollapsed && "justify-center px-0 h-9 w-9 mx-auto",
            isItemActive(pathname, dashboard.to, dashboard.exact)
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

      {groups.map((group) => (
        <SidebarGroup
          group={group}
          isCollapsed={isCollapsed}
          key={group.section}
          onMobileClose={onMobileClose}
          pathname={pathname}
        />
      ))}
    </nav>
  );
}

function SidebarGroup({
  group,
  isCollapsed,
  onMobileClose,
  pathname,
}: {
  group: MenuGroupType;
  isCollapsed: boolean;
  onMobileClose?: () => void;
  pathname: string;
}) {
  return (
    <div className="space-y-1" data-testid={group.testId}>
      {!isCollapsed ? (
        <div className="px-2 pt-2 pb-1 text-[11px] font-semibold tracking-wider text-muted-foreground/70 uppercase select-none">
          {group.label}
        </div>
      ) : (
        <div className="my-1 border-t border-border/40 mx-2" />
      )}

      <div className="space-y-0.5">
        {group.items.map((item) => (
          <SidebarLink
            active={isItemActive(pathname, item.to, item.exact)}
            isCollapsed={isCollapsed}
            item={item}
            key={item.to}
            onClick={onMobileClose}
          />
        ))}
      </div>
    </div>
  );
}

function SidebarFooter({
  isCollapsed,
  onMobileClose,
  onToggleCollapse,
  settings,
  pathname,
  collapseLabel,
  expandLabel,
  collapseText,
}: Pick<SidebarContentProps, "isCollapsed" | "onMobileClose" | "onToggleCollapse" | "settings" | "pathname" | "collapseLabel" | "expandLabel" | "collapseText">) {
  return (
    <div className="border-t border-border p-2 space-y-1 shrink-0 bg-card/40">
      {!!settings.to && (
        <SidebarLink
          active={isItemActive(pathname, settings.to)}
          highlightIcon={false}
          isCollapsed={isCollapsed}
          item={settings}
          onClick={onMobileClose}
        />
      )}

      <Button
        aria-label={isCollapsed ? expandLabel : collapseLabel}
        className={cn(
          "w-full text-muted-foreground hover:text-foreground hover:bg-muted transition-colors hidden lg:flex items-center",
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
            <span className="text-xs font-medium">{collapseText}</span>
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
  );
}

function SidebarContent(props: SidebarContentProps) {
  return (
    <div className="flex h-full flex-col justify-between overflow-hidden">
      <SidebarBrand {...props} />
      <SidebarNavigation {...props} />
      <SidebarFooter {...props} />
    </div>
  );
}

function MobileDrawer({ isOpen, onClose, children }: { isOpen: boolean; onClose?: () => void; children: ReactNode }) {
  const containerRef = useFocusTrap(isOpen);

  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose?.();
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div
      aria-modal="true"
      className="fixed inset-0 z-50 flex lg:hidden"
      data-testid="sidebar-mobile-drawer"
      role="dialog"
    >
      <div
        aria-hidden="true"
        className="fixed inset-0 bg-black/60 backdrop-blur-xs transition-opacity animate-in fade-in duration-200"
        data-testid="sidebar-mobile-backdrop"
        onClick={onClose}
      />
      <div
        className="relative flex w-72 max-w-[80vw] flex-col bg-card border-r border-border shadow-2xl animate-in slide-in-from-left duration-200"
        ref={containerRef}
      >
        {children}
      </div>
    </div>
  );
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
  useSidebarKeyboardShortcut(onToggleCollapse);

  const contentProps: SidebarContentProps = {
    collapseLabel: t("common.collapse_sidebar", "Collapse sidebar (⌘B)"),
    collapseText: t("common.collapse", "Thu gọn"),
    dashboard,
    expandLabel: t("common.expand_sidebar", "Expand sidebar (⌘B)"),
    groups,
    isCollapsed,
    isMobileOpen,
    onMobileClose,
    onToggleCollapse,
    organizationName: organization?.name ?? "LeeAttend",
    pathname: location.pathname,
    settings,
  };

  return (
    <>
      <aside
        aria-label="Application Sidebar"
        className={cn(
          "hidden lg:flex flex-col border-r border-border bg-card transition-all duration-200 ease-in-out shrink-0 select-none h-dvh",
          isCollapsed ? "w-16" : "w-64"
        )}
        data-collapsed={isCollapsed}
        data-testid="sidebar-container"
      >
        <SidebarContent {...contentProps} />
      </aside>

      <MobileDrawer isOpen={isMobileOpen} onClose={onMobileClose}>
        <SidebarContent {...contentProps} />
      </MobileDrawer>
    </>
  );
}

export default Sidebar;
