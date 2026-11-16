import { useContext, useEffect } from "react";

import { DashboardChartErrorContext } from "./dashboard-chart-context";

export function useDashboardChartError(hasError: boolean): void {
  const context = useContext(DashboardChartErrorContext);

  useEffect(() => {
    context?.reportError(hasError);
    return () => {
      context?.reportError(false);
    };
  }, [context, hasError]);
}
