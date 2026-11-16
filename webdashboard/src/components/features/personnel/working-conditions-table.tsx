import { Link } from "@tanstack/react-router";
import {
  Baby,
  Ban,
  Clock,
  ExternalLink,
  Heart,
  HeartHandshake,
  ShieldAlert,
  Sparkles,
  Trash2,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { ConditionTypeDefinition } from "@/lib/api";
import type { WorkingConditionWithEmployee } from "@/lib/api/employee-conditions";

import Button from "@/components/ui/button";
import EmptyState from "@/components/ui/empty-state";
import ResponsiveTable from "@/components/ui/responsive-table";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
import { cn } from "@/lib/utils";

const ICON_MAP: Record<string, React.ReactNode> = {
  Baby: <Baby size={18} />,
  Heart: <Heart size={18} />,
  ShieldAlert: <ShieldAlert size={18} />,
  Ban: <Ban size={18} />,
  Sparkles: <Sparkles size={18} />,
};

function getConditionIcon(typeDefs: ConditionTypeDefinition[], typeCode: string): React.ReactNode {
  const def = typeDefs.find((d) => d.code === typeCode.toUpperCase());
  if (def?.icon && ICON_MAP[def.icon]) return ICON_MAP[def.icon];
  return <Clock size={18} />;
}

function getConditionDisplayName(
  typeDefs: ConditionTypeDefinition[],
  typeCode: string,
  lang: string
): string {
  const def = typeDefs.find((d) => d.code === typeCode.toUpperCase());
  if (def?.name_i18n) {
    return def.name_i18n[lang] || def.name_i18n.en || typeCode;
  }
  return typeCode;
}

function ConditionMobileCard({
  condition,
  typeDefs,
  formatDate,
  onDelete,
  t,
  currentLang,
}: {
  condition: WorkingConditionWithEmployee;
  currentLang: string;
  formatDate: (date: string) => string;
  onDelete: (condition: WorkingConditionWithEmployee) => void;
  t: (key: string, options?: { defaultValue?: string }) => string;
  typeDefs: ConditionTypeDefinition[];
}) {
  const displayName = getConditionDisplayName(typeDefs, condition.type, currentLang);
  const icon = getConditionIcon(typeDefs, condition.type);

  return (
    <div className="rounded-2xl border border-slate-100 bg-white p-5 shadow-xs transition-all active:scale-[0.99] dark:border-slate-800 dark:bg-slate-900 space-y-4">
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-3 min-w-0">
          <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-primary-100 bg-primary-50 text-primary-600 dark:border-primary-800 dark:bg-primary-950 dark:text-primary-400">
            {icon}
          </div>
          <div className="min-w-0 flex-1">
            <Link
              className="truncate text-sm font-bold uppercase tracking-tight text-slate-800 dark:text-slate-100 hover:text-primary-600 dark:hover:text-primary-400 flex items-center gap-1"
              params={{ employeeId: condition.employee_id }}
              search={{ tab: "working-conditions" }}
              to="/employees/$employeeId"
            >
              <span className="truncate">{condition.employee_name}</span>
              <ExternalLink className="opacity-50 shrink-0" size={12} />
            </Link>
            {!!condition.employee_code && (
              <div className="mt-0.5 font-mono text-[10px] font-bold uppercase tracking-wider text-slate-400">
                {condition.employee_code}
              </div>
            )}
          </div>
        </div>

        <span
          className={cn(
            "text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 border rounded-full shadow-xs shrink-0",
            condition.is_paid
              ? "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800"
              : "bg-slate-50 text-slate-500 border-slate-100 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700"
          )}
        >
          {condition.is_paid
            ? t("working_conditions.paid", { defaultValue: "Có hưởng lương" })
            : t("working_conditions.unpaid", { defaultValue: "Không hưởng lương" })}
        </span>
      </div>

      <div className="text-xs font-semibold text-slate-700 dark:text-slate-300 bg-slate-50 dark:bg-slate-800/40 px-3 py-2 rounded-xl">
        <span className="font-bold text-primary-600 dark:text-primary-400">{displayName}</span>
      </div>

      <div className="text-xs text-slate-500 dark:text-slate-400">
        <span className="font-medium text-slate-400">{t("common.date", { defaultValue: "Thời hạn" })}:</span>{" "}
        <span className="font-semibold text-slate-700 dark:text-slate-300">
          {formatDate(condition.start_date)}
          {" → "}
          {condition.end_date ? formatDate(condition.end_date) : t("common.present", { defaultValue: "Hiện tại" })}
        </span>
      </div>

      <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100 dark:border-slate-800">
        <Button
          aria-label={t("working_conditions.actions.delete", { defaultValue: "Xóa điều kiện" })}
          className="h-8 text-xs font-semibold text-rose-600 hover:text-rose-700 hover:bg-rose-50 dark:hover:bg-rose-950/30"
          icon={<Trash2 size={13} />}
          onClick={() => { onDelete(condition); }}
          size="sm"
          variant="ghost"
        >
          {t("common.delete", { defaultValue: "Xóa" })}
        </Button>
      </div>
    </div>
  );
}

function ConditionsDesktopTable({
  conditions,
  currentLang,
  formatDate,
  onDelete,
  t,
  typeDefs,
}: {
  conditions: WorkingConditionWithEmployee[];
  currentLang: string;
  formatDate: (date: string) => string;
  onDelete: (condition: WorkingConditionWithEmployee) => void;
  t: (key: string, options?: { defaultValue?: string }) => string;
  typeDefs: ConditionTypeDefinition[];
}) {
  const { onClickCapture, setRef } = useHorizontalDragScroll<HTMLDivElement>();
  const headerClass = "px-8 py-5 text-[10px] font-bold text-slate-400 uppercase tracking-widest";
  const headers = [["working_conditions.table.employee", "NHÂN SỰ"], ["working_conditions.table.condition_type", "LOẠI ĐIỀU KIỆN"], ["working_conditions.table.duration", "THỜI HẠN ÁP DỤNG"], ["working_conditions.table.payment_regime", "CHẾ ĐỘ HƯỞNG LƯƠNG"]] as const;
  return (
    <div className="overflow-x-auto overscroll-contain" onClickCapture={onClickCapture} ref={setRef}><table className="w-full min-w-[820px] text-left border-collapse"><thead><tr className="border-b border-slate-100 dark:border-slate-800/60">{headers.map(([key, fallback]) => <th className={headerClass} key={key}>{t(key, { defaultValue: fallback })}</th>)}<th className={`${headerClass} text-right`}>{t("common.actions", { defaultValue: "THAO TÁC" })}</th></tr></thead><tbody>{conditions.map((condition) => <ConditionDesktopRow condition={condition} currentLang={currentLang} formatDate={formatDate} key={condition.id} onDelete={onDelete} t={t} typeDefs={typeDefs} />)}</tbody></table></div>
  );
}

function ConditionDesktopRow({
  condition,
  currentLang,
  formatDate,
  onDelete,
  t,
  typeDefs,
}: {
  condition: WorkingConditionWithEmployee;
  currentLang: string;
  formatDate: (date: string) => string;
  onDelete: (condition: WorkingConditionWithEmployee) => void;
  t: (key: string, options?: { defaultValue?: string }) => string;
  typeDefs: ConditionTypeDefinition[];
}) {
  const displayName = getConditionDisplayName(typeDefs, condition.type, currentLang);
  const icon = getConditionIcon(typeDefs, condition.type);
  const paymentClass = condition.is_paid ? "bg-emerald-50 text-emerald-600 border-emerald-100 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800" : "bg-slate-50 text-slate-500 border-slate-100 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700";
  return (
    <tr className="border-b border-slate-50 dark:border-slate-800/50 hover:bg-slate-50/40 dark:hover:bg-slate-800/20 transition-colors group"><td className="px-8 py-5"><div className="flex flex-col"><Link className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-wide group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors flex items-center gap-1.5" params={{ employeeId: condition.employee_id }} search={{ tab: "working-conditions" }} to="/employees/$employeeId"><span>{condition.employee_name}</span><ExternalLink className="opacity-0 group-hover:opacity-70 transition-opacity" size={11} /></Link>{!!condition.employee_code && <span className="text-[10px] font-mono text-slate-400 mt-0.5 font-bold">{condition.employee_code}</span>}</div></td><td className="px-8 py-5"><div className="flex items-center gap-2.5"><div className="w-8 h-8 rounded-xl bg-slate-100 dark:bg-slate-800 flex items-center justify-center text-slate-600 dark:text-slate-300 shrink-0">{icon}</div><span className="text-xs font-bold text-slate-700 dark:text-slate-200 uppercase tracking-wide">{displayName}</span></div></td><td className="px-8 py-5 text-xs text-slate-600 dark:text-slate-300 font-medium">{formatDate(condition.start_date)}{" → "}{condition.end_date ? formatDate(condition.end_date) : t("common.present", { defaultValue: "Hiện tại" })}</td><td className="px-8 py-5"><span className={cn("text-[9px] font-bold uppercase tracking-widest px-2.5 py-0.5 border rounded-full shadow-xs", paymentClass)}>{condition.is_paid ? t("working_conditions.paid", { defaultValue: "Có hưởng lương" }) : t("working_conditions.unpaid", { defaultValue: "Không hưởng lương" })}</span></td><td className="px-8 py-5 text-right"><div className="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity"><Button aria-label={t("working_conditions.actions.delete", { defaultValue: "Xóa" })} className="p-2 text-slate-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-900/30 rounded-xl transition-colors h-auto w-auto shadow-none" onClick={() => { onDelete(condition); }} size="icon" variant="ghost"><Trash2 size={14} /></Button></div></td></tr>
  );
}

export interface WorkingConditionsTableProps {
  conditions: WorkingConditionWithEmployee[];
  isLoading: boolean;
  onClearFilters: () => void;
  onDelete: (condition: WorkingConditionWithEmployee) => void;
  paymentFilter: string;
  search: string;
  selectedType: string;
  typeDefs: ConditionTypeDefinition[];
}

export function WorkingConditionsTable({
  conditions,
  isLoading,
  search,
  selectedType,
  paymentFilter,
  typeDefs,
  onClearFilters,
  onDelete,
}: WorkingConditionsTableProps) {
  const { t, i18n } = useTranslation();
  const { formatDate } = useDateTimeFormatter();
  const currentLang = i18n.language.split("-")[0];

  if (isLoading) {
    return (
      <div className="p-8 space-y-4 animate-pulse">
        <div className="h-10 bg-slate-100 dark:bg-slate-800 rounded-xl w-full" />
        <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full" />
        <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full" />
        <div className="h-16 bg-slate-50 dark:bg-slate-800/60 rounded-xl w-full" />
      </div>
    );
  }

  const hasFilter = Boolean(search || selectedType !== "all" || paymentFilter !== "all");

  if (conditions.length === 0) {
    return (
      <EmptyState
        action={
          hasFilter ? (
            <Button onClick={onClearFilters} variant="outline">
              {t("common.clear_filters", { defaultValue: "XÓA BỘ LỌC" })}
            </Button>
          ) : undefined
        }
        description={
          hasFilter
            ? t("working_conditions.search_no_results", {
                defaultValue: "Không tìm thấy hồ sơ điều kiện làm việc nào phù hợp với bộ lọc.",
              })
            : t("working_conditions.empty_state", {
                defaultValue: "Chưa có điều kiện làm việc đặc thù nào được ghi nhận trong không gian làm việc này.",
              })
        }
        icon={<HeartHandshake size={48} />}
        title={t("working_conditions.no_conditions_found", { defaultValue: "Không tìm thấy điều kiện làm việc" })}
      />
    );
  }

  return (
    <ResponsiveTable<WorkingConditionWithEmployee>
      data={conditions}
      emptyState={{
        title: t("working_conditions.no_conditions_found", { defaultValue: "Không tìm thấy điều kiện làm việc" }),
        description: hasFilter
          ? t("working_conditions.search_no_results", {
              defaultValue: "Không tìm thấy hồ sơ điều kiện làm việc nào phù hợp với bộ lọc.",
            })
          : t("working_conditions.empty_state", {
              defaultValue: "Chưa có điều kiện làm việc đặc thù nào được ghi nhận trong không gian làm việc này.",
            }),
        icon: <HeartHandshake size={48} />,
        action: hasFilter ? (
          <Button onClick={onClearFilters} variant="outline">
            {t("common.clear_filters", { defaultValue: "XÓA BỘ LỌC" })}
          </Button>
        ) : undefined,
      }}
      getCardKey={(cond) => cond.id}
      isLoading={isLoading}
      renderCard={(condition) => (
        <ConditionMobileCard
          condition={condition}
          currentLang={currentLang}
          formatDate={formatDate}
          onDelete={onDelete}
          t={t}
          typeDefs={typeDefs}
        />
      )}
      renderDesktopTable={() => <ConditionsDesktopTable conditions={conditions} currentLang={currentLang} formatDate={formatDate} onDelete={onDelete} t={t} typeDefs={typeDefs} />}
    />
  );
}
