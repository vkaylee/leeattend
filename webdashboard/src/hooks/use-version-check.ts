import { useState, useEffect, useCallback, useRef } from "react";

const CHECK_INTERVAL_MS = 5 * 60 * 1000;
const DISMISSED_VERSION_KEY = "dismissedVersion";

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

export function useVersionCheck() {
  const [hasNewVersion, setHasNewVersion] = useState(false);
  const latestServerHash = useRef<string | null>(null);
  const pendingHash = useRef<string | null>(null);

  const check = useCallback(async () => {
    try {
      const current = __BUILD_HASH__;
      if (!current) return;

      const res = await fetch(`/?_v=${Date.now()}`, {
        method: "GET",
        headers: { "Cache-Control": "no-cache" },
      });
      const html = await res.text();
      const match = /<meta\s+name="build-version"\s+content="([^"]+)"/i.exec(html);
      if (!match) return;

      const serverHash = match[1];

      // Server serves the build we're running — nothing new; clear any
      // half-confirmed sighting.
      if (serverHash === current) {
        pendingHash.current = null;
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

  return { hasNewVersion, dismiss };
}
