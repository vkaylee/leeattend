import { ShieldAlert } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

export function DashboardErrorBanner() {
  const { t } = useTranslation();
  
  return (
    <div className="bg-rose-50 dark:bg-rose-950 border-2 border-rose-100 dark:border-rose-800 p-6 sm:p-10 rounded-3xl sm:rounded-[3rem] flex items-center gap-6 sm:gap-8 text-rose-700 dark:text-rose-300 animate-in slide-in-from-bottom-8 shadow-2xl shadow-rose-100/50">
      <div className="bg-white dark:bg-slate-900 p-4 sm:p-5 rounded-[1.5rem] shadow-sm border border-rose-100 dark:border-rose-800">
        <ShieldAlert className="text-rose-500" size={40} />
      </div>
      <div>
        <Typography className="font-bold text-xl sm:text-2xl uppercase tracking-tighter leading-none mb-2" variant="p">{t("dashboard.connection_error")}</Typography>
        <Typography className="font-bold opacity-70 uppercase tracking-tight text-sm" variant="p">{t("dashboard.connection_error_desc")}</Typography>
      </div>
    </div>
  );
}
