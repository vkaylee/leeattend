import { History as HistoryIcon, Calendar as CalendarIcon } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import Modal from "@/components/ui/modal";
import StatusBadge from "@/components/ui/status-badge";
import { EmploymentPeriod } from "@/lib/api";

interface EmploymentHistoryModalProps {
  employmentHistory: EmploymentPeriod[] | undefined;
  formatDate: (dateString: string) => string;
  isOpen: boolean;
  onClose: () => void;
}

export default function EmploymentHistoryModal({
  isOpen,
  onClose,
  employmentHistory,
  formatDate,
}: EmploymentHistoryModalProps) {
  const { t } = useTranslation();

  return (
    <Modal
      icon={<HistoryIcon size={24} />}
      isOpen={isOpen}
      maxWidth="lg"
      onClose={onClose}
      subtitle={t("employees.history.subtitle")}
      title={t("employees.history.title")}
    >
      <div className="space-y-4 pt-4 animate-in fade-in duration-500">
        {employmentHistory?.map((period) => (
          <div
            className="p-8 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] shadow-inner group hover:bg-white dark:hover:bg-slate-900 transition-all relative overflow-hidden"
            key={period.id}
          >
            <div className="flex justify-between items-center relative z-10">
              <div>
                <div className="font-bold text-slate-700 dark:text-slate-200 text-lg uppercase tracking-tight flex items-center gap-4">
                  {period.is_rehire
                    ? t("employees.history.rehire_period")
                    : t("employees.history.first_period")}
                  {!period.termination_date && (
                    <StatusBadge variant="success">
                      {t("employees.history.currently_working")}
                    </StatusBadge>
                  )}
                </div>
                <div className="text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.25em] mt-3 flex items-center gap-3 opacity-70">
                  <CalendarIcon className="text-primary-400" size={14} />{" "}
                  {t("employees.history.period_range", {
                    start: formatDate(period.hire_date),
                    end: period.termination_date
                      ? formatDate(period.termination_date)
                      : t("employees.history.present").toUpperCase(),
                  })}
                </div>
              </div>
            </div>
            <CalendarIcon className="absolute -right-8 -bottom-8 w-32 h-32 text-slate-700 dark:text-slate-200 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-700" />
          </div>
        ))}
      </div>
      <div className="mt-10 pt-8 border-t border-slate-50 dark:border-slate-800 flex justify-end">
        <Button onClick={onClose} size="lg" type="button" variant="secondary">
          {t("common.close").toUpperCase()}
        </Button>
      </div>
    </Modal>
  );
}
