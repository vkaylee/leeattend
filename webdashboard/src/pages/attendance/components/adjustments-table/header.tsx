import React from "react";
import { useTranslation } from "react-i18next";

export function AdjustmentsTableHeader() {
  const { t } = useTranslation();

  return (
    <thead>
      <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
        <th className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
          {t("common.employee")}
        </th>
        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
          {t("common.time")}
        </th>
        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]">
          {t("common.notes")}
        </th>
        <th className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center">
          {t("common.status")}
        </th>
        <th className="px-10 py-6 border-b border-slate-100 dark:border-slate-800"></th>
      </tr>
    </thead>
  );
}
