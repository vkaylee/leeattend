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
  id: string;
  testId: string;
  titleKey: string;
  defaultTitle: string;
  items: SettingItem[];
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
      data-testid="settings-sidebar"
      className={cn(
        "flex flex-col bg-slate-50/70 dark:bg-slate-900/70 backdrop-blur-xl border border-slate-200/60 dark:border-slate-800/60 rounded-3xl p-4 md:p-5 h-full overflow-hidden shrink-0",
        className
      )}
    >
      {/* 1. Header */}
      <div className="pb-4 mb-4 border-b border-slate-200/60 dark:border-slate-800/60 flex items-center justify-between gap-3">
        <div className="flex items-center gap-3 min-w-0">
          <div className="p-2.5 bg-primary-50 dark:bg-primary-950/80 rounded-2xl border border-primary-100 dark:border-primary-900/50 text-primary-600 dark:text-primary-400 shrink-0">
            <Settings className="w-5 h-5" />
          </div>
          <div className="min-w-0">
            <Typography
              variant="h2"
              className="text-lg font-black text-slate-800 dark:text-slate-100 truncate"
            >
              {t("common.settings.title", "Cài đặt")}
            </Typography>
            <Typography
              variant="p"
              className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest truncate"
            >
              {t("settings.layout.control_panel", "Bảng điều khiển")}
            </Typography>
          </div>
        </div>

        <div className="hidden sm:flex items-center gap-1.5 px-2.5 py-1 rounded-xl bg-slate-100 dark:bg-slate-800/80 border border-slate-200/60 dark:border-slate-700/60 shrink-0">
          <ShieldCheck className="w-3.5 h-3.5 text-slate-400 dark:text-slate-500" />
          <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
            {t("settings.admin_only", "Admin")}
          </span>
        </div>
      </div>

      {/* 2. Fast Filter / Search Input */}
      <div className="relative mb-4">
        <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 pointer-events-none" />
        <input
          type="text"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          placeholder={t(
            "settings.searchPlaceholder",
            "Tìm cài đặt & tính năng..."
          )}
          data-testid="settings-search-input"
          className="w-full pl-9 pr-8 py-2 text-xs rounded-xl bg-white dark:bg-slate-950/60 border border-slate-200/80 dark:border-slate-800/80 placeholder:text-slate-400 text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all"
        />
        {searchTerm && (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => setSearchTerm("")}
            className="absolute right-1.5 top-1/2 -translate-y-1/2 h-6 w-6 p-0 text-slate-400 hover:text-slate-600 rounded-lg"
          >
            <X className="w-3.5 h-3.5" />
          </Button>
        )}
      </div>

      {/* 3. Categorized Items List */}
      <div className="flex-1 overflow-y-auto overscroll-contain custom-scrollbar space-y-5 pr-1">
        {sections.map((section) => (
          <div
            key={section.id}
            data-testid={section.testId}
            className="space-y-1"
          >
            <div className="px-2 py-1 text-[11px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider">
              {t(section.titleKey, section.defaultTitle)}
            </div>

            <div className="space-y-0.5">
              {section.items.map((item) => {
                const isActive =
                  location.pathname === item.path ||
                  location.pathname.startsWith(item.path + "/");

                return (
                  <Link
                    key={item.path}
                    to={item.path as "/"}
                    className={cn(
                      "flex items-center gap-3 px-3 py-2.5 rounded-2xl text-xs font-semibold transition-all duration-200 group relative",
                      isActive
                        ? "bg-primary-50 dark:bg-primary-950/80 text-primary-600 dark:text-primary-400 shadow-2xs font-bold border-l-2 border-primary-500"
                        : "text-slate-600 dark:text-slate-400 hover:bg-white dark:hover:bg-slate-800 hover:text-slate-900 dark:hover:text-slate-100"
                    )}
                  >
                    <div
                      className={cn(
                        "w-7 h-7 rounded-xl flex items-center justify-center shrink-0 transition-colors",
                        isActive
                          ? "bg-primary-100 dark:bg-primary-900/60 text-primary-600 dark:text-primary-400"
                          : "bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 group-hover:text-primary-600 dark:group-hover:text-primary-400"
                      )}
                    >
                      {item.icon}
                    </div>

                    <div className="min-w-0 flex-1">
                      <div className="truncate leading-tight">{item.label}</div>
                      {item.description && (
                        <div className="text-[10px] text-slate-400 dark:text-slate-500 truncate font-normal">
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
