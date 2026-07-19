import { Zap } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

interface CorrectionConfigCardProps {
  correctionEffectiveDate: string;
  isCorrection: boolean;
  onCorrectionChange: (checked: boolean) => void;
  onEffectiveDateChange: (date: string) => void;
}

export function CorrectionConfigCard({
  isCorrection,
  correctionEffectiveDate,
  onCorrectionChange,
  onEffectiveDateChange,
}: CorrectionConfigCardProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-amber-50 dark:bg-amber-950/20 border border-amber-100 dark:border-amber-900/60 rounded-[2.5rem] p-10 shadow-sm space-y-6">
      <div className="flex items-center gap-3">
        <div className="bg-amber-100 dark:bg-amber-900 p-2.5 rounded-2xl text-amber-600 dark:text-amber-300">
          <Zap size={18} />
        </div>
        <div>
          <Typography
            className="text-xs font-bold text-amber-900 dark:text-amber-400 uppercase tracking-tight"
            variant="h4"
          >
            {t("shifts.modal.correction_label", "Cập nhật hồi tố (Retroactive Correction)")}
          </Typography>
          <Typography
            className="text-[10px] font-medium text-amber-700 dark:text-amber-300/80 leading-normal"
            variant="p"
          >
            {t(
              "shifts.modal.correction_desc",
              "Quyết định xem các thay đổi này có áp dụng cho dữ liệu quá khứ hay không.",
            )}
          </Typography>
        </div>
      </div>

      <div className="flex items-center gap-3 bg-white dark:bg-slate-900/50 p-4 rounded-2xl border border-amber-100/50 dark:border-amber-900/40">
        <input
          checked={isCorrection}
          className="w-4 h-4 rounded-md border-amber-300 text-amber-600 dark:text-amber-400 focus:ring-amber-500 cursor-pointer"
          id="editor_is_correction"
          onChange={(e) => { onCorrectionChange(e.target.checked); }}
          type="checkbox"
        />
        <label
          className="text-[10px] font-bold text-amber-800 dark:text-amber-300 uppercase tracking-widest cursor-pointer select-none"
          htmlFor="editor_is_correction"
        >
          {t(
            "shifts.modal.correction_warning",
            "CẬP NHẬT TRỰC TIẾP LÊN CA GỐC (ÁP DỤNG HỒI TỐ)",
          )}
        </label>
      </div>

      {!isCorrection ? (
        <div className="space-y-2 pt-2 animate-in fade-in slide-in-from-top-2 duration-300">
          <label
            className="text-[10px] font-bold text-amber-750 dark:text-amber-400 uppercase tracking-widest block"
            htmlFor="correction_effective_date_input"
          >
            {t("shifts.modal.effective_date_label", "Ngày hiệu lực mới")}
          </label>
          <input
            className="w-full bg-white dark:bg-slate-900 border border-amber-250 dark:border-amber-900 rounded-2xl px-5 py-3.5 font-bold text-slate-700 dark:text-slate-200 focus:ring-2 focus:ring-amber-500 transition-all outline-none"
            id="correction_effective_date_input"
            onChange={(e) => { onEffectiveDateChange(e.target.value); }}
            type="date"
            value={correctionEffectiveDate}
          />
          <Typography
            className="text-[9px] font-bold text-amber-600 dark:text-amber-350 leading-tight italic px-1"
            variant="p"
          >
            {t(
              "shifts.modal.effective_date_desc",
              "Các thay đổi sẽ chỉ có hiệu lực từ ngày được chọn trở đi.",
            )}
          </Typography>
        </div>
      ) : null}
    </div>
  );
}
