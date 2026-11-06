import { LayoutGrid, PlusCircle } from "lucide-react";
import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";

import {
  DESKTOP_WIDGET_REGISTRY,
  filterDesktopWidgets,
  useDesktopLayout,
  DesktopHeader,
  DesktopWidgetCard,
  DesktopCustomizerModal,
} from "@/components/features/desktop";
import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import { Skeleton } from "@/components/ui/skeleton";
import { useMyPermissions } from "@/hooks/use-permissions";

export default function DesktopPage() {
  const { t } = useTranslation();
  const { hasAnyPermission, isLoading: permsLoading } = useMyPermissions();

  const {
    enabledIds,
    order,
    isCustomizing,
    toggleWidget,
    isWidgetEnabled,
    resetToDefault,
    openCustomizer,
    closeCustomizer,
  } = useDesktopLayout();

  // Filter available widgets based on standalone permissions.
  const availableWidgets = useMemo(() => {
    if (permsLoading) return [];
    return filterDesktopWidgets(DESKTOP_WIDGET_REGISTRY, hasAnyPermission);
  }, [permsLoading, hasAnyPermission]);

  // Order active enabled widgets
  const activeWidgets = useMemo(() => {
    const enabledMap = new Map(availableWidgets.map((w) => [w.id, w]));

    // First sort according to custom order
    const ordered: typeof availableWidgets = [];
    for (const id of order) {
      if (enabledIds.includes(id) && enabledMap.has(id)) {
        ordered.push(enabledMap.get(id)!);
        enabledMap.delete(id);
      }
    }

    // Add any remaining enabled widgets not present in order array
    for (const [id, widget] of enabledMap.entries()) {
      if (enabledIds.includes(id)) {
        ordered.push(widget);
      }
    }

    return ordered;
  }, [availableWidgets, enabledIds, order]);

  // Skeleton loading state
  if (permsLoading) {
    return (
      <div className="space-y-6 animate-in fade-in duration-500" data-testid="desktop-skeleton">
        <div className="flex items-center justify-between">
          <Skeleton className="h-10 w-64 rounded-2xl" />
          <Skeleton className="h-10 w-48 rounded-2xl" />
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
          {Array.from({ length: 6 }).map((_, i) => (
            <Skeleton key={i} className="h-56 rounded-3xl" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6 animate-in fade-in duration-700" data-testid="desktop-page">
      {/* Desktop Header: Greetings, Spotlight Search & Widget Manager Trigger */}
      <DesktopHeader onOpenCustomizer={openCustomizer} />

      {/* Main Desktop Canvas */}
      {activeWidgets.length === 0 ? (
        <div
          data-testid="desktop-empty-state"
          className="flex items-center justify-center min-h-[50vh] p-8 rounded-3xl border border-dashed border-slate-200 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-900/20"
        >
          <EmptyState
            icon={<LayoutGrid className="w-10 h-10 text-primary-500" />}
            title={t("desktop.emptyTitle", "Màn hình chưa có Widget nào")}
            description={t(
              "desktop.emptyDesc",
              "Bạn đã ẩn tất cả các widget. Bấm vào nút bên dưới để chọn và gắn các widget hữu ích lên Desktop.",
            )}
            action={
              <Button
                type="button"
                variant="primary"
                onClick={openCustomizer}
                className="mt-4 flex items-center gap-2 rounded-2xl font-bold px-5"
                data-testid="desktop-empty-add-widget-btn"
              >
                <PlusCircle className="w-4 h-4" />
                <span>{t("desktop.attachWidgetsNow", "Gắn Widget vào Desktop")}</span>
              </Button>
            }
          />
        </div>
      ) : (
        <div
          data-testid="desktop-widget-grid"
          className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 items-stretch"
        >
          {activeWidgets.map((widget) => (
            <DesktopWidgetCard
              key={widget.id}
              widget={widget}
              onHideWidget={toggleWidget}
            />
          ))}
        </div>
      )}

      {/* Widget Customization Dialog */}
      <DesktopCustomizerModal
        isOpen={isCustomizing}
        onClose={closeCustomizer}
        availableWidgets={availableWidgets}
        isWidgetEnabled={isWidgetEnabled}
        onToggleWidget={toggleWidget}
        onResetToDefault={resetToDefault}
      />
    </div>
  );
}
