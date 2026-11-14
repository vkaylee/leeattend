import { useState, useEffect, useCallback } from "react";

export type PayrollPeriodStatus = "DRAFT" | "CALCULATED" | "LOCKED";

export interface PayrollPeriodAudit {
  lockedAt?: string | null;
  lockedBy?: string | null;
  status: PayrollPeriodStatus;
}

interface UsePayrollLifecycleOptions { calcMonth: number;
calcYear: number;
hasCalculatedPayslips: boolean;  }

const STORAGE_PREFIX = "leeattend_payroll_lock";

function isPayrollPeriodAudit(value: unknown): value is PayrollPeriodAudit {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Record<string, unknown>;
  return candidate.status === "DRAFT" || candidate.status === "CALCULATED" || candidate.status === "LOCKED";
}

function getStorageKey(year: number, month: number): string {
  return `${STORAGE_PREFIX}_${year}_${month}`;
}

export function usePayrollLifecycle({
  calcMonth,
  calcYear,
  hasCalculatedPayslips,
}: UsePayrollLifecycleOptions) {
  const [audit, setAudit] = useState<PayrollPeriodAudit>(() => {
    if (typeof window === "undefined") {
      return { status: "DRAFT" };
    }
    try {
      const stored = localStorage.getItem(getStorageKey(calcYear, calcMonth));
      const parsed: unknown = stored ? JSON.parse(stored) : null;
      if (isPayrollPeriodAudit(parsed)) {
        return parsed;
      }
    } catch {
      // Fallback on storage read error
    }
    return { status: hasCalculatedPayslips ? "CALCULATED" : "DRAFT" };
  });

  // Sync state when period or calculation presence changes
  useEffect(() => {
    
    try {
      const stored = localStorage.getItem(getStorageKey(calcYear, calcMonth));
      const parsed: unknown = stored ? JSON.parse(stored) : null;
      if (isPayrollPeriodAudit(parsed)) {
        setAudit(parsed);
      } else {
        setAudit({
          status: hasCalculatedPayslips ? "CALCULATED" : "DRAFT",
        });
      }
    } catch {
      setAudit({
        status: hasCalculatedPayslips ? "CALCULATED" : "DRAFT",
      });
    }
  }, [calcYear, calcMonth, hasCalculatedPayslips]);

  const persistAudit = useCallback((newAudit: PayrollPeriodAudit) => {
    setAudit(newAudit);
    
    try {
      localStorage.setItem(getStorageKey(calcYear, calcMonth), JSON.stringify(newAudit));
    } catch {
      // Safely ignore storage quotas
    }
  }, [calcYear, calcMonth]);

  const lockPeriod = useCallback((operatorName = "HR Manager") => {
    const newAudit: PayrollPeriodAudit = {
      status: "LOCKED",
      lockedAt: new Date().toISOString(),
      lockedBy: operatorName,
    };
    persistAudit(newAudit);
  }, [persistAudit]);

  const unlockPeriod = useCallback(() => {
    const newAudit: PayrollPeriodAudit = {
      status: hasCalculatedPayslips ? "CALCULATED" : "DRAFT",
      lockedAt: null,
      lockedBy: null,
    };
    persistAudit(newAudit);
  }, [persistAudit, hasCalculatedPayslips]);

  return {
    audit,
    isLocked: audit.status === "LOCKED",
    lockPeriod,
    status: audit.status,
    unlockPeriod,
  };
}
