import { Link } from "@tanstack/react-router";
import { Calendar, CheckCircle2, ExternalLink, Play, Square } from "lucide-react";
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { useMyProfile } from "@/hooks/use-ess";
import { useMyClock } from "@/hooks/use-my-attendance";
import { useMyPermissions } from "@/hooks/use-permissions";
import { ROUTES } from "@/lib/routes";

type Translator = ReturnType<typeof useTranslation>["t"];

function ClockDisplay({ currentTime }: { currentTime: Date }) {
  return (
    <div className="space-y-1">
      <div className="text-3xl sm:text-4xl font-extrabold tracking-tight font-mono text-slate-800 dark:text-slate-100">
        {currentTime.toLocaleTimeString("vi-VN", { hour: "2-digit", minute: "2-digit", second: "2-digit" })}
      </div>
      <div className="flex items-center gap-1.5 text-xs text-slate-500 dark:text-slate-400 capitalize">
        <Calendar className="w-3.5 h-3.5 text-primary-500" />
        <span>{currentTime.toLocaleDateString("vi-VN", { weekday: "long", year: "numeric", month: "long", day: "numeric" })}</span>
      </div>
    </div>
  );
}

function ClockStatus({ isClockedIn, t }: { isClockedIn: boolean; t: Translator }) {
  const statusClass = isClockedIn
    ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20"
    : "bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 border border-slate-200 dark:border-slate-700";
  return (
    <div className="text-right">
      <span className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold ${statusClass}`}>
        <span className={`w-2 h-2 rounded-full ${isClockedIn ? "bg-emerald-500 animate-pulse" : "bg-slate-400"}`} />
        {t(isClockedIn ? "ess.clockedIn" : "ess.notClockedIn", isClockedIn ? "Đang trong ca" : "Chưa vào ca")}
      </span>
    </div>
  );
}

function ActiveEntryDetails({ punchInTime, t }: { punchInTime: string | null; t: Translator }) {
  const time = punchInTime
    ? new Date(punchInTime).toLocaleTimeString("vi-VN", { hour: "2-digit", minute: "2-digit" })
    : "—";
  return (
    <div className="p-3 rounded-2xl bg-emerald-50/60 dark:bg-emerald-950/20 border border-emerald-100 dark:border-emerald-900/40 text-xs text-emerald-800 dark:text-emerald-300 flex items-center justify-between">
      <div className="flex items-center gap-2">
        <CheckCircle2 className="w-4 h-4 text-emerald-500" />
        <span>{t("ess.startedAt", "Vào ca lúc")}: <strong>{time}</strong></span>
      </div>
    </div>
  );
}

function PunchAction({ isClockedIn, isPunching, isLoading, onPunch, t }: { isClockedIn: boolean; isLoading: boolean; isPunching: boolean; onPunch: () => void; t: Translator }) {
  return (
    <Button className={`flex-1 h-11 rounded-2xl text-sm font-bold flex items-center justify-center gap-2 ${isClockedIn ? "bg-rose-500/10 hover:bg-rose-500/20 text-rose-600 dark:text-rose-400 border border-rose-500/20" : "shadow-md hover:shadow-lg"}`} data-testid="desktop-quick-punch-btn" loading={isPunching || isLoading} onClick={onPunch} type="button" variant={isClockedIn ? "secondary" : "primary"}>
      {isClockedIn ? <><Square className="w-4 h-4 fill-current" /><span>{t("ess.clockOutNow", "Bấm giờ tan ca")}</span></> : <><Play className="w-4 h-4 fill-current" /><span>{t("ess.clockInNow", "Bấm giờ vào ca")}</span></>}
    </Button>
  );
}

function NoEmployeeState({ t }: { t: Translator }) {
  return <div className="flex h-full min-h-36 items-center justify-center text-center"><span className="text-sm text-slate-500 dark:text-slate-400">{t("ess.noEmployeeRecord", "Chưa có hồ sơ nhân viên để chấm công")}</span></div>;
}

export function QuickPunchWidget() {
  const { t } = useTranslation();
  const { success, error } = useToast();
  const canUseClock = useMyPermissions().hasRole("employee");
  const { employee, isLoading: isLoadingProfile } = useMyProfile({ enabled: canUseClock });
  const { activeEntry, isPunching, submitAction, isLoading } = useMyClock({
    enabled: canUseClock && !isLoadingProfile && Boolean(employee?.active_employment_id),
    suppressErrors: true,
  });
  const [currentTime, setCurrentTime] = useState(() => new Date());

  useEffect(() => {
    const timer = setInterval(() => { setCurrentTime(new Date()); }, 1000);
    return () => { clearInterval(timer); };
  }, []);

  const isClockedIn = activeEntry !== undefined && !activeEntry.punch_out_time;
  if (isLoadingProfile) return <div className="h-full min-h-36 animate-pulse rounded-2xl bg-slate-100 dark:bg-slate-800" />;
  if (!employee) return <NoEmployeeState t={t} />;

  const handlePunch = async () => {
    try {
      await submitAction(isClockedIn ? "work_out" : "work_in");
      success(t(isClockedIn ? "ess.punchOutSuccess" : "ess.punchInSuccess", isClockedIn ? "Đã ghi nhận tan ca" : "Đã ghi nhận vào ca thành công"));
    } catch {
      error(t("ess.punchErrorDesc", "Vui lòng thử lại hoặc kiểm tra kết nối mạng"), t("ess.punchError", "Không thể thực hiện chấm công"));
    }
  };

  return (
    <div className="flex flex-col justify-between h-full space-y-4">
      <div className="flex items-center justify-between"><ClockDisplay currentTime={currentTime} /><ClockStatus isClockedIn={isClockedIn} t={t} /></div>
      {isClockedIn ? <ActiveEntryDetails punchInTime={activeEntry.punch_in_time} t={t} /> : null}
      <div className="flex items-center gap-3 pt-2">
        <PunchAction isClockedIn={isClockedIn} isLoading={isLoading} isPunching={isPunching} onPunch={handlePunch} t={t} />
        <Link aria-label={t("ess.viewMyClock", "Xem chi tiết chấm công")} className="p-3 rounded-2xl bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors flex items-center justify-center" to={ROUTES.ESS_CLOCK}><ExternalLink className="w-4 h-4" /></Link>
      </div>
    </div>
  );
}
