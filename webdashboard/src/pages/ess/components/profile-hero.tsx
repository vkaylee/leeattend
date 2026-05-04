import { useTranslation } from "react-i18next";

import { useDateTimeFormatter } from "@/hooks/use-datetime";

interface ProfileHeroProps {
  employee: {
    employee_id_number: string;
    full_name: string;
    hire_date: string;
  };
}

export function ProfileHero({ employee }: ProfileHeroProps) {
  const { t } = useTranslation();
  const { formatDate } = useDateTimeFormatter();

  return (
    <div className="max-w-5xl bg-white dark:bg-slate-900 rounded-[3rem] border border-slate-100 dark:border-slate-800 shadow-sm overflow-hidden mb-10 group/hero">
      <div className="h-40 bg-slate-900 relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-primary-600/40 via-blue-600/20 to-transparent" />
        <div className="absolute top-0 left-0 w-full h-full opacity-10 pointer-events-none bg-[radial-gradient(circle_at_50%_50%,rgba(255,255,255,0.1)_0,transparent_50%)]" />
      </div>
      <div className="px-12 pb-10">
        <div className="relative -mt-20 mb-8 flex flex-col sm:flex-row items-center sm:items-end gap-6 sm:gap-10">
          <div className="w-40 h-40 rounded-[3rem] border-[12px] border-white dark:border-slate-900 shadow-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 font-black text-5xl uppercase shadow-inner rotate-3 group-hover/hero:rotate-0 transition-all duration-700 shrink-0">
            {employee.full_name[0]}
          </div>
          
          <div className="flex-1 pb-4">
            <div className="backdrop-blur-xl bg-white/60 dark:bg-slate-900/60 p-6 sm:p-8 rounded-[2.5rem] border border-white/40 dark:border-slate-800/50 shadow-xl shadow-slate-200/20 dark:shadow-none inline-block min-w-full sm:min-w-[400px]">
              <h2 className="text-3xl sm:text-4xl font-black text-slate-800 dark:text-white tracking-tight uppercase leading-none mb-4 drop-shadow-sm">
                {employee.full_name}
              </h2>
              <div className="flex flex-wrap gap-4 items-center">
                <div className="flex items-center gap-2 text-[10px] font-black text-primary-700 dark:text-primary-300 bg-primary-100/50 dark:bg-primary-900/50 px-4 py-2 rounded-2xl border border-primary-200/50 dark:border-primary-700/50 uppercase tracking-widest">
                  <span className="opacity-60">{t("common.employee_id")}:</span>
                  <span>{employee.employee_id_number}</span>
                </div>
                <div className="hidden sm:block w-1.5 h-1.5 bg-slate-300 dark:bg-slate-600 rounded-full" />
                <div className="flex items-center gap-2 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-80">
                  <span className="opacity-60">{t("common.hire_date")}:</span>
                  <span>{formatDate(employee.hire_date)}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
