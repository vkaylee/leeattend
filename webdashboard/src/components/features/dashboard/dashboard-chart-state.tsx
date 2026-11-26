import { type ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

import { DashboardChartErrorContext } from "./dashboard-chart-context";
import { useDashboardChartError } from "./use-dashboard-chart-error";

interface DashboardChartErrorProviderProps {
  children: ReactNode;
  onErrorChange?: (hasError: boolean) => void;
}

export function DashboardChartErrorProvider({ children, onErrorChange }: DashboardChartErrorProviderProps) {
  return (
    <DashboardChartErrorContext.Provider value={{ reportError: onErrorChange ?? (() => undefined) }}>
      {children}
    </DashboardChartErrorContext.Provider>
  );
}

interface DashboardChartLoadingProps {
  heightClass?: string;
}

export function DashboardChartLoading({ heightClass = "h-[280px]" }: DashboardChartLoadingProps) {
  return (
    <div className={cn(heightClass, "flex items-center justify-center")}>
      <div className="w-8 h-8 border-2 border-primary-200 dark:border-primary-800 border-t-primary-600 dark:border-t-primary-400 rounded-full animate-spin" />
    </div>
  );
}

interface DashboardChartEmptyProps {
  heightClass?: string;
  icon: ReactNode;
}

export function DashboardChartEmpty({ icon, heightClass = "h-[280px]" }: DashboardChartEmptyProps) {
  const { t } = useTranslation();

  return (
    <div className={cn(heightClass, "flex flex-col items-center justify-center")}>
      <div className="text-slate-200 dark:text-slate-700 mb-4">{icon}</div>
      <Typography className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest" variant="p">
        {t("common.no_data")}
      </Typography>
    </div>
  );
}

interface DashboardChartErrorProps {
  icon: ReactNode;
}

export function DashboardChartError({ icon }: DashboardChartErrorProps) {
  useDashboardChartError(true);
  const { t } = useTranslation();

  return (
    <div
      aria-live="assertive"
      className="h-[280px] flex flex-col items-center justify-center text-center"
      role="alert"
    >
      <div className="text-rose-300 dark:text-rose-800 mb-4">{icon}</div>
      <Typography className="text-[10px] font-bold text-rose-500 dark:text-rose-400 uppercase tracking-widest" variant="p">
        {t("dashboard.connection_error")}
      </Typography>
      <Typography className="mt-2 max-w-sm text-[10px] font-medium text-rose-400 dark:text-rose-500" variant="p">
        {t("dashboard.connection_error_desc")}
      </Typography>
    </div>
  );
}
