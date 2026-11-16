import { Link } from "@tanstack/react-router";
import { Play, Square, ExternalLink, Calendar, CheckCircle2 } from "lucide-react";
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { useMyProfile } from "@/hooks/use-ess";
import { useMyClock } from "@/hooks/use-my-attendance";
import { ROUTES } from "@/lib/routes";

export function QuickPunchWidget() {
  const { t } = useTranslation();
  const { success, error } = useToast();
  const { employee, isLoading: isLoadingProfile } = useMyProfile();
  const { activeEntry, isPunching, punch, isLoading } = useMyClock({
    enabled: !isLoadingProfile && Boolean(employee?.active_employment_id),
    suppressErrors: true,
  });
  const [currentTime, setCurrentTime] = useState(() => new Date());

  useEffect(() => {
    const timer = setInterval(() => { setCurrentTime(new Date()); }, 1000);
    return () => { clearInterval(timer); };
  }, []);

  const isClockedIn = Boolean(activeEntry && !activeEntry.punch_out_time);

  if (isLoadingProfile) {
    return <div className="h-full min-h-36 animate-pulse rounded-2xl bg-slate-100 dark:bg-slate-800" />;
  }

  if (!employee) {
    return (
      <div className="flex h-full min-h-36 items-center justify-center text-center">
        <span className="text-sm text-slate-500 dark:text-slate-400">
          {t("ess.noEmployeeRecord", "Chưa có hồ sơ nhân viên để chấm công")}
        </span>
      </div>
    );
  }

  const handlePunch = async () => {
    try {
      await punch();
      if (isClockedIn) {
        success(t("ess.punchOutSuccess", "Đã ghi nhận tan ca"));
      } else {
        success(t("ess.punchInSuccess", "Đã ghi nhận vào ca thành công"));
      }
    } catch {
      error(
        t("ess.punchErrorDesc", "Vui lòng thử lại hoặc kiểm tra kết nối mạng"),
        t("ess.punchError", "Không thể thực hiện chấm công"),
      );
    }
  };

  const timeString = currentTime.toLocaleTimeString("vi-VN", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });

  const dateString = currentTime.toLocaleDateString("vi-VN", {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
  });

  return (
    <div className="flex flex-col justify-between h-full space-y-4">
      {/* Live Digital Clock & Date */}
      <div className="flex items-center justify-between">
        <div className="space-y-1">
          <div className="text-3xl sm:text-4xl font-extrabold tracking-tight font-mono text-slate-800 dark:text-slate-100">
            {timeString}
          </div>
          <div className="flex items-center gap-1.5 text-xs text-slate-500 dark:text-slate-400 capitalize">
            <Calendar className="w-3.5 h-3.5 text-primary-500" />
            <span>{dateString}</span>
          </div>
        </div>

        {/* Current status pill */}
        <div className="text-right">
          <span
            className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold ${
              isClockedIn
                ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20"
                : "bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 border border-slate-200 dark:border-slate-700"
            }`}
          >
            <span
              className={`w-2 h-2 rounded-full ${
                isClockedIn ? "bg-emerald-500 animate-pulse" : "bg-slate-400"
              }`}
            />
            {isClockedIn
              ? t("ess.clockedIn", "Đang trong ca")
              : t("ess.notClockedIn", "Chưa vào ca")}
          </span>
        </div>
      </div>

      {/* Active entry details if in shift */}
      {!!isClockedIn && !!activeEntry && (
        <div className="p-3 rounded-2xl bg-emerald-50/60 dark:bg-emerald-950/20 border border-emerald-100 dark:border-emerald-900/40 text-xs text-emerald-800 dark:text-emerald-300 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 text-emerald-500" />
            <span>
              {t("ess.startedAt", "Vào ca lúc")}:{" "}
              <strong>
                {activeEntry.punch_in_time
                  ? new Date(activeEntry.punch_in_time).toLocaleTimeString("vi-VN", {
                      hour: "2-digit",
                      minute: "2-digit",
                    })
                  : "—"}
              </strong>
            </span>
          </div>
        </div>
      )}

      {/* Action buttons */}
      <div className="flex items-center gap-3 pt-2">
        <Button
          className={`flex-1 h-11 rounded-2xl text-sm font-bold flex items-center justify-center gap-2 ${
            isClockedIn
              ? "bg-rose-500/10 hover:bg-rose-500/20 text-rose-600 dark:text-rose-400 border border-rose-500/20"
              : "shadow-md hover:shadow-lg"
          }`}
          data-testid="desktop-quick-punch-btn"
          loading={isPunching || isLoading}
          onClick={handlePunch}
          type="button"
          variant={isClockedIn ? "secondary" : "primary"}
        >
          {isClockedIn ? (
            <>
              <Square className="w-4 h-4 fill-current" />
              <span>{t("ess.clockOutNow", "Bấm giờ tan ca")}</span>
            </>
          ) : (
            <>
              <Play className="w-4 h-4 fill-current" />
              <span>{t("ess.clockInNow", "Bấm giờ vào ca")}</span>
            </>
          )}
        </Button>

        <Link
          aria-label={t("ess.viewMyClock", "Xem chi tiết chấm công")}
          className="p-3 rounded-2xl bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors flex items-center justify-center"
          to={ROUTES.ESS_CLOCK}
        >
          <ExternalLink className="w-4 h-4" />
        </Link>
      </div>
    </div>
  );
}
