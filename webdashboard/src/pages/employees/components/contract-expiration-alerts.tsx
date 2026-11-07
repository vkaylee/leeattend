import { AlertTriangle, CalendarClock, ChevronDown, ChevronUp, FileCheck } from "lucide-react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Employee } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface ContractExpirationAlertsProps {
  employees: Employee[];
  onFilterProbation?: () => void;
  onFilterRenewals?: () => void;
}

export function ContractExpirationAlerts({
  employees,
  onFilterProbation,
  onFilterRenewals,
}: ContractExpirationAlertsProps) {
  const { t } = useTranslation();
  const [isExpanded, setIsExpanded] = useState(false);

  // Analyze employees nearing probation end (e.g. within 60 days of hire)
  const probationCandidates = useMemo(() => {
    const now = new Date().getTime();
    return employees.filter((e) => {
      if (e.employment_status === "terminated") return false;
      if (!e.hire_date) return false;
      const hireTime = new Date(e.hire_date).getTime();
      const diffDays = Math.floor((now - hireTime) / (1000 * 60 * 60 * 24));
      // In Vietnam, standard probation is 30 or 60 days
      return diffDays >= 45 && diffDays <= 60;
    });
  }, [employees]);

  // Total active employees
  const activeCount = useMemo(() => {
    return employees.filter((e) => e.employment_status !== "terminated").length;
  }, [employees]);

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl shadow-xs overflow-hidden">
      <div className="p-4 flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20">
            <CalendarClock size={20} />
          </div>
          <div>
            <h4 className="text-sm font-bold text-slate-800 dark:text-slate-100 flex items-center gap-2">
              <span>{t("employees.alerts.title")}</span>
              <span className="text-[11px] font-semibold px-2 py-0.5 rounded-full bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300">
                {activeCount} {t("employees.alerts.active_count")}
              </span>
            </h4>
            <p className="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
              {t("employees.alerts.subtitle")}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {probationCandidates.length > 0 && (
            <div
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-amber-500/10 text-amber-800 dark:text-amber-200 border border-amber-500/20 text-xs font-semibold"
              title="Nhân sự sắp hoàn thành thời gian thử việc 60 ngày cần làm thủ tục đánh giá"
            >
              <AlertTriangle size={14} className="text-amber-600 dark:text-amber-400" />
              <span>
                {probationCandidates.length}{" "}
                {t("employees.alerts.probation_review_due")}
              </span>
            </div>
          )}

          <div
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-blue-500/10 text-blue-800 dark:text-blue-200 border border-blue-500/20 text-xs font-semibold"
            title="Tuân thủ Điều 20 Bộ luật Lao động 2019"
          >
            <FileCheck size={14} className="text-blue-600 dark:text-blue-400" />
            <span>{t("employees.alerts.contract_compliant")}</span>
          </div>

          <Button
            aria-label={isExpanded ? t("employees.alerts.collapse") : t("employees.alerts.expand")}
            className="h-8 w-8 flex items-center justify-center p-0 rounded-xl hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-500"
            onClick={() => {
              setIsExpanded(!isExpanded);
            }}
            variant="ghost"
          >
            {isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
          </Button>
        </div>
      </div>

      {isExpanded && (
        <div className="px-5 py-3 border-t border-slate-100 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-800/20 text-xs space-y-2 animate-in slide-in-from-top-1 duration-200">
          <div className="text-slate-600 dark:text-slate-300 flex items-center gap-2">
            <span className="font-semibold">
              {t("employees.alerts.legal_note_title", "Quy định pháp lý BLLĐ:")}
            </span>
            <span>
              {t(
                "employees.alerts.legal_note_desc",
                "Doanh nghiệp phải thông báo kết quả thử việc trước khi hết thời gian thử việc 03 ngày (Điều 27). Hợp đồng xác định thời hạn phải được ký phụ lục hoặc tái ký trong vòng 30 ngày kể từ ngày hết hạn (Điều 20)."
              )}
            </span>
          </div>

          {probationCandidates.length > 0 && (
            <div className="flex flex-wrap gap-2 pt-1">
              {probationCandidates.map((c) => (
                <span
                  className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-200 text-xs"
                  key={c.id}
                >
                  <span className="font-bold">{c.full_name}</span>
                  <span className="text-[11px] text-slate-400">
                    (Vào làm: {c.hire_date})
                  </span>
                </span>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
