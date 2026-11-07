import { Link } from "@tanstack/react-router";
import { Clock, CalendarPlus, Zap, Banknote, ChevronRight } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";
import { ROUTES } from "@/lib/routes";

interface ShortcutItem {
  to: string;
  titleKey: string;
  titleFallback: string;
  descKey: string;
  descFallback: string;
  icon: React.ElementType;
  gradient: string;
}

const SHORTCUTS: ShortcutItem[] = [
  {
    to: ROUTES.ESS_CLOCK,
    titleKey: "desktop.shortcuts.clock.title",
    titleFallback: "Bấm giờ cá nhân",
    descKey: "desktop.shortcuts.clock.desc",
    descFallback: "Ghi nhận vào / ra ca",
    icon: Clock,
    gradient: "from-cyan-500 to-blue-600 shadow-cyan-500/20",
  },
  {
    to: ROUTES.LEAVE_REQUESTS,
    titleKey: "desktop.shortcuts.leave.title",
    titleFallback: "Tạo đơn xin nghỉ",
    descKey: "desktop.shortcuts.leave.desc",
    descFallback: "Nghỉ phép & chế độ",
    icon: CalendarPlus,
    gradient: "from-sky-500 to-indigo-600 shadow-sky-500/20",
  },
  {
    to: ROUTES.ESS_OT,
    titleKey: "desktop.shortcuts.ot.title",
    titleFallback: "Đăng ký tăng ca",
    descKey: "desktop.shortcuts.ot.desc",
    descFallback: "Làm thêm giờ (OT)",
    icon: Zap,
    gradient: "from-amber-500 to-orange-600 shadow-amber-500/20",
  },
  {
    to: ROUTES.PAYROLL,
    titleKey: "desktop.shortcuts.payroll.title",
    titleFallback: "Xem phiếu lương",
    descKey: "desktop.shortcuts.payroll.desc",
    descFallback: "Bảng lương & thưởng",
    icon: Banknote,
    gradient: "from-emerald-500 to-teal-600 shadow-emerald-500/20",
  },
];

export function QuickShortcutsWidget() {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col justify-between h-full space-y-2">
      <div className="space-y-2 flex-1">
        {SHORTCUTS.map((item) => {
          const Icon = item.icon;
          const title = t(item.titleKey, { defaultValue: item.titleFallback });
          const desc = t(item.descKey, { defaultValue: item.descFallback });
          return (
            <Link
              key={item.to}
              to={item.to}
              aria-label={`${title}: ${desc}`}
              className="flex items-center justify-between p-2.5 rounded-xl bg-slate-50/70 dark:bg-slate-900/50 border border-slate-100 dark:border-slate-800/60 hover:bg-white dark:hover:bg-slate-800 hover:shadow-2xs hover:border-slate-200 dark:hover:border-slate-700 transition-all text-xs group"
            >
              <div className="flex items-center gap-2.5 min-w-0">
                <div
                  className={`w-7 h-7 rounded-lg bg-gradient-to-br flex items-center justify-center text-white shrink-0 group-hover:scale-105 transition-transform ${item.gradient}`}
                >
                  <Icon className="w-3.5 h-3.5" />
                </div>
                <div className="min-w-0">
                  <span className="font-semibold text-slate-700 dark:text-slate-200 text-xs block leading-tight truncate">
                    {title}
                  </span>
                  <span className="text-[10px] text-slate-400 dark:text-slate-500 block leading-tight truncate mt-0.5">
                    {desc}
                  </span>
                </div>
              </div>

              <ChevronRight className="w-4 h-4 text-slate-300 dark:text-slate-600 group-hover:text-primary-500 group-hover:translate-x-0.5 transition-all shrink-0 ml-2" />
            </Link>
          );
        })}
      </div>
    </div>
  );
}
