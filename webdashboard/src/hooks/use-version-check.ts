import { useState, useEffect, useCallback, useRef } from "react";

import { logError } from "@/lib/logger";

const CHECK_INTERVAL_MS = 5 * 60 * 1000;
const DISMISSED_VERSION_KEY = "dismissedVersion";
const RELOAD_ATTEMPT_KEY = "versionReloadAttempt";
const RELOAD_ATTEMPT_TTL_MS = 10 * 60 * 1000;

declare const __BUILD_HASH__: string;

// ── Safe sessionStorage wrapper ──────────────────────────────────────────────
// sessionStorage throws in private browsing and some restricted environments.
// This helper consolidates all get/set/remove with silent fallback on failure.

function getSessionItem(key: string): string | null {
  try {
    return sessionStorage.getItem(key);
  } catch { /* sessionStorage unavailable (private browsing) */ return null; }
}

function setSessionItem(key: string, value: string): void {
  try {
    sessionStorage.setItem(key, value);
  } catch { /* sessionStorage unavailable — persist silently fails */ }
}

function removeSessionItem(key: string): void {
  try {
    sessionStorage.removeItem(key);
  } catch { /* sessionStorage unavailable */ }
}

function getDismissedVersion(): string | null {
  return getSessionItem(DISMISSED_VERSION_KEY);
}

function setDismissedVersion(hash: string): void {
  setSessionItem(DISMISSED_VERSION_KEY, hash);
}

interface ReloadAttempt {
  attemptedAt: number;
  buildId: string;
}

function getReloadAttempt(): ReloadAttempt | null {
  const raw = getSessionItem(RELOAD_ATTEMPT_KEY);
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as Partial<ReloadAttempt>;
    if (typeof parsed.buildId !== "string" || typeof parsed.attemptedAt !== "number") {
      removeSessionItem(RELOAD_ATTEMPT_KEY);
      return null;
    }
    if (Date.now() - parsed.attemptedAt > RELOAD_ATTEMPT_TTL_MS) {
      removeSessionItem(RELOAD_ATTEMPT_KEY);
      return null;
    }
    return parsed as ReloadAttempt;
  } catch { /* JSON.parse on corrupted sessionStorage — clean up and move on */
    removeSessionItem(RELOAD_ATTEMPT_KEY);
    return null;
  }
}

function setReloadAttempt(buildId: string): void {
  setSessionItem(RELOAD_ATTEMPT_KEY, JSON.stringify({ buildId, attemptedAt: Date.now() }));
}

function clearReloadAttempt(): void {
  removeSessionItem(RELOAD_ATTEMPT_KEY);
}

/** Extract and validate build ID from manifest JSON response. */
async function fetchServerBuildId(): Promise<string | null> {
  try {
    const res = await fetch(`/version.json?_v=${Date.now()}`, {
      method: "GET",
      cache: "no-store",
      headers: { "Cache-Control": "no-cache" },
    });
    if (!res.ok) return null;
    const contentType = res.headers.get("content-type");
    if (contentType && !contentType.includes("application/json")) {
      return null;
    }
    const manifest = (await res.json()) as { buildId?: unknown };
    if (typeof manifest.buildId === "string" && manifest.buildId.length > 0) {
      return manifest.buildId;
    }
    return null;
  } catch {
    return null;
  }
}

export function useVersionCheck() {
  const [hasNewVersion, setHasNewVersion] = useState(false);
  const latestServerHash = useRef<string | null>(null);
  const pendingHash = useRef<string | null>(null);

  const check = useCallback(async () => {
    try {
      const current = __BUILD_HASH__;
      if (!current) return;

      const serverHash = await fetchServerBuildId();
      if (!serverHash) return;

      // Server serves the build we're running — nothing new; clear any
      // half-confirmed sighting.
      if (serverHash === current) {
        pendingHash.current = null;
        latestServerHash.current = null;
        clearReloadAttempt();
        setHasNewVersion(false);
        return;
      }

      const reloadAttempt = getReloadAttempt();
      if (reloadAttempt?.buildId === serverHash) {
        pendingHash.current = null;
        setHasNewVersion(false);
        return;
      }

      // A different hash appeared. Require the SAME new hash on two
      // consecutive checks before trusting it, so a half-propagated deploy
      // (one edge node briefly serving the new build, then flapping back to
      // the old one) doesn't surface the banner prematurely.
      if (pendingHash.current !== serverHash) {
        pendingHash.current = serverHash;
        return;
      }

      // Confirmed on two consecutive checks — the deploy looks stable.
      latestServerHash.current = serverHash;
      if (serverHash !== getDismissedVersion()) {
        setHasNewVersion(true);
      }
    } catch (e: unknown) {
      // Network error or fetch failure — skip this check cycle
      logError(e, { component: 'useVersionCheck' });
    }
  }, []);

  useEffect(() => {
    let cancelled = false;

    const run = async () => {
      await check();
    };
    void run();

    const interval = setInterval(() => {
      if (!cancelled) void check();
    }, CHECK_INTERVAL_MS);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [check]);

  useEffect(() => {
    const handleVisibility = () => {
      if (document.visibilityState === "visible") void check();
    };
    document.addEventListener("visibilitychange", handleVisibility);
    return () =>
      { document.removeEventListener("visibilitychange", handleVisibility); };
  }, [check]);

  const dismiss = useCallback(() => {
    if (latestServerHash.current) {
      setDismissedVersion(latestServerHash.current);
    }
    setHasNewVersion(false);
  }, []);

  const refresh = useCallback(() => {
    const targetBuildId = latestServerHash.current;
    if (!targetBuildId) return;
    setReloadAttempt(targetBuildId);
    const url = new URL(window.location.href);
    url.searchParams.set("_build", targetBuildId);
    window.location.replace(url.toString());
  }, []);

  return { hasNewVersion, dismiss, refresh };
}
