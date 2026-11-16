import { createContext } from "react";

export interface DashboardChartErrorContextValue {
  reportError: (hasError: boolean) => void;
}

export const DashboardChartErrorContext = createContext<DashboardChartErrorContextValue | null>(null);
