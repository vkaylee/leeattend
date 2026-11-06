import { SlidersHorizontal, RotateCcw, Check, Plus, Minus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";

import type { DesktopWidgetDefinition } from "./types";

interface DesktopCustomizerModalProps {
  isOpen: boolean;
  onClose: () => void;
  availableWidgets: DesktopWidgetDefinition[];
  isWidgetEnabled: (id: string) => boolean;
  onToggleWidget: (id: string) => void;
  onResetToDefault: () => void;
}

export function DesktopCustomizerModal({
  isOpen,
  onClose,
  availableWidgets,
  isWidgetEnabled,
  onToggleWidget,
  onResetToDefault,
}: DesktopCustomizerModalProps) {
  const { t } = useTranslation();

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={t("desktop.customizeTitle", "Tùy biến Desktop & Widget")}
      subtitle={t("desktop.customizeSubtitle", "Bật hoặc tắt các widget hiển thị trên màn hình chính của bạn")}
      icon={<SlidersHorizontal className="w-5 h-5 text-primary-500" />}
      maxWidth="lg"
      testId="desktop-customizer-modal"
    >
      <div className="space-y-6 pt-2">
        {/* Widgets List */}
        <div className="space-y-3 max-h-[60vh] overflow-y-auto pr-1">
          {availableWidgets.map((widget) => {
            const Icon = widget.icon;
            const enabled = isWidgetEnabled(widget.id);

            return (
              <div
                key={widget.id}
                className={`flex items-center justify-between p-3.5 rounded-2xl border transition-all ${
                  enabled
                    ? "bg-primary-50/30 dark:bg-primary-950/20 border-primary-500/30"
                    : "bg-slate-50/50 dark:bg-slate-900/30 border-slate-200/60 dark:border-slate-800/60 opacity-75"
                }`}
              >
                <div className="flex items-center gap-3 min-w-0 pr-4">
                  <div
                    className={`w-9 h-9 rounded-xl flex items-center justify-center shrink-0 ${
                      enabled
                        ? "bg-primary-500 text-white shadow-xs"
                        : "bg-slate-200 dark:bg-slate-800 text-slate-500"
                    }`}
                  >
                    <Icon className="w-4 h-4" />
                  </div>
                  <div className="min-w-0">
                    <div className="text-xs font-bold text-slate-800 dark:text-slate-100 truncate">
                      {widget.titleFallback}
                    </div>
                    <div className="text-[11px] text-slate-500 dark:text-slate-400 line-clamp-1">
                      {widget.descriptionFallback}
                    </div>
                  </div>
                </div>

                {/* Toggle Button */}
                <Button
                  type="button"
                  variant={enabled ? "primary" : "outline"}
                  onClick={() => onToggleWidget(widget.id)}
                  className={`h-8 px-3 rounded-xl text-xs font-semibold shrink-0 flex items-center gap-1.5 ${
                    enabled
                      ? "bg-primary-600 hover:bg-primary-700 text-white"
                      : "text-slate-600 dark:text-slate-300"
                  }`}
                  data-testid={`toggle-widget-${widget.id}`}
                >
                  {enabled ? (
                    <>
                      <Check className="w-3.5 h-3.5" />
                      <span>{t("desktop.enabled", "Đã bật")}</span>
                    </>
                  ) : (
                    <>
                      <Plus className="w-3.5 h-3.5" />
                      <span>{t("desktop.add", "Gắn vào")}</span>
                    </>
                  )}
                </Button>
              </div>
            );
          })}
        </div>

        {/* Footer Actions */}
        <div className="flex items-center justify-between pt-4 border-t border-slate-100 dark:border-slate-800">
          <Button
            type="button"
            variant="ghost"
            onClick={onResetToDefault}
            className="text-xs font-semibold text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 flex items-center gap-1.5"
            data-testid="reset-widgets-btn"
          >
            <RotateCcw className="w-3.5 h-3.5" />
            <span>{t("desktop.resetDefault", "Khôi phục mặc định")}</span>
          </Button>

          <Button
            type="button"
            variant="primary"
            onClick={onClose}
            className="px-5 rounded-xl text-xs font-bold"
          >
            {t("common.done", "Xong")}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
