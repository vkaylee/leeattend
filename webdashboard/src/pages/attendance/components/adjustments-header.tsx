import { Upload } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface AdjustmentsHeaderProps {
  onImportClick: () => void;
}

export function AdjustmentsHeader({ onImportClick }: AdjustmentsHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
      <div className="space-y-1">
        <Typography className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none" variant="h1">
          {t("attendance_confirm.title")}
        </Typography>
        <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60" variant="p">
          {t("attendance_confirm.subtitle")}
        </Typography>
      </div>
      <div className="flex items-center gap-4">
        <Button icon={<Upload size={18} />} onClick={onImportClick} variant="outline">
          {t("attendance_confirm.import_excel").toUpperCase()}
        </Button>
      </div>
    </div>
  );
}
