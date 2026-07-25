import { useState, useEffect, useCallback, useRef } from "react";

const CHECK_INTERVAL_MS = 5 * 60 * 1000;
const DISMISSED_VERSION_KEY = "dismissedVersion";
const RELOAD_ATTEMPT_KEY = "versionReloadAttempt";
const RELOAD_ATTEMPT_TTL_MS = 10 * 60 * 1000;

declare const __BUILD_HASH__: string;

function getDismissedVersion(): string | null {
  try {
    return sessionStorage.getItem(DISMISSED_VERSION_KEY);
  } catch {
    return null;
  }
}

function setDismissedVersion(hash: string): void {
  try {
    sessionStorage.setItem(DISMISSED_VERSION_KEY, hash);
  } catch {
    // sessionStorage unavailable (private mode) — dismiss won't persist
  }
}

interface ReloadAttempt {
  attemptedAt: number;
  buildId: string;
}

function getReloadAttempt(): ReloadAttempt | null {
  try {
    const raw = sessionStorage.getItem(RELOAD_ATTEMPT_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<ReloadAttempt>;
    if (typeof parsed.buildId !== "string" || typeof parsed.attemptedAt !== "number") {
      sessionStorage.removeItem(RELOAD_ATTEMPT_KEY);
      return null;
    }
    if (Date.now() - parsed.attemptedAt > RELOAD_ATTEMPT_TTL_MS) {
      sessionStorage.removeItem(RELOAD_ATTEMPT_KEY);
      return null;
    }
    return parsed as ReloadAttempt;
  } catch {
    return null;
  }
}

function setReloadAttempt(buildId: string): void {
  try {
    sessionStorage.setItem(RELOAD_ATTEMPT_KEY, JSON.stringify({ buildId, attemptedAt: Date.now() }));
  } catch {
    // sessionStorage unavailable; the reload still works without persistence.
  }
}

function clearReloadAttempt(): void {
  try {
    sessionStorage.removeItem(RELOAD_ATTEMPT_KEY);
  } catch {
    // sessionStorage unavailable.
  }
}

/** Extract and validate build ID from manifest JSON response. */
async function fetchServerBuildId(): Promise<string | null> {
  const res = await fetch(`/version.json?_v=${Date.now()}`, {
    method: "GET",
    cache: "no-store",
    headers: { "Cache-Control": "no-cache" },
  });
  if (!res.ok) return null;
  const manifest = await res.json() as { buildId?: unknown };
  if (typeof manifest.buildId === "string" && manifest.buildId.length > 0) {
    return manifest.buildId;
  }
  return null;
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
    } catch {
      // network error — skip check
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
