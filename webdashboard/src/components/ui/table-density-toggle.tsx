import { AlignJustify, Maximize2, Rows } from "lucide-react";
import React, { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { cn } from "@/lib/utils";

export type TableDensity = "compact" | "normal" | "comfortable";

export interface TableDensityToggleProps {
  className?: string;
  density: TableDensity;
  onDensityChange: (density: TableDensity) => void;
}

export function useTableDensity(
  storageKey = "hrm_table_density",
  defaultDensity: TableDensity = "normal"
) {
  const [density, setDensityState] = useState<TableDensity>(() => {
    if (typeof window === "undefined") return defaultDensity;
    try {
      const saved = localStorage.getItem(storageKey);
      if (
        saved === "compact" ||
        saved === "normal" ||
        saved === "comfortable"
      ) {
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
    [storageKey]
  );

  return { density, setDensity };
}

export function TableDensityToggle({
  density,
  onDensityChange,
  className,
}: TableDensityToggleProps) {
  const { t } = useTranslation();

  const options: Array<{
    id: TableDensity;
    label: string;
    icon: React.ReactNode;
  }> = [
    {
      id: "compact",
      label: t("table.density.compact", "Nhỏ gọn"),
      icon: <AlignJustify className="h-3.5 w-3.5" />,
    },
    {
      id: "normal",
      label: t("table.density.normal", "Tiêu chuẩn"),
      icon: <Rows className="h-3.5 w-3.5" />,
    },
    {
      id: "comfortable",
      label: t("table.density.comfortable", "Rộng rãi"),
      icon: <Maximize2 className="h-3.5 w-3.5" />,
    },
  ];

  return (
    <div
      aria-label={t("table.density.label", "Mật độ hiển thị bảng")}
      className={cn(
        "inline-flex items-center p-1 rounded-xl bg-slate-100 dark:bg-slate-800/80 border border-slate-200/80 dark:border-slate-700/80 shadow-xs",
        className
      )}
      data-testid="table-density-toggle"
      role="group"
    >
      {options.map((opt) => {
        const isActive = density === opt.id;
        return (
          <Button
            aria-pressed={isActive}
            className={cn(
              "h-7 px-2.5 rounded-lg text-xs font-semibold gap-1.5 transition-all shadow-none",
              isActive
                ? "bg-white dark:bg-slate-900 text-primary-700 dark:text-primary-300 shadow-xs font-bold"
                : "text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 bg-transparent"
            )}
            data-testid={`density-btn-${opt.id}`}
            key={opt.id}
            onClick={() => onDensityChange(opt.id)}
            size="sm"
            variant="ghost"
          >
            {opt.icon}
            <span className="hidden sm:inline">{opt.label}</span>
          </Button>
        );
      })}
    </div>
  );
}
