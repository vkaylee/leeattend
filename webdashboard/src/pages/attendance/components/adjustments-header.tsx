import { Plus, Upload } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { PageHeader } from "@/components/ui/page-header";
interface AdjustmentsHeaderProps {
  canSubmit?: boolean;
  onImportClick: () => void;
  onRequestClick?: () => void;
}

export function AdjustmentsHeader({ canSubmit, onImportClick, onRequestClick }: AdjustmentsHeaderProps) {
  const { t } = useTranslation(["common", "attendance"]);

  return (
    <PageHeader
      actions={
        <div className="flex items-center gap-3 flex-wrap">
          {!!canSubmit && !!onRequestClick && (
            <Button className="rounded-xl h-10 font-bold" icon={<Plus size={18} />} onClick={onRequestClick} variant="primary">
              {t("time_tracking.modal.new_title")}
            </Button>
          )}
          <Button className="rounded-xl h-10 font-bold" icon={<Upload size={18} />} onClick={onImportClick} variant="outline">
            {t("attendance_confirm.import_excel")}
          </Button>
        </div>
      }
      className="px-2"
      subtitle={t("attendance_confirm.subtitle")}
      title={t("attendance_confirm.title")}
    />
  );
}
