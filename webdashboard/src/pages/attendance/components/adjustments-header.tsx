import { FileDown, Upload } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";
import Button from "@/components/ui/button";

interface AdjustmentsHeaderProps {
  onDownloadTemplate: () => void;
  onImportClick: () => void;
}

export function AdjustmentsHeader({ onDownloadTemplate, onImportClick }: AdjustmentsHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
      <div className="space-y-1">
        <h1 className="text-4xl font-bold tracking-tighter text-slate-700 dark:text-slate-200 uppercase leading-none">
          {t("attendance_confirm.title")}
        </h1>
        <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] opacity-60">
          {t("attendance_confirm.subtitle")}
        </p>
      </div>
      <div className="flex items-center gap-4">
        <Button icon={<FileDown size={18} />} onClick={onDownloadTemplate} variant="outline">
          {t("attendance_confirm.excel_template").toUpperCase()}
        </Button>
        <Button icon={<Upload size={18} />} onClick={onImportClick}>
          {t("attendance_confirm.import_excel").toUpperCase()}
        </Button>
      </div>
    </div>
  );
}
