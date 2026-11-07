import { Link } from "@tanstack/react-router";
import { Clock, CalendarPlus, Zap, Banknote, Search, Sparkles } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";
 import Button from "@/components/ui/button";
import { openCommandPalette } from "@/components/layout/command-palette";
 import { ROUTES } from "@/lib/routes";

interface ShortcutItem {
  to: string;
  label: string;
  icon: React.ElementType;
  gradient: string;
}

const SHORTCUTS: ShortcutItem[] = [
  {
    to: ROUTES.ESS_CLOCK,
    label: "Bấm giờ cá nhân",
    icon: Clock,
    gradient: "from-cyan-500 to-blue-600 shadow-cyan-500/20",
  },
  {
    to: ROUTES.LEAVE_REQUESTS,
    label: "Tạo đơn xin nghỉ",
    icon: CalendarPlus,
    gradient: "from-sky-500 to-indigo-600 shadow-sky-500/20",
  },
  {
    to: ROUTES.ESS_OT,
    label: "Đăng ký làm thêm OT",
    icon: Zap,
    gradient: "from-amber-500 to-orange-600 shadow-amber-500/20",
  },
  {
    to: ROUTES.PAYROLL,
    label: "Xem phiếu lương",
    icon: Banknote,
    gradient: "from-emerald-500 to-teal-600 shadow-emerald-500/20",
  },
];

export function QuickShortcutsWidget() {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col justify-between h-full space-y-4">
      {/* DSM 7 Launcher Trigger Banner */}
      <Button
         type="button"
        onClick={openCommandPalette}
         className="w-full h-auto flex items-center justify-between p-3.5 rounded-2xl bg-gradient-to-r from-slate-900 to-slate-800 text-white shadow-lg hover:shadow-xl hover:from-slate-800 hover:to-slate-700 transition-all duration-300 border border-white/10 group cursor-pointer text-left"
        data-testid="desktop-dsm-launcher-shortcut"
      >
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-xl bg-white/10 flex items-center justify-center text-primary-400 group-hover:scale-110 transition-transform">
            <Sparkles className="w-4 h-4" />
          </div>
          <div>
            <div className="text-xs font-bold leading-tight flex items-center gap-1.5">
              <span>{t("desktop.openDsmTitle", "Main Menu & Tìm kiếm")}</span>
            </div>
            <div className="text-[11px] text-slate-400">
              {t("desktop.openDsmSubtitle", "Duyệt toàn bộ 45 ứng dụng & cài đặt")}
            </div>
          </div>
        </div>

        <kbd className="px-2 py-0.5 text-[10px] font-semibold text-slate-300 bg-white/10 rounded-lg border border-white/10">
          ⌘K
        </kbd>
      </Button>

      {/* Grid Shortcuts */}
      <div className="grid grid-cols-2 gap-2.5">
        {SHORTCUTS.map((item) => {
          const Icon = item.icon;
          return (
            <Link
              key={item.to}
              to={item.to}
              className="flex items-center gap-2.5 p-3 rounded-2xl bg-slate-50/70 dark:bg-slate-900/50 border border-slate-100 dark:border-slate-800/60 hover:bg-white dark:hover:bg-slate-800 hover:shadow-sm hover:border-slate-200 dark:hover:border-slate-700 transition-all text-xs"
            >
              <div
                className={`w-7 h-7 rounded-xl bg-gradient-to-br flex items-center justify-center text-white shrink-0 ${item.gradient}`}
              >
                <Icon className="w-3.5 h-3.5" />
              </div>
              <span className="font-semibold text-slate-700 dark:text-slate-200 text-xs leading-tight line-clamp-2">
                {item.label}
              </span>
            </Link>
          );
        })}
      </div>
    </div>
  );
}
