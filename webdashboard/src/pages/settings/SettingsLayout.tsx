import { Link, Outlet, useLocation } from "react-router-dom";
import { ROUTES } from "@/lib/routes";
import { useTranslation } from "react-i18next";
import {
  Settings2,
  Users,
  Clock,
  Banknote,
  Cpu,
  ChevronRight,
  ShieldCheck,
  CalendarDays,
  Palmtree
} from "lucide-react";

export default function SettingsLayout() {
  const { t } = useTranslation();
  const location = useLocation();

  const navItems = [
    {
      path: ROUTES.SETTINGS.GENERAL,
      label: t("settings.categories.general"),
      icon: <Settings2 className="w-4 h-4" />,
      description: t("settings.categories.general_desc")
    },
    {
      path: ROUTES.SETTINGS.PEOPLE,
      label: t("settings.categories.people"),
      icon: <Users className="w-4 h-4" />,
      description: t("settings.categories.people_desc")
    },
    {
      path: ROUTES.SETTINGS.TIME,
      label: t("settings.categories.time"),
      icon: <Clock className="w-4 h-4" />,
      description: t("settings.categories.time_desc")
    },
    {
      path: ROUTES.SETTINGS.PAYROLL,
      label: t("settings.categories.payroll"),
      icon: <Banknote className="w-4 h-4" />,
      description: t("settings.categories.payroll_desc")
    },
    {
      path: ROUTES.SETTINGS.LEAVE,
      label: "Leave Management",
      icon: <CalendarDays className="w-4 h-4" />,
      description: "Manage leave types and policies"
    },
    {
      path: ROUTES.SETTINGS.HOLIDAYS,
      label: "Holidays",
      icon: <Palmtree className="w-4 h-4" />,
      description: "Manage public holidays"
    },
    {
      path: ROUTES.SETTINGS.CONNECTIVITY,
      label: t("settings.categories.connectivity"),
      icon: <Cpu className="w-4 h-4" />,
      description: t("settings.categories.connectivity_desc")
    },
    {
      path: ROUTES.SETTINGS.AUDIT_LOGS,
      label: "Audit Logs",
      icon: <ShieldCheck className="w-4 h-4" />,
      description: "View application security audit logs"
    }
  ];

  const isActive = (path: string) => {
    if (path === ROUTES.SETTINGS.GENERAL && location.pathname === ROUTES.SETTINGS.ROOT) return true;
    return location.pathname.startsWith(path);
  };

  return (
    <div className="flex flex-col lg:flex-row gap-8 min-h-[calc(100vh-10rem)]">
      {/* Sub-navigation Sidebar */}
      <aside className="w-full lg:w-72 flex-shrink-0">
        <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-4 shadow-sm sticky top-24">
          <div className="px-4 py-3 mb-2">
            <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.2em]">
              {t("settings.navigation")}
            </h2>
          </div>
          <nav className="space-y-1">
            {navItems.map((item) => (
              <Link
                key={item.path}
                to={item.path}
                className={`flex items-start gap-3 px-4 py-3 rounded-2xl transition-all duration-300 group ${isActive(item.path)
                  ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 shadow-sm shadow-primary-100/50"
                  : "text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700"
                  }`}
              >
                <div className={`p-2 rounded-xl transition-colors ${isActive(item.path) ? "bg-white dark:bg-slate-900 shadow-sm" : "bg-slate-50 dark:bg-slate-800 group-hover:bg-white dark:hover:bg-slate-900"
                  }`}>
                  {item.icon}
                </div>
                <div className="flex flex-col min-w-0">
                  <span className="font-bold text-sm leading-tight">{item.label}</span>
                  <span className="text-[10px] text-slate-500 dark:text-slate-400 font-medium leading-tight hidden xl:block">
                    {item.description}
                  </span>
                </div>
                {isActive(item.path) && (
                  <ChevronRight className="w-4 h-4 ml-auto text-primary-400" />
                )}
              </Link>
            ))}
          </nav>

          <div className="mt-8 pt-6 border-t border-slate-50 dark:border-slate-800 px-4 pb-2">
            <div className="flex items-center gap-2 text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
              <ShieldCheck className="w-3 h-3" />
              <span>{t("settings.admin_only")}</span>
            </div>
          </div>
        </div>
      </aside>

      {/* Settings Content Area */}
      <main className="flex-1 min-w-0">
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
