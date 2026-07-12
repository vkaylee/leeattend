import { AlertCircle, Loader2, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";

interface Props {
  jobError: string | null;
  jobId: string | null;
  onBack: () => void;
}

export default function ImportErrorState({ jobError, jobId, onBack }: Props) {
  const { t } = useTranslation("employees");

  if (jobError) {
    return (
      <div className="flex flex-col items-center justify-center py-10 space-y-6 text-center animate-in zoom-in-95 duration-500">
        <div className="w-20 h-20 bg-rose-50 dark:bg-rose-950/30 rounded-3xl flex items-center justify-center text-rose-500 shadow-lg border border-rose-100 dark:border-rose-900/50">
          <AlertCircle size={40} />
        </div>
        <div>
          <Typography className="font-extrabold text-slate-800 dark:text-slate-200 uppercase tracking-tight text-xl mb-2 text-rose-600" variant="h3">
            {t("employees.bulk_termination.processing_error_title")}
          </Typography>
          <Typography className="text-sm text-slate-500 dark:text-slate-400 max-w-md" variant="p">
            {jobError}
          </Typography>
        </div>
        <Button className="px-10 mt-4 shadow-sm" onClick={onBack} variant="primary">
          {t("employees.bulk_termination.back_btn")}
        </Button>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center justify-center py-12 space-y-6 text-center animate-in fade-in duration-500">
      <div className="relative flex items-center justify-center">
        <Loader2 className="animate-spin text-primary-500" size={48} />
        <RefreshCw className="absolute text-primary-300 animate-pulse" size={24} />
      </div>
      <div>
        <Typography className="font-extrabold text-slate-800 dark:text-slate-200 uppercase tracking-tight text-xl mb-2" variant="h3">
          {t("employees.bulk_termination.processing_data_title")}
        </Typography>
        <Typography className="text-sm text-slate-500 dark:text-slate-400 max-w-md" variant="p">
          {t("employees.bulk_termination.processing_data_desc")}
        </Typography>
        {!!jobId && (
          <span className="inline-block mt-3 text-[10px] font-mono font-bold text-slate-400 bg-slate-100 dark:bg-slate-800 px-2.5 py-1 rounded-full">
            {t("employees.bulk_termination.task_id_label", { id: jobId })}
          </span>
        )}
      </div>
    </div>
  );
}
