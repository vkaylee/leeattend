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
  const description = widget.descriptionKey
    ? t(widget.descriptionKey, { defaultValue: widget.descriptionFallback })
    : widget.descriptionFallback;
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
      <div className="relative pb-3 mb-3 border-b border-slate-100 dark:border-slate-800/60">
        <div className="absolute left-0 top-0 flex h-8 w-8 items-center justify-center rounded-xl bg-primary-500/10 text-primary-600 dark:text-primary-400">
          <Icon className="w-4 h-4" />
        </div>

        <div className="min-w-0 px-9 text-center">
          <h2 className="break-words text-sm font-bold leading-tight text-slate-800 dark:text-slate-100">
            {title}
          </h2>
          <p className="line-clamp-2 text-[11px] leading-relaxed text-slate-500 dark:text-slate-400">
            {description}
          </p>
        </div>

        {/* Quick hide button */}
        {onHideWidget && (
          <Button
            type="button"
            variant="ghost"
            onClick={() => onHideWidget(widget.id)}
            aria-label={`${t("desktop.hideWidget", "Ẩn widget")} ${title}`}
            title={t("desktop.hideWidget", "Ẩn widget này")}
            className="absolute right-0 top-0 p-1.5 h-auto min-w-0 rounded-lg text-slate-400 dark:text-slate-500 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors focus-visible:ring-2 focus-visible:ring-primary"
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
