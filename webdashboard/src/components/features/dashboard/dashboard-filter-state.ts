import { useEffect, useState } from "react";

import {
  canonicalizeDashboardMetricsFilters,
  parseDashboardMetricsFilters,
  type DashboardMetricsFilters,
} from "@/hooks/dashboard-metrics-filters";

function initialFilters(): DashboardMetricsFilters {
  if (typeof window === "undefined") return canonicalizeDashboardMetricsFilters({});
  return parseDashboardMetricsFilters(window.location.search);
}

export function useDashboardFilterState(): readonly [
  DashboardMetricsFilters,
  React.Dispatch<React.SetStateAction<DashboardMetricsFilters>>,
] {
  const [filters, setFilters] = useState<DashboardMetricsFilters>(initialFilters);
  useEffect(() => {
    const handlePopState = () => { setFilters(initialFilters()); };
    window.addEventListener("popstate", handlePopState);
    return () => { window.removeEventListener("popstate", handlePopState); };
  }, []);
  return [filters, setFilters];
}
