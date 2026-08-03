// Detects errors thrown when the browser fails to load a code-split JS/CSS
// chunk — the classic symptom of a stale client running after a new deploy
// replaced the hashed chunk files. Kept deliberately narrow: matching on the
// stack trace (which contains ".js" for almost any error) would misfire on
// ordinary runtime errors and trigger spurious reloads.
export function isChunkLoadError(reason: Error | undefined): boolean {
  if (!reason) return false;
  if (reason.name === "ChunkLoadError") return true;
  const message = reason.message;
  return (
    /Loading chunk [\d]+ failed/i.test(message) ||
    /Loading CSS chunk [\w-]+ failed/i.test(message) ||
    message.includes("Failed to fetch dynamically imported module") ||
    message.includes("error loading dynamically imported module") ||
    message.includes("Importing a module script failed")
  );
}

const CHUNK_RETRY_KEY = "chunkRetry";
const MAX_CHUNK_RETRIES = 3;
const STABLE_RUNTIME_MS = 60_000;

// Records one more chunk-load reload attempt and reports whether another
// reload is still within budget. Returns false once the cap is reached so a
// genuinely-broken deploy shows the error boundary instead of looping forever.
export function shouldRetryChunkLoad(): boolean {
  let retryCount = 0;
  try {
    retryCount = Number(sessionStorage.getItem(CHUNK_RETRY_KEY) ?? "0");
  } catch { /* sessionStorage unavailable (private browsing) — can't track retries, allow one reload */ return false;
  }
  if (retryCount >= MAX_CHUNK_RETRIES) return false;
  try {
    sessionStorage.setItem(CHUNK_RETRY_KEY, String(retryCount + 1));
  } catch { /* sessionStorage unavailable — allow this one reload, can't persist count */
  }
  return true;
}

// Called once the app has mounted successfully: the client is on a healthy
// bundle, so reset the counter and give a future deploy a fresh retry budget.
export function clearChunkRetry(): void {
  try {
    sessionStorage.removeItem(CHUNK_RETRY_KEY);
  } catch { /* sessionStorage unavailable — ignore */
  }
}

/**
 * Reset the retry budget only after the mounted application has stayed alive.
 * A chunk can fail after React mounts, so clearing the budget synchronously
 * would allow a stale deploy to reload forever.
 */
export function scheduleChunkRetryReset(
  delayMs = STABLE_RUNTIME_MS,
): () => void {
  const timerId = window.setTimeout(clearChunkRetry, delayMs);
  return () => { window.clearTimeout(timerId); };
}
