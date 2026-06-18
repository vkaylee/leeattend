import { Plus, Upload } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface AdjustmentsHeaderProps {
  canSubmit?: boolean;
  onImportClick: () => void;
  onRequestClick?: () => void;
}

export function AdjustmentsHeader({ canSubmit, onImportClick, onRequestClick }: AdjustmentsHeaderProps) {
  const { t } = useTranslation(["common", "attendance"]);

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
      <div>
        <Typography className="font-bold mb-0.5 normal-case" variant="h3">
          {t("attendance_confirm.title")}
        </Typography>
        <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium opacity-80" variant="small">
          {t("attendance_confirm.subtitle")}
        </Typography>
      </div>
      <div className="flex items-center gap-3 flex-wrap">
        {!!canSubmit && !!onRequestClick && (
          <Button className="rounded-xl h-10" icon={<Plus size={18} />} onClick={onRequestClick} variant="primary">
            {t("time_tracking.modal.new_title").toUpperCase()}
          </Button>
        )}
        <Button className="rounded-xl h-10" icon={<Upload size={18} />} onClick={onImportClick} variant="outline">
          {t("attendance_confirm.import_excel").toUpperCase()}
        </Button>
      </div>
    </div>
  );
}
