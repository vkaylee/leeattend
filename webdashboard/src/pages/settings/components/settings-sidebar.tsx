import { Link, useLocation } from "@tanstack/react-router";
import {
  Search,
  Settings,
  ShieldCheck,
  X,
} from "lucide-react";
import React, { memo, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { cn, removeDiacritics } from "@/lib/utils";

import { SettingItem, useSettingsNav } from "../hooks/use-settings-nav";

interface SettingsSidebarProps {
  className?: string;
}

interface SectionDefinition {
  defaultTitle: string;
  id: string;
  items: SettingItem[];
  testId: string;
  titleKey: string;
}

export const SettingsSidebar = memo(function SettingsSidebar({
  className,
}: SettingsSidebarProps) {
  const { t } = useTranslation();
  const location = useLocation();
  const [searchTerm, setSearchTerm] = useState("");

  const { workspaceGroup, accessGroup, operationsGroup, systemGroup } =
    useSettingsNav();

  // Normalize search for Vietnamese diacritics
  const cleanSearch = useMemo(
    () => removeDiacritics(searchTerm.trim().toLowerCase()),
    [searchTerm]
  );

  const filterGroup = (items: SettingItem[]): SettingItem[] => {
    if (!cleanSearch) return items;
    return items.filter((item) => {
      const labelMatch = removeDiacritics(item.label.toLowerCase()).includes(
        cleanSearch
      );
      const descMatch = removeDiacritics(
        (item.description || "").toLowerCase()
      ).includes(cleanSearch);
      return labelMatch || descMatch;
    });
  };

  const sections: SectionDefinition[] = useMemo(() => {
    return [
      {
        id: "workspace",
        testId: "settings-section-workspace",
        titleKey: "settings.categories.workspace",
        defaultTitle: "Không gian làm việc",
        items: filterGroup(workspaceGroup),
      },
      {
        id: "access",
        testId: "settings-section-access",
        titleKey: "settings.categories.access",
        defaultTitle: "Phân quyền & Tài khoản",
        items: filterGroup(accessGroup),
      },
      {
        id: "operations",
        testId: "settings-section-operations",
        titleKey: "settings.categories.operations",
        defaultTitle: "Vận hành & Nghiệp vụ",
        items: filterGroup(operationsGroup),
      },
      {
        id: "system",
        testId: "settings-section-system",
        titleKey: "settings.categories.system",
        defaultTitle: "Bảo mật & Hệ thống",
        items: filterGroup(systemGroup),
      },
    ].filter((s) => s.items.length > 0);
  }, [workspaceGroup, accessGroup, operationsGroup, systemGroup, cleanSearch]);

  return (
    <aside
      aria-label={t("common.settings.title", "Cài đặt")}
      className={cn(
        "flex flex-col bg-slate-50/80 dark:bg-slate-900/80 backdrop-blur-xl border border-slate-200/80 dark:border-slate-800/80 rounded-2xl p-3.5 md:p-4 h-full overflow-hidden shrink-0 shadow-2xs",
        className
      )}
      data-testid="settings-sidebar"
    >
      {/* 1. Header (macOS Ventura/Sonoma style) */}
      <div className="pb-3.5 mb-3 border-b border-slate-200/60 dark:border-slate-800/60 flex items-center justify-between gap-3">
        <div className="flex items-center gap-2.5 min-w-0">
          <div className="w-8 h-8 rounded-lg bg-primary-600 text-white flex items-center justify-center shrink-0 shadow-2xs">
            <Settings className="w-4 h-4" />
          </div>
          <div className="min-w-0">
            <Typography
              className="text-sm font-bold text-slate-800 dark:text-slate-100 truncate leading-snug"
              variant="h2"
            >
              {t("common.settings.title", "Cài đặt")}
            </Typography>
            <Typography
              className="text-[10px] font-semibold text-slate-400 dark:text-slate-500 uppercase tracking-wider truncate"
              variant="p"
            >
              {t("settings.layout.control_panel", "Bảng điều khiển")}
            </Typography>
          </div>
        </div>

        <div className="hidden sm:flex items-center gap-1 px-2 py-0.5 rounded-full bg-slate-100 dark:bg-slate-800 border border-slate-200/60 dark:border-slate-700/60 shrink-0">
          <ShieldCheck className="w-3 h-3 text-slate-400 dark:text-slate-500" />
          <span className="text-[10px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
            {t("settings.admin_only", "Admin")}
          </span>
        </div>
      </div>

      {/* 2. Fast Filter / Search Input */}
      <div className="relative mb-3">
        <Search className="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400 pointer-events-none" />
        <input
          className="w-full pl-8 pr-7 py-1.5 text-xs rounded-xl bg-white dark:bg-slate-950/60 border border-slate-200/80 dark:border-slate-800/80 placeholder:text-slate-400 text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all shadow-2xs"
          data-testid="settings-search-input"
          onChange={(e) => setSearchTerm(e.target.value)}
          placeholder={t(
            "settings.searchPlaceholder",
            "Tìm cài đặt & tính năng..."
          )}
          type="text"
          value={searchTerm}
        />
        {searchTerm && (
          <Button
            className="absolute right-1 top-1/2 -translate-y-1/2 h-5 w-5 p-0 text-slate-400 hover:text-slate-600 rounded-md"
            onClick={() => setSearchTerm("")}
            size="sm"
            type="button"
            variant="ghost"
          >
            <X className="w-3 h-3" />
          </Button>
        )}
      </div>

      {/* 3. Categorized Items List (Apple Inset Menu) */}
      <div className="flex-1 overflow-y-auto overscroll-contain custom-scrollbar space-y-4 pr-0.5">
        {sections.map((section) => (
          <div
            className="space-y-0.5"
            data-testid={section.testId}
            key={section.id}
          >
            <div className="px-2 py-1 text-[11px] font-semibold text-slate-400 dark:text-slate-500 uppercase tracking-wider select-none">
              {t(section.titleKey, section.defaultTitle)}
            </div>

            <div className="space-y-0.5">
              {section.items.map((item) => {
                const isActive =
                  location.pathname === item.path ||
                  location.pathname.startsWith(item.path + "/");

                return (
                  <Link
                    className={cn(
                      "flex items-center gap-2.5 px-2.5 py-2 rounded-xl text-xs transition-all duration-150 group relative select-none",
                      isActive
                        ? "bg-primary-600 text-white font-semibold shadow-2xs"
                        : "text-slate-700 dark:text-slate-300 hover:bg-slate-200/60 dark:hover:bg-slate-800/70"
                    )}
                    key={item.path}
                    to={item.path as "/"}
                  >
                    <div
                      className={cn(
                        "w-6 h-6 rounded-lg flex items-center justify-center shrink-0 transition-colors",
                        isActive
                          ? "bg-white/20 text-white"
                          : "bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 group-hover:text-primary-600 dark:group-hover:text-primary-400"
                      )}
                    >
                      {item.icon}
                    </div>

                    <div className="min-w-0 flex-1">
                      <div className="truncate leading-tight font-medium">
                        {item.label}
                      </div>
                      {item.description && (
                        <div
                          className={cn(
                            "text-[10px] truncate leading-normal",
                            isActive
                              ? "text-white/80 font-normal"
                              : "text-slate-400 dark:text-slate-500"
                          )}
                        >
                          {item.description}
                        </div>
                      )}
                    </div>
                  </Link>
                );
              })}
            </div>
          </div>
        ))}

        {sections.length === 0 && (
          <div className="text-center py-8 text-xs text-slate-400">
            {t("common.no_results", "Không tìm thấy cài đặt phù hợp")}
          </div>
        )}
      </div>
    </aside>
  );
});

export default SettingsSidebar;
