import { Clock } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

interface AdjustmentsTableEmptyProps {
  colSpan: number;
}

export function AdjustmentsTableEmpty({ colSpan }: AdjustmentsTableEmptyProps) {
  const { t } = useTranslation();

  return (
    <tr>
      <td
        className="px-10 py-32 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 flex flex-col items-center gap-4"
        colSpan={colSpan}
      >
        <Clock className="text-slate-200 dark:text-slate-700" size={40} />
        {t("attendance_confirm.messages.no_data")}
      </td>
    </tr>
  );
}
