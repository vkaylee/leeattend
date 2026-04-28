import { useState, useEffect, useCallback } from "react";

const CHECK_INTERVAL_MS = 5 * 60 * 1000;
const CHECK_ON_VISIBLE = true;

declare const __BUILD_HASH__: string;

export function useVersionCheck() {
  const [hasNewVersion, setHasNewVersion] = useState(false);

  const check = useCallback(async () => {
    try {
      const current = __BUILD_HASH__;
      if (!current) return;

      const res = await fetch(`/?_v=${Date.now()}`, {
        method: "GET",
        headers: { "Cache-Control": "no-cache" },
      });
      const html = await res.text();
      const match = html.match(
        /<meta\s+name="build-version"\s+content="([^"]+)"/i,
      );
      if (match && match[1] !== current) {
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
    run();

    const interval = setInterval(() => {
      if (!cancelled) check();
    }, CHECK_INTERVAL_MS);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [check]);

  useEffect(() => {
    if (!CHECK_ON_VISIBLE) return;

    const handleVisibility = () => {
      if (document.visibilityState === "visible") check();
    };
    document.addEventListener("visibilitychange", handleVisibility);
    return () =>
      document.removeEventListener("visibilitychange", handleVisibility);
  }, [check]);

  const dismiss = useCallback(() => setHasNewVersion(false), []);

  return { hasNewVersion, dismiss };
}
