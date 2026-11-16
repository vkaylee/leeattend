import { useCallback, useState } from "react";

import type { TableDensity } from "./table-density-toggle";

export function useTableDensity(
  storageKey = "hrm_table_density",
  defaultDensity: TableDensity = "normal",
): { density: TableDensity; setDensity: (density: TableDensity) => void } {
  const [density, setDensityState] = useState<TableDensity>(() => {
    if (typeof window === "undefined") return defaultDensity;
    try {
      const saved = localStorage.getItem(storageKey);
      if (saved === "compact" || saved === "normal" || saved === "comfortable") {
        return saved;
      }
    } catch {
      // Ignore localStorage access errors
    }
    return defaultDensity;
  });

  const setDensity = useCallback(
    (newDensity: TableDensity) => {
      setDensityState(newDensity);
      try {
        localStorage.setItem(storageKey, newDensity);
      } catch {
        // Ignore localStorage access errors
      }
    },
    [storageKey],
  );

  return { density, setDensity };
}
