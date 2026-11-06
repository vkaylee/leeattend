import { CheckCircle2, Info, MapPin, ShieldCheck, Wifi } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

interface PunchGeofenceStatusProps {
  className?: string;
  gpsRadiusMeters?: number;
  isGpsValid?: boolean;
  isWifiValid?: boolean;
  locationName?: string;
  wifiSsid?: string;
}

export function PunchGeofenceStatus({
  className,
  isWifiValid = true,
  wifiSsid = "LeeAttend-Corp-5G",
  isGpsValid = true,
  gpsRadiusMeters = 18,
  locationName = "Trụ sở chính - Tầng 12",
}: PunchGeofenceStatusProps) {
  const { t } = useTranslation();

  return (
    <div
      aria-label={t("ess.clock.geofence_status_label", "Trạng thái vị trí chấm công")}
      className={cn(
        "flex flex-wrap items-center justify-between gap-3 p-4 rounded-2xl bg-slate-50/90 dark:bg-slate-800/40 border border-slate-200/80 dark:border-slate-800 text-xs",
        className
      )}
      data-testid="punch-geofence-status"
    >
      <div className="flex flex-wrap items-center gap-3">
        {/* Wi-Fi Verification Chip */}
        <div
          className={cn(
            "flex items-center gap-1.5 px-3 py-1 rounded-xl border font-semibold",
            isWifiValid
              ? "bg-emerald-50 dark:bg-emerald-950/60 text-emerald-800 dark:text-emerald-300 border-emerald-200/80 dark:border-emerald-800/80"
              : "bg-rose-50 dark:bg-rose-950/60 text-rose-800 dark:text-rose-300 border-rose-200 dark:border-rose-800"
          )}
          data-testid="wifi-status-badge"
        >
          <Wifi className="h-3.5 w-3.5" />
          <span>
            {wifiSsid}{" "}
            {isWifiValid
              ? t("ess.clock.wifi_verified", "(Wi-Fi hợp lệ)")
              : t("ess.clock.wifi_invalid", "(Ngoài mạng)")}
          </span>
        </div>

        {/* GPS Geofence Verification Chip */}
        <div
          className={cn(
            "flex items-center gap-1.5 px-3 py-1 rounded-xl border font-semibold",
            isGpsValid
              ? "bg-sky-50 dark:bg-sky-950/60 text-sky-800 dark:text-sky-300 border-sky-200/80 dark:border-sky-800/80"
              : "bg-rose-50 dark:bg-rose-950/60 text-rose-800 dark:text-rose-300 border-rose-200 dark:border-rose-800"
          )}
          data-testid="gps-status-badge"
        >
          <MapPin className="h-3.5 w-3.5" />
          <span>
            {locationName} (~{gpsRadiusMeters}m)
          </span>
        </div>
      </div>

      {/* Security & Grace Period Notice */}
      <div className="flex items-center gap-1.5 text-slate-500 dark:text-slate-400 font-medium">
        <ShieldCheck className="h-4 w-4 text-primary-500" />
        <span className="hidden sm:inline">
          {t("ess.clock.security_notice", "Vị trí được xác thực tự động")}
        </span>
      </div>
    </div>
  );
}
