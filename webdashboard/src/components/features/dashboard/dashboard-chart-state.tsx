import { type ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

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
