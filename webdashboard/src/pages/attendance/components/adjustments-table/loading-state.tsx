import React from "react";
import { useTranslation } from "react-i18next";

interface AdjustmentsTableLoadingProps {
  colSpan: number;
}

export function AdjustmentsTableLoading({ colSpan }: AdjustmentsTableLoadingProps) {
  const { t } = useTranslation();

  return (
    <tr>
      <td
        className="px-10 py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse"
        colSpan={colSpan}
      >
        {t("common.loading")}
      </td>
    </tr>
  );
}
