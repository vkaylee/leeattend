import { Clock } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { TableEmptyRow } from "@/components/ui/table-empty-row";

interface AdjustmentsTableEmptyProps {
  colSpan: number;
}

export function AdjustmentsTableEmpty({ colSpan }: AdjustmentsTableEmptyProps) {
  const { t } = useTranslation();

  return (
    <TableEmptyRow
      colSpan={colSpan}
      description={t("adjustments.empty.description", { defaultValue: "Hiện không có yêu cầu điều chỉnh công nào cần xử lý." })}
      icon={<Clock className="text-slate-300 dark:text-slate-600" size={40} />}
      title={t("attendance_confirm.messages.no_data")}
    />
  );
}
