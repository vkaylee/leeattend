import { Link, useLocation } from "@tanstack/react-router";
import {
  X,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import React from "react";

import type { MenuGroupType, MenuLinkType } from "@/hooks/use-nav-menu-items";

import Button from "@/components/ui/button";
import { useFilteredMenuItems } from "@/hooks/use-nav-menu-items";
import { useWorkspace } from "@/hooks/use-workspace";
import { cn } from "@/lib/utils";


interface SidebarProps {
  isCollapsed: boolean;
  isOpen: boolean;
  onClose: () => void;
  openSections: Partial<Record<string, boolean>>;
  toggleSection: (section: string) => void;
}

type MenuItem = MenuLinkType | MenuGroupType;

function SidebarLink({
  to,
  icon: Icon,
  label,
  testId,
  isCollapsed,
  isActive
}: {
  isActive: boolean;
  isCollapsed: boolean;
  label: string;
  testId: string;
  to: string;
  icon?: React.ElementType;
}) {
  return (
    <Link
      className={cn(
        "flex items-center gap-3 px-4 py-3 rounded-2xl transition-all duration-300 group relative",
        isCollapsed ? "justify-center px-0" : "px-4",
        isActive
          ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300 shadow-sm shadow-primary-100/50"
          : "text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 hover:text-slate-700 dark:hover:text-slate-200"
      )}
      data-testid={testId}
      to={to as "/"}
    >
      <div className={cn(
        "p-2 rounded-xl transition-colors",
        isActive
          ? "bg-white dark:bg-primary-900/30 shadow-sm"
          : "bg-slate-50 dark:bg-slate-800 group-hover:bg-white dark:group-hover:bg-slate-700"
      )}>
        {Icon ? <Icon className="h-4 w-4" /> : null}
      </div>
      {!isCollapsed && <span className="font-bold text-sm truncate">{label}</span>}
    </Link>
  );
}

function SidebarSection({
  item,
  isCollapsed,
  openSections,
  toggleSection,
  location,
}: {
  isCollapsed: boolean;
  item: MenuGroupType;
  location: { pathname: string };
  openSections: Partial<Record<string, boolean>>;
  toggleSection: (section: string) => void;
}) {
  return (
    <div className="space-y-2">
      {!isCollapsed && (
        <Button
          className="flex w-full items-center justify-between px-4 py-3 min-h-[44px] text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] hover:text-primary-600 dark:hover:text-primary-400 transition-colors group"
          data-testid={item.testId}
          onClick={() => { toggleSection(item.section); }}
          variant="ghost"
        >
          <div className="flex items-center gap-2">
            <item.sectionIcon className="h-3 w-3 opacity-50 group-hover:opacity-100" />
            <span>{item.label}</span>
          </div>
          {openSections[item.section] ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
        </Button>
      )}
      {(openSections[item.section] ?? isCollapsed) ? (
        <div className="space-y-1">
          {item.items.map((subItem) => (
            <SidebarLink
              icon={subItem.icon}
              isActive={location.pathname === subItem.to}
              isCollapsed={isCollapsed}
              key={subItem.to}
              label={subItem.label}
              testId={subItem.testId}
              to={subItem.to}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function useSidebarMenuItems(): MenuItem[] {
  const { dashboard, groups } = useFilteredMenuItems();
  return [dashboard, ...groups];
}

export default function Sidebar({ isOpen, onClose, openSections, toggleSection, isCollapsed }: SidebarProps) {
  const location = useLocation();
  const { workspace } = useWorkspace();

  const filteredMenuItems = useSidebarMenuItems();

  return (
    <div className={cn("fixed inset-y-0 left-0 z-50 flex flex-col bg-white dark:bg-slate-950 border-r border-slate-100 dark:border-slate-800 transition-all duration-300 shadow-xl shadow-slate-200/50 dark:shadow-black/30", isCollapsed ? "w-20" : "w-72", !isOpen && "-translate-x-full lg:translate-x-0")}>
      <div className={cn(
        "flex items-center px-6 border-b border-slate-50 dark:border-slate-800 shrink-0 relative",
        workspace?.logo_url && !isCollapsed ? "py-4 min-h-[6rem] items-start" : "h-24 items-center justify-between"
      )}>
        {!isCollapsed && (
          <div className="flex flex-col gap-1.5 w-full">
            {workspace?.logo_url ? (
              <div className="flex flex-col gap-1">
                <div 
                  aria-label={workspace.name || "Workspace Logo"}
                  className="h-12 w-full bg-contain bg-no-repeat bg-left drop-shadow-sm"
                  data-testid="sidebar-workspace-logo"
                  role="img"
                  style={{ backgroundImage: `url(${workspace.logo_url})` }}
                />
                { }
                {workspace?.tagline ? (
                  <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] line-clamp-1 mt-1" data-testid="sidebar-workspace-tagline">
                    {workspace.tagline}
                  </span>
                ) : null}
              </div>
            ) : (
              <div className="flex items-center gap-3">
                <div className="h-10 w-10 shrink-0 rounded-2xl bg-gradient-to-tr from-primary-600 to-primary-400 shadow-lg shadow-primary-200 dark:shadow-primary-900/20 flex items-center justify-center text-white font-black text-xl tracking-tighter italic pl-1">
                  {workspace?.name.charAt(0).toUpperCase() ?? "L"}
                </div>
                <div className="flex flex-col overflow-visible pr-4">
                  <span className="text-xl font-black tracking-tighter text-slate-800 dark:text-slate-200 uppercase italic truncate pl-1 pr-4">
                    {workspace?.name ?? <React.Fragment>Lee<span className="text-primary-600 dark:text-primary-400">Attend</span></React.Fragment>}
                  </span>
                  {workspace?.tagline ? (
                    <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] truncate" data-testid="sidebar-workspace-tagline">
                      {workspace.tagline}
                    </span>
                  ) : null}
                </div>
              </div>
            )}
          </div>
        )}

        {isCollapsed && workspace?.logo_url ? (
          <div className="w-full flex justify-center">
            <div 
              aria-label={workspace.name || "Logo"}
              className="w-10 h-10 bg-contain bg-no-repeat bg-center drop-shadow-sm"
              role="img"
              style={{ backgroundImage: `url(${workspace.logo_url})` }}
            />
          </div>
        ) : null}

        {isCollapsed && !workspace?.logo_url ? (
          <div className="w-full flex justify-center">
            <div className="h-10 w-10 shrink-0 rounded-2xl bg-gradient-to-tr from-primary-600 to-primary-400 shadow-lg shadow-primary-200 dark:shadow-primary-900/20 flex items-center justify-center text-white font-black text-xl tracking-tighter italic">
              {workspace?.name.charAt(0).toUpperCase() ?? "L"}
            </div>
          </div>
        ) : null}

        <Button 
          className={cn(
            "lg:hidden p-3 min-w-[44px] min-h-[44px] flex items-center justify-center text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 bg-slate-50 dark:bg-slate-800 rounded-xl transition-colors",
            workspace?.logo_url && !isCollapsed ? "absolute right-4 top-4" : ""
          )} 
          onClick={onClose}
          variant="ghost"
        >
          <X className="h-5 w-5" />
        </Button>
      </div>
      <div className="flex-1 overflow-y-auto custom-scrollbar px-4 py-6 space-y-8">
        {filteredMenuItems.map((item: MenuItem) => (
          <div key={item.label}>
            {"section" in item ? (
              <SidebarSection
                isCollapsed={isCollapsed}
                item={item}
                location={location}
                openSections={openSections}
                toggleSection={toggleSection}
              />
            ) : (
              <SidebarLink
                icon={item.icon}
                isActive={item.exact ? location.pathname === item.to : location.pathname.startsWith(item.to)}
                isCollapsed={isCollapsed}
                label={item.label}
                testId={item.testId}
                to={item.to}
              />
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

