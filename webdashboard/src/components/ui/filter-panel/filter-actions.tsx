import { Check, X } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import {
  DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";

import type { FilterActionsProps } from "./types";

// ── Footer / Actions ────────────────────────────────────────────────────

function FilterFooter({
  onApply,
  onCancel,
  onClear,
  selectedCount,
  showApplyButton,
  showSelectionCount,
  totalCount,
}: FilterActionsProps) {
  const { t } = useTranslation();
  const showFooter = showApplyButton || showSelectionCount;
  if (!showFooter) return null;

  return (
    <div className="pt-3 mt-1 border-t border-slate-50 dark:border-slate-800 flex items-center justify-between">
      {showSelectionCount ? (
        <span className="text-[10px] font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider">
          {selectedCount > 0
            ? t("common.selected_count", { selected: selectedCount, total: totalCount })
            : t("common.all_selected", "Tất cả")}
        </span>
      ) : (
        <span />
      )}

      {showApplyButton ? (
        <div className="flex gap-2">
          <Button onClick={onClear} size="sm" variant="ghost">
            {t("common.clear")}
          </Button>
          <Button onClick={onCancel} size="sm" variant="ghost">
            {t("common.cancel")}
          </Button>
          <Button
            icon={<Check size={12} />}
            onClick={onApply}
            size="sm"
            variant="primary"
          >
            {t("common.apply")}
          </Button>
        </div>
      ) : selectedCount > 0 ? (
        <Button
          className="text-[10px] font-bold text-red-500 dark:text-red-400 hover:underline uppercase tracking-wide cursor-pointer h-auto p-0"
          onClick={onClear}
          variant="ghost"
        >
          {t("common.clear")}
        </Button>
      ) : null}
    </div>
  );
}

// ── Dropdown Actions (inline, no footer wrapper) ────────────────────────

interface DropdownActionsProps {
  effectiveSelected: string[];
  onApply?: () => void;
  onCancel: () => void;
  onClear: () => void;
  showApplyButton?: boolean;
  testId?: string;
}

export function DropdownActions({
  onApply,
  onCancel,
  onClear,
  effectiveSelected,
  showApplyButton,
  testId,
}: DropdownActionsProps) {
  const { t } = useTranslation();

  if (showApplyButton) {
    return (
      <div className="pt-2 mt-1 border-t border-slate-100 dark:border-slate-800 flex justify-end gap-2">
        <Button onClick={onCancel} size="sm" variant="ghost">
          {t("common.cancel")}
        </Button>
        <Button
          icon={<Check size={12} />}
          onClick={onApply}
          size="sm"
          variant="primary"
        >
          {t("common.apply")}
        </Button>
      </div>
    );
  }

  if (effectiveSelected.length > 0) {
    return (
      <>
        <DropdownMenuSeparator />
        <Button
          className="h-auto w-full px-3 py-2.5 flex items-center gap-3 rounded-xl text-[11px] font-bold uppercase tracking-wide text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors border-none shadow-none justify-start"
          data-testid={testId ? `${testId}-clear` : undefined}
          onClick={(e) => { e.preventDefault(); e.stopPropagation(); onClear(); }}
          type="button"
          variant="ghost"
        >
          <X className="shrink-0" size={12} />
          {t("common.clear_filter", "Xóa bộ lọc")}
        </Button>
      </>
    );
  }

  return null;
}

export { FilterFooter };
