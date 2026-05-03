import { ArrowLeftRight } from "lucide-react";
import React from "react";

import { TFunction } from "i18next";

export function getTableHeaders(t: TFunction) {
  return [
    <th
      className="px-10 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]"
      key="date"
    >
      {t("shift_swaps.swap_date")}
    </th>,
    <th
      className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]"
      key="requester"
    >
      {t("shift_swaps.requester")}
    </th>,
    <th className="px-4 py-6 text-center" key="arrow">
      <ArrowLeftRight className="text-slate-400 mx-auto" size={14} />
    </th>,
    <th
      className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]"
      key="target"
    >
      {t("shift_swaps.target")}
    </th>,
    <th
      className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em]"
      key="reason"
    >
      {t("shift_swaps.reason")}
    </th>,
    <th
      className="px-8 py-6 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] text-center"
      key="status"
    >
      {t("common.status")}
    </th>,
    <th className="px-10 py-6" key="actions" />,
  ];
}
