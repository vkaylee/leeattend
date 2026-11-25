import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { EmployeeMatchCandidate, MatchEmployeeRequest } from "@/lib/api";

import { matchEmployeeIdentity } from "@/lib/api";
import { ApiError } from "@/lib/api/errors";

export type PreflightStatus = "idle" | "loading" | "none" | "hard" | "soft" | "stale" | "error";
export type PreflightErrorKind = "rate_limited" | "unavailable" | "forbidden" | "generic" | null;

export interface IdentityPreflightInput {
  dateOfBirth: string;
  debounceMs?: number;
  enabled: boolean;
  fullName: string;
  nationalId: string;
  phone: string;
}

export interface IdentityPreflightState {
  candidates: EmployeeMatchCandidate[];
  errorKind: PreflightErrorKind;
  hasCriteria: boolean;
  retry: () => void;
  status: PreflightStatus;
}

interface MatchSnapshot {
  candidates: EmployeeMatchCandidate[];
  errorKind: PreflightErrorKind;
  signature: string;
  tier: "none" | "hard" | "soft" | "error";
}

const ISO_DATE_RE = /^\d{4}-\d{2}-\d{2}$/;
const PHONE_FORMAT_RE = /[\s().\t-]/g;
const NATIONAL_ID_SEP_RE = /[\s./_\-‐-―−]/g;

function applyPhoneCriteria(
  payload: MatchEmployeeRequest,
  trimmedPhone: string,
  hasName: boolean,
): { hasHard: boolean; hasSoft: boolean } {
  if (!trimmedPhone) return { hasHard: false, hasSoft: false };
  if (trimmedPhone.startsWith("+")) {
    const digits = trimmedPhone.slice(1).replace(PHONE_FORMAT_RE, "");
    if (/^[1-9]\d{6,14}$/.test(digits)) {
      payload.phone = `+${digits}`;
      return { hasHard: true, hasSoft: false };
    }
    return { hasHard: false, hasSoft: false };
  }
  if (!hasName) return { hasHard: false, hasSoft: false };
  const partialDigits = trimmedPhone.replace(PHONE_FORMAT_RE, "");
  if (/^\d{4,15}$/.test(partialDigits)) {
    payload.partial_phone = partialDigits;
    return { hasHard: false, hasSoft: true };
  }
  return { hasHard: false, hasSoft: false };
}

/**
 * Builds a bounded POST body for /api/v1/employees/match from identity fields only.
 * Enrollment/raw device IDs are intentionally excluded and never treated as identity proof.
 */
export function buildIdentityMatchPayload(
  input: Pick<IdentityPreflightInput, "dateOfBirth" | "fullName" | "nationalId" | "phone">,
): MatchEmployeeRequest | null {
  const payload: MatchEmployeeRequest = {};
  const trimmedName = input.fullName.trim();
  const trimmedNationalId = input.nationalId.trim();
  const trimmedDob = input.dateOfBirth.trim();

  const hasNationalId = trimmedNationalId.replace(NATIONAL_ID_SEP_RE, "").length > 0;
  if (hasNationalId) payload.national_id = trimmedNationalId;

  const phoneMatch = applyPhoneCriteria(payload, input.phone.trim(), trimmedName.length > 0);
  const hasDobSoft = trimmedName.length > 0 && ISO_DATE_RE.test(trimmedDob);
  if (hasDobSoft) payload.date_of_birth = trimmedDob;

  const hasAny = hasNationalId || phoneMatch.hasHard || phoneMatch.hasSoft || hasDobSoft;
  if (trimmedName.length > 0 && hasAny) payload.full_name = trimmedName;
  return hasAny ? payload : null;
}

function classifyPreflightError(error: unknown): Exclude<PreflightErrorKind, null> {
  if (error instanceof ApiError) {
    if (error.status === 429) return "rate_limited";
    if (error.status === 503) return "unavailable";
    if (error.status === 403) return "forbidden";
  }
  return "generic";
}

function derivePreflightView(
  activeSignature: string,
  loadingSignature: string | null,
  snapshot: MatchSnapshot | null,
): Pick<IdentityPreflightState, "candidates" | "errorKind" | "status"> {
  if (!activeSignature) {
    return { candidates: [], errorKind: null, status: "idle" };
  }
  if (loadingSignature === activeSignature) {
    return { candidates: [], errorKind: null, status: "loading" };
  }
  if (snapshot?.signature === activeSignature) {
    return { candidates: snapshot.candidates, errorKind: snapshot.errorKind, status: snapshot.tier };
  }
  if (snapshot) {
    return { candidates: snapshot.candidates, errorKind: null, status: "stale" };
  }
  return { candidates: [], errorKind: null, status: "idle" };
}

export function useEmployeeIdentityPreflight({
  dateOfBirth,
  debounceMs = 350,
  enabled,
  fullName,
  nationalId,
  phone,
}: IdentityPreflightInput): IdentityPreflightState {
  const payload = useMemo(
    () => (enabled ? buildIdentityMatchPayload({ dateOfBirth, fullName, nationalId, phone }) : null),
    [dateOfBirth, enabled, fullName, nationalId, phone],
  );
  const signature = useMemo(() => (payload ? JSON.stringify(payload) : ""), [payload]);

  const [snapshot, setSnapshot] = useState<MatchSnapshot | null>(null);
  const [loadingSignature, setLoadingSignature] = useState<string | null>(null);
  const [retryToken, setRetryToken] = useState(0);

  const requestSeqRef = useRef(0);

  const retry = useCallback(() => {
    setRetryToken((prev) => prev + 1);
  }, []);

  useEffect(() => {
    if (!enabled || !payload || !signature) {
      requestSeqRef.current += 1;
      return;
    }

    const seq = ++requestSeqRef.current;
    const timer = window.setTimeout(() => {
      setLoadingSignature(signature);
      void matchEmployeeIdentity(payload)
        .then((response) => {
          if (seq !== requestSeqRef.current) return;
          setLoadingSignature(null);
          setSnapshot({
            candidates: response.candidates,
            errorKind: null,
            signature,
            tier: response.tier as "none" | "soft" | "hard" | "error",
          });
        })
        .catch((error: unknown) => {
          if (seq !== requestSeqRef.current) return;
          setLoadingSignature(null);
          setSnapshot({
            candidates: [],
            errorKind: classifyPreflightError(error),
            signature,
            tier: "error",
          });
        });
    }, debounceMs);

    return () => {
      window.clearTimeout(timer);
    };
  }, [debounceMs, enabled, payload, retryToken, signature]);

  const view = derivePreflightView(signature, loadingSignature, snapshot);

  return {
    ...view,
    hasCriteria: Boolean(payload),
    retry,
  };
}
