import { EyeOff } from "lucide-react";
import React, { memo } from "react";
import { useTranslation } from "react-i18next";
import Button from "@/components/ui/button";
import type { DesktopWidgetDefinition } from "./types";
import { cn } from "@/lib/utils";

interface DesktopWidgetCardProps {
  widget: DesktopWidgetDefinition;
  onHideWidget?: (id: string) => void;
  className?: string;
}

export const DesktopWidgetCard = memo(function DesktopWidgetCard({
  widget,
  onHideWidget,
  className,
}: DesktopWidgetCardProps) {
  const { t } = useTranslation();
  const Icon = widget.icon;
  const Component = widget.component;

  const title = widget.titleKey
    ? t(widget.titleKey, { defaultValue: widget.titleFallback })
    : widget.titleFallback;

  return (
    <section
      data-testid={`desktop-widget-${widget.id}`}
      aria-label={title}
      className={cn(
        "flex flex-col justify-between p-5 rounded-3xl bg-card border border-slate-200/80 dark:border-slate-800/80",
        "shadow-xs hover:shadow-md transition-all duration-200",
        widget.colSpan,
        className,
      )}
    >
      {/* Widget Header */}
      <div className="flex items-center justify-between pb-3 mb-3 border-b border-slate-100 dark:border-slate-800/60">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-xl bg-primary-500/10 text-primary-600 dark:text-primary-400 flex items-center justify-center shrink-0">
            <Icon className="w-4 h-4" />
          </div>
          <div>
            <h2 className="text-sm font-bold text-slate-800 dark:text-slate-100 leading-tight">
              {title}
            </h2>
            <p className="text-[11px] text-slate-400 line-clamp-1">
              {widget.descriptionFallback}
            </p>
          </div>
        </div>

        {/* Quick hide button */}
        {onHideWidget && (
          <Button
            type="button"
            variant="ghost"
            onClick={() => onHideWidget(widget.id)}
            aria-label={`${t("desktop.hideWidget", "Ẩn widget")} ${title}`}
            title={t("desktop.hideWidget", "Ẩn widget này")}
            className="p-1.5 h-auto min-w-0 rounded-lg text-slate-300 dark:text-slate-600 hover:text-slate-500 dark:hover:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          >
            <EyeOff className="w-3.5 h-3.5" />
          </Button>
        )}
      </div>

      {/* Widget Body */}
      <div className="flex-1 min-h-[140px] flex flex-col justify-center">
        <Component />
      </div>
    </section>
  );
});
