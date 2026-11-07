import {
  ArrowLeft,
  ArrowRight,
  Calculator,
  CheckCircle2,
  Clock,
  Lock,
  Unlock,
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { MonthYearPicker } from "@/components/common/month-year-picker";
import Button from "@/components/ui/button";
import ConfirmDialog from "@/components/ui/confirm-dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import PageHeader from "@/components/ui/page-header";
import { cn } from "@/lib/utils";

interface PayrollHeaderProps {
  calcMonth: number;
  calcYear: number;
  isLocked?: boolean;
  lifecycleStatus?: "DRAFT" | "CALCULATED" | "LOCKED";
  lockedAt?: string | null;
  lockedBy?: string | null;
  onChangeMonth: (delta: number) => void;
  onMonthYearChange: (month: number, year: number) => void;
  onOpenBatchCalc: () => void;
  onToggleLock?: () => void;
  selectedCount: number;
}

export const PayrollHeader: React.FC<PayrollHeaderProps> = ({
  calcMonth,
  calcYear,
  isLocked = false,
  lifecycleStatus = "DRAFT",
  lockedAt,
  lockedBy,
  onChangeMonth,
  onMonthYearChange,
  onOpenBatchCalc,
  onToggleLock,
  selectedCount,
}) => {
  const { t } = useTranslation();
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);

  const getStatusBadge = () => {
    if (isLocked || lifecycleStatus === "LOCKED") {
      return (
        <div
          className="flex items-center gap-1.5 px-3 py-1 rounded-xl bg-rose-500/10 text-rose-700 dark:text-rose-300 border border-rose-500/20 text-[11px] font-bold tracking-wider uppercase"
          title={
            lockedBy
              ? t("payroll.lifecycle.locked_by", {
                  name: lockedBy,
                  time: lockedAt ? new Date(lockedAt).toLocaleDateString("vi-VN") : "",
                })
              : t("payroll.lifecycle.locked_tooltip")
          }
        >
          <Lock className="w-3.5 h-3.5 text-rose-600 dark:text-rose-400" />
          <span>{t("payroll.lifecycle.locked_badge")}</span>
        </div>
      );
    }

    if (lifecycleStatus === "CALCULATED") {
      return (
        <div className="flex items-center gap-1.5 px-3 py-1 rounded-xl bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border border-emerald-500/20 text-[11px] font-bold tracking-wider uppercase">
          <CheckCircle2 className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
          <span>{t("payroll.lifecycle.calculated")}</span>
        </div>
      );
    }

    return (
      <div className="flex items-center gap-1.5 px-3 py-1 rounded-xl bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-slate-700 text-[11px] font-semibold tracking-wider uppercase">
        <Clock className="w-3.5 h-3.5 text-slate-500" />
        <span>{t("payroll.lifecycle.draft")}</span>
      </div>
    );
  };

  return (
    <>
      <PageHeader
        actions={
          <div className="flex flex-wrap items-center gap-3 bg-white dark:bg-slate-900 p-2 rounded-xl border border-slate-200 dark:border-slate-700 shadow-sm">
            {getStatusBadge()}

            <div className="flex items-center bg-slate-100 dark:bg-slate-700/50 rounded-xl p-1 shadow-inner border border-slate-200 dark:border-slate-700/50">
              <Button
                aria-label={t("payroll.prev_period", "Kỳ trước")}
                className="w-10 h-10 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-sm rounded-xl transition-all"
                onClick={() => {
                  onChangeMonth(-1);
                }}
                variant="ghost"
              >
                <ArrowLeft size={18} />
              </Button>

              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    className="px-6 flex flex-col items-center min-w-[150px] cursor-pointer hover:bg-white dark:hover:bg-slate-900 hover:shadow-sm rounded-xl transition-all py-1 group outline-none border border-transparent hover:border-slate-100 dark:border-slate-800"
                    variant="ghost"
                  >
                    <span className="text-[9px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest leading-none mb-0.5 group-hover:text-primary-500 dark:group-hover:text-primary-400 transition-colors">
                      {t("payroll.table.period")}
                    </span>
                    <span className="text-sm font-bold text-slate-700 dark:text-slate-200 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors uppercase tracking-tighter">
                      {calcMonth} / {calcYear}
                    </span>
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent
                  align="center"
                  className="w-80 p-4 rounded-[2rem] shadow-2xl border-slate-100 dark:border-slate-800"
                  sideOffset={8}
                >
                  <DropdownMenuLabel className="text-center text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 mb-4">
                    {t("common.select_month", { defaultValue: "CHỌN KỲ LƯƠNG" })}
                  </DropdownMenuLabel>
                  <MonthYearPicker
                    month={calcMonth}
                    onChange={(m, y) => {
                      onMonthYearChange(m, y);
                    }}
                    year={calcYear}
                  />
                </DropdownMenuContent>
              </DropdownMenu>

              <Button
                aria-label={t("payroll.next_period", "Kỳ sau")}
                className="w-10 h-10 flex items-center justify-center text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 hover:bg-white dark:hover:bg-slate-900 hover:shadow-sm rounded-xl transition-all"
                onClick={() => {
                  onChangeMonth(1);
                }}
                variant="ghost"
              >
                <ArrowRight size={18} />
              </Button>
            </div>

            <div className="h-8 w-px bg-slate-200 dark:bg-slate-700" />

            {/* Lock / Unlock Toggle Button */}
            {onToggleLock && (
              <Button
                aria-label={
                  isLocked
                    ? t("payroll.lifecycle.unlock_btn")
                    : t("payroll.lifecycle.lock_btn")
                }
                className={cn(
                  "flex items-center gap-1.5 px-3.5 h-10 rounded-xl font-bold text-xs transition-all",
                  isLocked
                    ? "text-rose-700 dark:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950/30"
                    : "text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800"
                )}
                onClick={() => {
                  setIsConfirmOpen(true);
                }}
                variant="ghost"
              >
                {isLocked ? (
                  <>
                    <Unlock size={16} />
                    <span>{t("payroll.lifecycle.unlock_btn")}</span>
                  </>
                ) : (
                  <>
                    <Lock size={16} />
                    <span>{t("payroll.lifecycle.lock_btn")}</span>
                  </>
                )}
              </Button>
            )}

            {/* Batch Calculate Button */}
            <Button
              className={cn(
                "flex items-center gap-2 px-5 h-10 rounded-xl font-bold text-[10px] uppercase tracking-widest transition-all shadow-md active:scale-95",
                selectedCount > 0 && !isLocked
                  ? "bg-primary-600 text-white shadow-primary-100 hover:bg-primary-700"
                  : "bg-slate-100 dark:bg-slate-700 text-slate-400 dark:text-slate-500 shadow-none cursor-not-allowed"
              )}
              data-testid="calculate-batch-btn"
              disabled={selectedCount === 0 || isLocked}
              onClick={() => {
                onOpenBatchCalc();
              }}
              title={
                isLocked
                  ? t("payroll.lifecycle.locked_tooltip")
                  : undefined
              }
              variant="ghost"
            >
              <Calculator size={18} />
              {t("payroll.calculate_btn")}{" "}
              {selectedCount > 0 && (
                <span className="ml-1 bg-white/20 px-2 py-0.5 rounded-lg text-white">
                  {selectedCount}
                </span>
              )}
            </Button>
          </div>
        }
        data-testid="payroll-title"
        subtitle={t("payroll.subtitle")}
        title={t("payroll.title")}
      />

      {/* Confirmation Dialog for Lock / Unlock */}
      <ConfirmDialog
        cancelLabel={t("common.cancel", "Hủy")}
        confirmLabel={
          isLocked
            ? t("payroll.lifecycle.unlock_btn")
            : t("payroll.lifecycle.lock_btn")
        }
        isOpen={isConfirmOpen}
        message={
          isLocked
            ? t("payroll.lifecycle.unlock_confirm_desc")
            : t("payroll.lifecycle.lock_confirm_desc")
        }
        onClose={() => {
          setIsConfirmOpen(false);
        }}
        onConfirm={() => {
          setIsConfirmOpen(false);
          onToggleLock?.();
        }}
        title={
          isLocked
            ? t("payroll.lifecycle.unlock_confirm_title")
            : t("payroll.lifecycle.lock_confirm_title")
        }
        variant={isLocked ? "info" : "warning"}
      />
    </>
  );
};
