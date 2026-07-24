/* eslint-disable @typescript-eslint/no-unnecessary-condition -- CHART_METADATA Record lookup may return undefined for runtime widget IDs */
import { Search, ChevronLeft, ChevronRight } from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import type { WidgetId } from "@/hooks/use-dashboard-layout";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import {
  CATEGORY_LABEL_KEYS,
  CATEGORY_ORDER,
  CHART_METADATA,
  type ChartCategory,
} from "@/lib/chart-category-metadata";
import { cn } from "@/lib/utils";

const STORAGE_KEY = "dashboard-catalog-collapsed";

function useCollapsedState() {
  const [collapsed, setCollapsed] = useState<boolean>(() => {
    try {
      return localStorage.getItem(STORAGE_KEY) === "true";
    } catch {
      return false;
    }
  });

  const toggle = useCallback(() => {
    setCollapsed((prev) => {
      const next = !prev;
      try {
        localStorage.setItem(STORAGE_KEY, String(next));
      } catch {
        // ignore
      }
      return next;
    });
  }, []);

  return { collapsed, toggle };
}

interface ChartCatalogSidebarProps {
  allWidgets: { id: WidgetId; labelKey: string }[];
  onToggleWidget: (id: WidgetId) => void;
  visibleWidgets: Record<WidgetId, boolean>;
}

export default function ChartCatalogSidebar({
  allWidgets,
  visibleWidgets,
  onToggleWidget,
}: ChartCatalogSidebarProps) {
  const { t } = useTranslation();
  const { collapsed, toggle } = useCollapsedState();
  const [searchQuery, setSearchQuery] = useState("");

  const widgetMap = useMemo(() => {
    const map = new Map<WidgetId, { id: WidgetId; labelKey: string }>();
    allWidgets.forEach((w) => map.set(w.id, w));
    return map;
  }, [allWidgets]);

  const filteredWidgets = useMemo(() => {
    const query = searchQuery.toLowerCase().trim();
    const grouped: Record<ChartCategory, WidgetId[]> = {
      attendance: [],
      workforce: [],
      performance: [],
      summary: [],
    };

    CATEGORY_ORDER.forEach((cat) => {
      grouped[cat] = allWidgets
        .filter((w) => {
          const meta = CHART_METADATA[w.id];
           
          if (meta?.category !== cat) return false;
          if (!query) return true;
          const label = t(w.labelKey).toLowerCase();
          return label.includes(query);
        })
        .map((w) => w.id);
    });

    return grouped;
  }, [allWidgets, searchQuery, t]);

  const totalResults = useMemo(
    () => CATEGORY_ORDER.reduce((sum, cat) => sum + filteredWidgets[cat].length, 0),
    [filteredWidgets],
  );

  return (
    <aside
      className={cn(
        "hidden lg:flex flex-col shrink-0",
        "sticky top-0 self-start",
        "h-[calc(100vh-8rem)]",
        "bg-white dark:bg-slate-900",
        "border border-slate-100 dark:border-slate-800",
        "rounded-2xl shadow-sm",
        "transition-all duration-300",
        collapsed ? "w-14" : "w-72",
      )}
    >
      {/* Header */}
      {!collapsed && (
        <div className="px-4 pt-4 pb-2 space-y-3 shrink-0">
          <Typography
            className="text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest"
            variant="small"
          >
            {t("dashboard.catalog.title")}
          </Typography>

          {/* Search */}
          <div className="relative">
            <Search
              className="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400 dark:text-slate-500"
              size={14}
            />
            <input
              aria-label={t("dashboard.catalog.search_placeholder")}
              className={cn(
                "w-full pl-8 pr-3 py-2 rounded-xl text-xs",
                "bg-slate-50 dark:bg-slate-800",
                "border border-slate-200 dark:border-slate-700",
                "text-slate-700 dark:text-slate-200",
                "placeholder:text-slate-400 dark:placeholder:text-slate-500",
                "focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500",
                "transition-colors",
              )}
              onChange={(e) => { setSearchQuery(e.target.value); }}
              placeholder={t("dashboard.catalog.search_placeholder")}
              type="text"
              value={searchQuery}
            />
          </div>
        </div>
      )}

      {/* Widget list */}
      <div className="flex-1 overflow-y-auto px-2 py-2 space-y-1 overscroll-contain">
        {CATEGORY_ORDER.map((cat) => {
          const widgets = filteredWidgets[cat];
          if (widgets.length === 0) return null;

          return (
            <div key={cat}>
              {/* Category header */}
              {!collapsed && (
                <Typography
                  className="px-3 pt-3 pb-1.5 text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-widest"
                  id={`catalog-cat-${cat}`}
                  variant="small"
                >
                  {t(CATEGORY_LABEL_KEYS[cat])}
                </Typography>
              )}

              <section aria-labelledby={`catalog-cat-${cat}`}>

                {widgets.map((widgetId) => {
                  const widget = widgetMap.get(widgetId);
                  if (!widget) return null;

                  const meta = CHART_METADATA[widgetId];
                  const Icon = meta?.icon;
                  const isVisible = visibleWidgets[widgetId];
                  const label = t(widget.labelKey);

                  return (
                    // eslint-disable-next-line react/forbid-elements -- role="switch" button with ARIA needs native button semantics
                    <button
                      aria-checked={isVisible}
                      className={cn(
                        "w-full flex items-center gap-3 rounded-xl transition-all duration-150",
                        collapsed ? "justify-center px-0 py-2.5" : "px-3 py-2",
                        isVisible
                          ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400"
                          : "text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800/50 hover:text-slate-700 dark:hover:text-slate-200",
                      )}
                      key={widgetId}
                      onClick={() => { onToggleWidget(widgetId); }}
                      role="switch"
                      tabIndex={0}
                      title={collapsed ? label : undefined}
                      type="button"
                    >
                      {/* Checkbox circle */}
                      <span
                        className={cn(
                          "shrink-0 w-4 h-4 rounded-full border-2 flex items-center justify-center transition-colors duration-150",
                          isVisible
                            ? "border-primary-600 dark:border-primary-400 bg-primary-600 dark:bg-primary-400"
                            : "border-slate-300 dark:border-slate-600",
                        )}
                      >
                        {isVisible ? (
                          <svg
                            className="w-2.5 h-2.5 text-white"
                            fill="none"
                            stroke="currentColor"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={3}
                            viewBox="0 0 24 24"
                          >
                            <polyline points="20 6 9 17 4 12" />
                          </svg>
                        ) : null}
                      </span>

                      {!collapsed && !!Icon && <Icon className="shrink-0" size={15} />}

                      {!collapsed && (
                        <span className="flex-1 text-left text-xs font-medium truncate">
                          {label}
                        </span>
                      )}
                    </button>
                  );
                })}
              </section>
            </div>
          );
        })}

        {/* Empty search state */}
        {!collapsed && totalResults === 0 && (
          <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
            <Search className="text-slate-300 dark:text-slate-600 mb-3" size={28} />
            <Typography
              className="text-xs text-slate-400 dark:text-slate-500 font-medium"
              variant="small"
            >
              {t("dashboard.catalog.no_results")}
            </Typography>
          </div>
        )}
      </div>

      {/* Collapse button */}
      <div className="shrink-0 px-2 pb-2 pt-1 border-t border-slate-100 dark:border-slate-800">
        <Button
          aria-label={collapsed ? t("dashboard.catalog.expand") : t("dashboard.catalog.collapse")}
          className={cn(
            "w-full flex items-center gap-2 py-2 rounded-xl",
            "text-xs font-medium text-slate-400 dark:text-slate-500",
            "hover:text-slate-600 dark:hover:text-slate-300",
            "hover:bg-slate-50 dark:hover:bg-slate-800",
            "transition-colors h-auto shadow-none",
            collapsed ? "justify-center px-0" : "justify-start px-3",
          )}
          onClick={toggle}
          size="icon"
          variant="ghost"
        >
          {collapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
          {!collapsed && (
            <span className="text-[10px] font-bold uppercase tracking-widest">
              {t("dashboard.catalog.collapse")}
            </span>
          )}
        </Button>
      </div>
    </aside>
  );
}
