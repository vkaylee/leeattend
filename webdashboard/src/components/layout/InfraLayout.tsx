import { Outlet, Link } from "@tanstack/react-router";
import { Activity, LogOut, ArrowLeft } from "lucide-react";
import { useTranslation } from "react-i18next";

import { ROUTES } from "@/lib/routes";


/**
 * Minimal layout for infrastructure pages that don't require workspace context.
 * Used for global monitoring, system admin, etc.
 */
export default function InfraLayout() {
  const { t } = useTranslation();

  const handleLogout = () => {
    localStorage.clear();
    window.location.href = ROUTES.LOGIN;
  };

  return (
    <div className="min-h-screen bg-slate-50 dark:bg-slate-800">
      {/* Top bar */}
      <header className="h-16 bg-white dark:bg-slate-900 border-b border-slate-100 dark:border-slate-800 px-6 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Link
            to={ROUTES.HOME}
            className="flex items-center gap-2 text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700 transition-colors"
          >
            <ArrowLeft size={18} />
            <span className="text-sm font-medium">{t("common.back_to_workspace")}</span>
          </Link>
        </div>

        <div className="flex items-center gap-3">
          <div className="h-10 w-10 rounded-2xl bg-gradient-to-tr from-primary-600 to-indigo-500 shadow-lg shadow-primary-200 flex items-center justify-center text-white font-black text-xl tracking-tighter italic">
            L
          </div>
          <span className="text-xl font-black tracking-tighter text-slate-800 dark:text-slate-100 uppercase italic">
            Lee<span className="text-primary-600 dark:text-primary-400">Attend</span>
          </span>
        </div>

        <button
          onClick={handleLogout}
          className="flex items-center gap-2 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 transition-colors"
        >
          <LogOut size={18} />
          <span className="text-sm font-medium">{t("auth.logout")}</span>
        </button>
      </header>

      {/* Content */}
      <main className="p-8">
        <Outlet />
      </main>
    </div>
  );
}
