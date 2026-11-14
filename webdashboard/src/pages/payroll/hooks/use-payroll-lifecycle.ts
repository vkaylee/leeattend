import { useState, useCallback } from "react";

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

function readAuditFromStorage(year: number, month: number, hasCalculatedPayslips: boolean): PayrollPeriodAudit {
  if (typeof window === "undefined") {
    return { status: "DRAFT" };
  }
  try {
    const stored = localStorage.getItem(getStorageKey(year, month));
    const parsed: unknown = stored ? JSON.parse(stored) : null;
    if (isPayrollPeriodAudit(parsed)) {
      return parsed;
    }
  } catch {
    // Fallback on storage read error
  }
  return { status: hasCalculatedPayslips ? "CALCULATED" : "DRAFT" };
}

export function usePayrollLifecycle({
  calcMonth,
  calcYear,
  hasCalculatedPayslips,
}: UsePayrollLifecycleOptions) {
  const [audit, setAudit] = useState<PayrollPeriodAudit>(() =>
    readAuditFromStorage(calcYear, calcMonth, hasCalculatedPayslips)
  );
  const [prevPeriod, setPrevPeriod] = useState({ calcMonth, calcYear, hasCalculatedPayslips });

  if (
    prevPeriod.calcYear !== calcYear ||
    prevPeriod.calcMonth !== calcMonth ||
    prevPeriod.hasCalculatedPayslips !== hasCalculatedPayslips
  ) {
    setPrevPeriod({ calcMonth, calcYear, hasCalculatedPayslips });
    setAudit(readAuditFromStorage(calcYear, calcMonth, hasCalculatedPayslips));
  }

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
