import React from "react";

import { History as HistoryIcon, Calendar as CalendarIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/Button";
import Modal from "@/components/ui/Modal";
import StatusBadge from "@/components/ui/StatusBadge";

import { EmploymentPeriod } from "@/lib/api";

interface EmploymentHistoryModalProps {
  isOpen: boolean;
  onClose: () => void;
  employmentHistory: EmploymentPeriod[] | undefined;
  formatDate: (dateString: string) => string;
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
      isOpen={isOpen}
      onClose={onClose}
      title={t("employees.history.title")}
      subtitle={t("employees.history.subtitle")}
      icon={<HistoryIcon size={24} />}
      maxWidth="lg"
    >
      <div className="space-y-4 pt-4 animate-in fade-in duration-500">
        {employmentHistory?.map((period) => (
          <div
            key={period.id}
            className="p-8 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] shadow-inner group hover:bg-white dark:hover:bg-slate-900 transition-all relative overflow-hidden"
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
                  <CalendarIcon size={14} className="text-primary-400" />{" "}
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
        <Button variant="secondary" size="lg" onClick={onClose} type="button">
          {t("common.close").toUpperCase()}
        </Button>
      </div>
    </Modal>
  );
}
