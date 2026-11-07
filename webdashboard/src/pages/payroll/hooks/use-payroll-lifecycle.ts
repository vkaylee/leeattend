import { useState, useEffect, useCallback } from "react";

export type PayrollPeriodStatus = "DRAFT" | "CALCULATED" | "LOCKED";

export interface PayrollPeriodAudit {
  status: PayrollPeriodStatus;
  lockedAt?: string | null;
  lockedBy?: string | null;
}

interface UsePayrollLifecycleOptions {
  calcMonth: number;
  calcYear: number;
  hasCalculatedPayslips: boolean;
  wsId: string;
}

const STORAGE_PREFIX = "leeattend_payroll_lock";

function getStorageKey(wsId: string, year: number, month: number): string {
  return `${STORAGE_PREFIX}_${wsId}_${year}_${month}`;
}

export function usePayrollLifecycle({
  calcMonth,
  calcYear,
  hasCalculatedPayslips,
  wsId,
}: UsePayrollLifecycleOptions) {
  const [audit, setAudit] = useState<PayrollPeriodAudit>(() => {
    if (typeof window === "undefined" || !wsId) {
      return { status: "DRAFT" };
    }
    try {
      const stored = localStorage.getItem(getStorageKey(wsId, calcYear, calcMonth));
      if (stored) {
        return JSON.parse(stored) as PayrollPeriodAudit;
      }
    } catch {
      // Fallback on storage read error
    }
    return { status: hasCalculatedPayslips ? "CALCULATED" : "DRAFT" };
  });

  // Sync state when period or calculation presence changes
  useEffect(() => {
    if (!wsId) return;
    try {
      const stored = localStorage.getItem(getStorageKey(wsId, calcYear, calcMonth));
      if (stored) {
        setAudit(JSON.parse(stored));
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
  }, [wsId, calcYear, calcMonth, hasCalculatedPayslips]);

  const persistAudit = useCallback((newAudit: PayrollPeriodAudit) => {
    setAudit(newAudit);
    if (!wsId) return;
    try {
      localStorage.setItem(getStorageKey(wsId, calcYear, calcMonth), JSON.stringify(newAudit));
    } catch {
      // Safely ignore storage quotas
    }
  }, [wsId, calcYear, calcMonth]);

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
