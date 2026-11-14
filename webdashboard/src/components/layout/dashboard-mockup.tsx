import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";

import { ATTENDANCE_BARS, DAY_LABELS, MOCK_EMPLOYEES, STATUS_STYLE, STAT_CARDS } from "./auth-mock-data";

export function DashboardMockup() {
  const { t } = useTranslation();
  return (
    <div
      aria-hidden="true"
      className="w-full max-w-[400px] mx-auto"
    >
      {/* App window chrome */}
      <div className="rounded-2xl overflow-hidden shadow-2xl border border-white/10 bg-white/[0.06] backdrop-blur-[12px]">

        {/* Title bar */}
        <div className="flex items-center gap-2 px-4 py-3 border-b border-white/8 bg-white/[0.04]">
          <div className="flex gap-1.5">
            <div className="w-2.5 h-2.5 rounded-full bg-rose-400/60" />
            <div className="w-2.5 h-2.5 rounded-full bg-amber-400/60" />
            <div className="w-2.5 h-2.5 rounded-full bg-emerald-400/60" />
          </div>
          <span className="text-white/40 text-[11px] font-medium ml-2 tracking-wide">{t("auth.mock.window_title")}</span>
        </div>

        <div className="p-4 space-y-4">

          {/* Stat cards row */}
          <div className="grid grid-cols-3 gap-2">
            {STAT_CARDS.map(({ value, label, color, border }) => {
              const labelKey = label === "Employees" ? "employees" : label === "On Time" ? "on_time" : "on_leave";
              return (
                <div
                  className={`rounded-xl p-3 bg-gradient-to-br ${color} border ${border} animate-[auth-fade-in_0.5s_ease-out_both]`}
                  key={label}
                >
                  <Typography className="text-white font-bold text-lg leading-none" variant="p">{value}</Typography>
                  <Typography className="text-white/50 text-[10px] font-medium mt-1 uppercase tracking-wide" variant="p">{t(`auth.mock.${labelKey}`)}</Typography>
                </div>
              );
            })}
          </div>

          {/* Attendance chart */}
          <div className="rounded-xl p-3 bg-white/[0.04]">
            <Typography className="text-white/50 text-[10px] font-semibold uppercase tracking-widest mb-3" variant="p">{t("auth.mock.weekly_attendance")}</Typography>
            <div className="flex items-end gap-1.5 h-14">
              {ATTENDANCE_BARS.map((h, i) => (
                <div className="flex-1 flex flex-col items-center gap-1" key={i}>
                  <div
                    className="w-full rounded-t-sm origin-bottom"
                    style={{
                      height: `${h}%`,
                      background: i === 4
                        ? "linear-gradient(to top, rgba(52,211,153,0.9), rgba(16,185,129,0.5))"
                        : "rgba(255,255,255,0.15)",
                      animation: "auth-bar-grow 0.5s cubic-bezier(0.34,1.4,0.64,1) both",
                      animationDelay: `${i * 60}ms`,
                    }}
                  />
                  <span className="text-white/25 text-[8px] font-medium">{DAY_LABELS[i]}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Employee list */}
          <div className="space-y-1.5">
            <Typography className="text-white/50 text-[10px] font-semibold uppercase tracking-widest mb-2" variant="p">{t("auth.mock.todays_status")}</Typography>
            {MOCK_EMPLOYEES.map(({ name, role, status, initials, hue }, idx) => {
              const s = STATUS_STYLE[status];
              const roleKey = role.toLowerCase();
              return (
                <div
                  className="flex items-center gap-3 rounded-xl px-3 py-2 transition-colors"
                  key={name}
                  style={{
                    background: "rgba(255,255,255,0.04)",
                    animation: "auth-fade-in 0.4s ease-out both",
                    animationDelay: `${200 + idx * 80}ms`,
                  }}
                >
                  <div
                    className="w-7 h-7 rounded-lg flex items-center justify-center text-white text-[10px] font-bold flex-shrink-0"
                    style={{ background: `hsl(${hue} 60% 45% / 0.6)` }}
                  >
                    {initials}
                  </div>
                  <div className="flex-1 min-w-0">
                    <Typography className="text-white/80 text-xs font-semibold truncate leading-none" variant="p">{name}</Typography>
                    <Typography className="text-white/35 text-[10px] truncate mt-0.5" variant="p">{t(`auth.mock.roles.${roleKey}`)}</Typography>
                  </div>
                  <div className={`flex items-center gap-1.5 rounded-full px-2 py-0.5 text-[9px] font-bold uppercase tracking-wide ${s.badge}`}>
                    <div className={`w-1.5 h-1.5 rounded-full ${s.dot} ${status === "present" ? "animate-pulse" : ""}`} />
                    {t(`auth.mock.status.${status}`)}
                  </div>
                </div>
              );
            })}
          </div>

        </div>
      </div>
    </div>
  );
}
