import { useSyncExternalStore, useMemo } from "react";

/**
 * Parses a device-time string ("YYYY-MM-DD HH:mm:ss") into epoch ms.
 * Returns NaN if the string is invalid.
 */
function parseDeviceTime(dt: string): number {
    // "2026-03-08 10:00:00" → "2026-03-08T10:00:00" so Date.parse works consistently
    return new Date(dt.replace(" ", "T")).getTime();
}

/**
 * Formats a Date into "YYYY-MM-DD HH:mm:ss".
 */
function formatTime(d: Date): string {
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

/**
 * Subscribe to a 1-second ticker via useSyncExternalStore.
 * This avoids setState-in-useEffect (react-hooks/set-state-in-effect).
 */
function subscribeToTicker(onStoreChange: () => void): () => void {
    const id = setInterval(onStoreChange, 1000);
    return () => { clearInterval(id); };
}

function getTickerSnapshot(): number {
    return Date.now();
}

/**
 * Returns a ticking device-time string that advances every second.
 *
 * @param deviceTime  - The last-known device time string, e.g. "2026-03-08 10:00:00"
 * @param lastSync    - Timestamp string of when the agent last read the device (e.g. last_sync field)
 * @returns A string that ticks every second, or null when inputs are missing/invalid.
 *
 * How it works:
 *   offset = parse(deviceTime) - parse(lastSync)
 *   displayTime = new Date(Date.now() + offset)
 *
 * Using lastSync (server-side timestamp) instead of browser's Date.now()
 * ensures the offset is correct even when the cached data is stale (e.g.
 * after page reload). Both timestamps come from the same sync event, so
 * the offset represents the real clock difference at that moment.
 */
export function useTickingDeviceTime(
    deviceTime: string | undefined | null,
    lastSync: string | undefined | null,
): string | null {
    const now = useSyncExternalStore(subscribeToTicker, getTickerSnapshot);

    // Compute offset (stable between renders as long as inputs don't change)
    const offset = useMemo(() => {
        if (!deviceTime || !lastSync) return null;
        const deviceEpoch = parseDeviceTime(deviceTime);
        const syncEpoch = parseDeviceTime(lastSync);
        if (Number.isNaN(deviceEpoch) || Number.isNaN(syncEpoch)) return null;
        return deviceEpoch - syncEpoch;
    }, [deviceTime, lastSync]);

    if (offset === null) return null;

    return formatTime(new Date(now + offset));
}

