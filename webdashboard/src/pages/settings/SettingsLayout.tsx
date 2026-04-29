import { Link, Outlet, useLocation } from "@tanstack/react-router";
import { ROUTES } from "@/lib/routes";
import { useTranslation } from "react-i18next";
import { useMyPermissions } from "@/hooks/use-permissions";
import {
  Building2,
  Users,
  ShieldAlert,
  KeyRound,
  CreditCard,
  AlertTriangle,
  ChevronRight,
  ShieldCheck,
  Settings2,
  Clock,
  Banknote,
  CalendarDays,
  Palmtree,
  Cpu,
  UserCog,
  Settings
} from "lucide-react";

function NavGroup({ title, items, isActive }: { title: string, items: { path: string, label: string, icon: React.ReactNode, description: string }[], isActive: (path: string) => boolean }) {
  return (
    <div className="mb-6">
      <h3 className="px-4 text-[10px] font-black text-slate-400 dark:text-slate-500 uppercase tracking-[0.2em] mb-2">{title}</h3>
      <div className="space-y-1">
        {items.map((item) => (
          <Link
            key={item.path}
            to={item.path}
            className={`flex items-start gap-3 px-4 py-3 rounded-2xl transition-all duration-300 group ${
              isActive(item.path)
                ? "bg-indigo-50 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-400 shadow-sm shadow-indigo-100/50 border border-indigo-100 dark:border-indigo-800/50"
                : "text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 hover:text-slate-700 dark:hover:text-slate-200 border border-transparent"
            }`}
          >
            <div
              className={`p-2 rounded-xl transition-colors ${
                isActive(item.path)
                  ? "bg-white dark:bg-slate-900 shadow-sm"
                  : "bg-slate-50 dark:bg-slate-800 group-hover:bg-white dark:hover:bg-slate-900"
              }`}
            >
              {item.icon}
            </div>
            <div className="flex flex-col min-w-0">
              <span className="font-bold text-sm leading-tight">{item.label}</span>
              <span className="text-[10px] text-slate-500 dark:text-slate-400 font-medium leading-tight hidden xl:block mt-0.5">
                {item.description}
              </span>
            </div>
            {isActive(item.path) && (
              <ChevronRight className="w-4 h-4 ml-auto text-indigo-400" />
            )}
          </Link>
        ))}
      </div>
    </div>
  );
}

export default function SettingsLayout() {
  const { t } = useTranslation();
  const location = useLocation();
  const { hasAnyPermission } = useMyPermissions();

  const workspaceGroup = [
    {
      path: ROUTES.SETTINGS.OVERVIEW,
      label: "Overview",
      icon: <Building2 className="w-4 h-4" />,
      description: "Workspace summary and statistics",
      permissions: ["workspace:settings:read"],
    },
    {
      path: ROUTES.SETTINGS.GENERAL,
      label: t("settings.categories.general"),
      icon: <Settings2 className="w-4 h-4" />,
      description: t("settings.categories.general_desc"),
      permissions: ["workspace:settings:read"],
    },
    {
      path: ROUTES.SETTINGS.BILLING,
      label: "Billing",
      icon: <CreditCard className="w-4 h-4" />,
      description: "Subscription and invoices",
      permissions: ["workspace:settings:read"],
    },
  ];

  const accessGroup = [

    {
      path: ROUTES.SETTINGS.ROLES,
      label: "Roles",
      icon: <ShieldAlert className="w-4 h-4" />,
      description: "Define custom roles",
      permissions: ["members:roles:manage"],
    },
    {
      path: ROUTES.SETTINGS.PERMISSIONS,
      label: "Permissions",
      icon: <KeyRound className="w-4 h-4" />,
      description: "Permission matrix",
      permissions: ["members:roles:manage"],
    },
  ];

  const operationsGroup = [
    {
      path: ROUTES.SETTINGS.ORGANIZATION,
      label: t("settings.categories.organization", { defaultValue: "Organization" }),
      icon: <Building2 className="w-4 h-4" />,
      description: t("settings.categories.organization_desc", { defaultValue: "Manage departments and employment phases" }),
      permissions: ["employees:read", "departments:read"],
    },
    {
      path: ROUTES.SETTINGS.TIME,
      label: t("settings.categories.time"),
      icon: <Clock className="w-4 h-4" />,
      description: t("settings.categories.time_desc"),
      permissions: ["attendance:logs:read"],
    },
    {
      path: ROUTES.SETTINGS.PAYROLL,
      label: t("settings.categories.payroll"),
      icon: <Banknote className="w-4 h-4" />,
      description: t("settings.categories.payroll_desc"),
      permissions: ["payroll:read", "payroll:settings:manage"],
    },
    {
      path: ROUTES.SETTINGS.LEAVE,
      label: "Leave",
      icon: <CalendarDays className="w-4 h-4" />,
      description: "Leave policies",
      permissions: ["leave:policies:manage"],
    },
    {
      path: ROUTES.SETTINGS.HOLIDAYS,
      label: "Holidays",
      icon: <Palmtree className="w-4 h-4" />,
      description: "Public holidays",
      permissions: ["holidays:manage"],
    },
  ];

  const systemGroup = [
    {
      path: ROUTES.SETTINGS.CONNECTIVITY,
      label: t("settings.categories.connectivity"),
      icon: <Cpu className="w-4 h-4" />,
      description: t("settings.categories.connectivity_desc"),
      permissions: ["infra:monitor"],
    },
    {
      path: ROUTES.SETTINGS.AUDIT_LOGS,
      label: "Audit Logs",
      icon: <ShieldCheck className="w-4 h-4" />,
      description: "Security events",
      permissions: ["workspace:settings:read", "infra:monitor"],
    },
  ];

  const dangerItem = {
    path: ROUTES.SETTINGS.DANGER,
    label: "Danger Zone",
    icon: <AlertTriangle className="w-4 h-4" />,
    description: "Destructive operations",
    permissions: ["workspace:delete"],
  };

  const isActive = (path: string) => {
    if (path === ROUTES.SETTINGS.OVERVIEW && location.pathname === ROUTES.SETTINGS.ROOT) return true;
    return location.pathname.startsWith(path);
  };



  type SettingItem = { path: string; label: string; icon: React.ReactNode; description: string; permissions?: string[] };
  const filterItems = (items: SettingItem[]) => 
    items.filter(item => !item.permissions || item.permissions.length === 0 || hasAnyPermission(...item.permissions));

  const filteredWorkspaceGroup = filterItems(workspaceGroup);
  const filteredAccessGroup = filterItems(accessGroup);
  const filteredOperationsGroup = filterItems(operationsGroup);
  const filteredSystemGroup = filterItems(systemGroup);
  const showDangerItem = !dangerItem.permissions || dangerItem.permissions.length === 0 || hasAnyPermission(...dangerItem.permissions);

  return (
    <div className="flex flex-col lg:flex-row gap-8 min-h-[calc(100vh-10rem)]">
      {/* Sub-navigation Sidebar */}
      <aside className="w-full lg:w-72 flex-shrink-0">
        <div className="bg-white dark:bg-slate-900 rounded-3xl border border-slate-100 dark:border-slate-800 p-4 shadow-sm sticky top-24 max-h-[calc(100vh-8rem)] overflow-y-auto custom-scrollbar">
          <div className="px-4 py-3 mb-4 flex items-center gap-3">
            <div className="p-2 bg-indigo-50 dark:bg-indigo-900/30 rounded-xl">
              <Settings className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
            </div>
            <div>
              <h2 className="text-sm font-bold text-slate-800 dark:text-slate-200">
                Settings
              </h2>
              <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5">
                Control Panel
              </p>
            </div>
          </div>
          
          <nav>
            {filteredWorkspaceGroup.length > 0 && <NavGroup title="Workspace" items={filteredWorkspaceGroup} isActive={isActive} />}
            {filteredAccessGroup.length > 0 && <NavGroup title="Access" items={filteredAccessGroup} isActive={isActive} />}
            {filteredOperationsGroup.length > 0 && <NavGroup title="Operations" items={filteredOperationsGroup} isActive={isActive} />}
            {filteredSystemGroup.length > 0 && <NavGroup title="Security & System" items={filteredSystemGroup} isActive={isActive} />}
            
            {showDangerItem && (
              <div className="pt-4 mt-4 border-t border-slate-100 dark:border-slate-800">
                <Link
                  to={dangerItem.path}
                  className={`flex items-start gap-3 px-4 py-3 rounded-2xl transition-all duration-300 group ${
                    isActive(dangerItem.path)
                      ? "bg-rose-50 dark:bg-rose-950/30 text-rose-600 dark:text-rose-400 shadow-sm border border-rose-100 dark:border-rose-900/50"
                      : "text-slate-500 dark:text-slate-400 hover:bg-rose-50/50 dark:hover:bg-rose-900/10 hover:text-rose-600 dark:hover:text-rose-400 border border-transparent"
                  }`}
                >
                  <div
                    className={`p-2 rounded-xl transition-colors ${
                      isActive(dangerItem.path)
                        ? "bg-white dark:bg-slate-900 shadow-sm"
                        : "bg-slate-50 dark:bg-slate-800 group-hover:bg-rose-100 dark:group-hover:bg-rose-900/30 text-rose-500"
                    }`}
                  >
                    {dangerItem.icon}
                  </div>
                  <div className="flex flex-col min-w-0">
                    <span className="font-bold text-sm leading-tight">{dangerItem.label}</span>
                    <span className="text-[10px] text-slate-500 dark:text-slate-400 font-medium leading-tight hidden xl:block mt-0.5 group-hover:text-rose-500/70">
                      {dangerItem.description}
                    </span>
                  </div>
                  {isActive(dangerItem.path) && (
                    <ChevronRight className="w-4 h-4 ml-auto text-rose-400" />
                  )}
                </Link>
              </div>
            )}
          </nav>

          <div className="mt-8 pt-6 border-t border-slate-50 dark:border-slate-800 px-4 pb-2">
            <div className="flex items-center gap-2 text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">
              <ShieldCheck className="w-3 h-3" />
              <span>Workspace Admin Only</span>
            </div>
          </div>
        </div>
      </aside>

      {/* Content Area */}
      <main className="flex-1 min-w-0">
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500 bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm p-8 lg:p-10 min-h-full">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
