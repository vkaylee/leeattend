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
        <div className="relative -mt-16 mb-8 flex items-end gap-8">
          <div className="w-32 h-32 rounded-[2.5rem] border-[10px] border-white shadow-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 font-bold text-3xl uppercase shadow-inner rotate-3 group-hover/hero:rotate-0 transition-transform duration-700">
            {employee.full_name[0]}
          </div>
          <div className="mb-2">
            <h2 className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none mb-2">
              {employee.full_name}
            </h2>
            <div className="flex flex-wrap gap-4 items-center">
              <span className="text-[10px] font-bold text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-3 py-1 rounded-full border border-primary-100 dark:border-primary-800 shadow-sm uppercase tracking-widest">
                {t("common.employee_id")}: {employee.employee_id_number}
              </span>
              <div className="w-1.5 h-1.5 bg-slate-200 dark:bg-slate-600 rounded-full" />
              <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest opacity-70">
                {t("common.hire_date")}: {formatDate(employee.hire_date)}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
