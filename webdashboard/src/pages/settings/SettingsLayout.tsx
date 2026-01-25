import { Link, Outlet, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { 
  Settings2, 
  Users, 
  Clock, 
  Banknote, 
  Cpu, 
  ChevronRight,
  ShieldCheck,
  CalendarDays
} from "lucide-react";

export default function SettingsLayout() {
  const { t } = useTranslation();
  const location = useLocation();

  const navItems = [
    { 
      path: "/workspace/settings/general", 
      label: t("settings.categories.general"), 
      icon: <Settings2 className="w-4 h-4" />,
      description: t("settings.categories.general_desc")
    },
    { 
      path: "/workspace/settings/people", 
      label: t("settings.categories.people"), 
      icon: <Users className="w-4 h-4" />,
      description: t("settings.categories.people_desc")
    },
    { 
      path: "/workspace/settings/time", 
      label: t("settings.categories.time"), 
      icon: <Clock className="w-4 h-4" />,
      description: t("settings.categories.time_desc")
    },
    { 
      path: "/workspace/settings/payroll", 
      label: t("settings.categories.payroll"), 
      icon: <Banknote className="w-4 h-4" />,
      description: t("settings.categories.payroll_desc")
    },
        { 
      path: "/workspace/settings/leave", 
      label: "Leave Management", 
      icon: <CalendarDays className="w-4 h-4" />,
      description: "Manage leave types and policies"
    },
    { 
      path: "/workspace/settings/connectivity", 
      label: t("settings.categories.connectivity"), 
      icon: <Cpu className="w-4 h-4" />,
      description: t("settings.categories.connectivity_desc")
    }
  ];

  const isActive = (path: string) => {
    if (path === "/workspace/settings/general" && location.pathname === "/workspace/settings") return true;
    return location.pathname.startsWith(path);
  };

  return (
    <div className="flex flex-col lg:flex-row gap-8 min-h-[calc(100vh-10rem)]">
      {/* Sub-navigation Sidebar */}
      <aside className="w-full lg:w-72 flex-shrink-0">
        <div className="bg-white rounded-3xl border border-slate-100 p-4 shadow-sm sticky top-24">
          <div className="px-4 py-3 mb-2">
            <h2 className="text-xs font-bold text-slate-500 uppercase tracking-[0.2em]">
              {t("settings.navigation")}
            </h2>
          </div>
          <nav className="space-y-1">
            {navItems.map((item) => (
              <Link
                key={item.path}
                to={item.path}
                className={`flex items-center gap-3 px-4 py-3 rounded-2xl transition-all duration-300 group ${
                  isActive(item.path)
                    ? "bg-primary-50 text-primary-600 shadow-sm shadow-primary-100/50"
                    : "text-slate-500 hover:bg-slate-50 hover:text-slate-700"
                }`}
              >
                <div className={`p-2 rounded-xl transition-colors ${
                  isActive(item.path) ? "bg-white shadow-sm" : "bg-slate-50 group-hover:bg-white"
                }`}>
                  {item.icon}
                </div>
                <div className="flex flex-col min-w-0">
                  <span className="font-bold text-sm truncate">{item.label}</span>
                  <span className="text-[10px] text-slate-500 font-medium truncate hidden xl:block">
                    {item.description}
                  </span>
                </div>
                {isActive(item.path) && (
                  <ChevronRight className="w-4 h-4 ml-auto text-primary-400" />
                )}
              </Link>
            ))}
          </nav>

          <div className="mt-8 pt-6 border-t border-slate-50 px-4 pb-2">
            <div className="flex items-center gap-2 text-[10px] font-bold text-slate-300 uppercase tracking-widest">
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
