import { CheckCircle2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface Props {
  successCount: number;
  onClose: () => void;
}

export default function ImportSuccessState({ successCount, onClose }: Props) {
  const { t } = useTranslation("employees");

  return (
    <div className="flex flex-col items-center justify-center py-10 space-y-6 text-center animate-in zoom-in-95 duration-500">
      <div className="w-20 h-20 bg-emerald-50 dark:bg-emerald-950/30 rounded-3xl flex items-center justify-center text-emerald-500 shadow-lg border border-emerald-100 dark:border-emerald-900/50">
        <CheckCircle2 size={40} />
      </div>
      <div>
        <Typography className="font-extrabold text-slate-800 dark:text-slate-200 uppercase tracking-tight text-xl mb-2" variant="h3">
          {t("employees.bulk_termination.import_success_title")}
        </Typography>
        <Typography className="text-sm text-slate-500 dark:text-slate-400" variant="p">
          {t("employees.bulk_termination.import_success_desc", { count: successCount })}
        </Typography>
      </div>
      <Button className="px-10 mt-4 shadow-sm" onClick={onClose} variant="secondary">
        {t("employees.bulk_termination.close_btn")}
      </Button>
    </div>
  );
}
