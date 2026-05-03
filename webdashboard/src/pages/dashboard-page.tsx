import { Link } from "@tanstack/react-router";
import {
  Monitor,
  ShieldCheck,
  ShieldAlert,
  Activity,
  Cpu,
  Users,
  UserCheck,
  CalendarClock,
  Banknote,
  Users2,
  ArrowRight,
  Clock,
  UserX,
  AlertTriangle,
  LogOut,
  Timer,
  Settings2,
} from "lucide-react";
import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";

import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from "recharts";

import CollapsibleSection from "@/components/features/dashboard/collapsible-section";
import CustomizePanel from "@/components/features/dashboard/customize-panel";
import DashboardViewTabs from "@/components/features/dashboard/dashboard-view-tabs";
import WidgetRenderer from "@/components/features/dashboard/widget-renderer";
import PromoCard from "@/components/ui/promo-card";
import StatCard from "@/components/ui/stat-card";
import { useAgentsStatus } from "@/hooks/use-agents";
import { useCurrency } from "@/hooks/use-currency";
import { useDashboardStats } from "@/hooks/use-dashboard";
import { useDashboardLayout, type WidgetId } from "@/hooks/use-dashboard-layout";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useDevices } from "@/hooks/use-devices";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

const DONUT_COLORS = {
  on_time: "#10b981",   // emerald-500
  late: "#f59e0b",      // amber-500
  absent: "#ef4444",    // rose-500
  on_leave: "#6366f1",  // indigo-500
};

function DashboardHeader({ t, showSystemTab, setShowSystemTab, activeView, setActiveView, setCustomizeOpen }: any) {
  return (
    <header className="flex flex-col gap-6 px-2">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div className="space-y-1">
          <h1 className="text-5xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter uppercase leading-none" data-testid="dashboard-title">{t("dashboard.title")}</h1>
          <p className="text-slate-500 dark:text-slate-400 text-lg font-bold uppercase tracking-[0.25em] opacity-60">{t("dashboard.welcome")}</p>
        </div>
        <div className="flex bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50 shadow-inner">
          <button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", !showSystemTab ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200")} onClick={() => { setShowSystemTab(false); }}><Users size={14} /> {t("dashboard.tabs.hr")}</button>
          <button className={cn("flex items-center gap-2 px-8 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all", showSystemTab ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800" : "text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200")} onClick={() => { setShowSystemTab(true); }}><Cpu size={14} /> {t("dashboard.tabs.system")}</button>
        </div>
      </div>
      {!showSystemTab && (
        <div className="flex items-center justify-between gap-4 flex-wrap">
          <DashboardViewTabs activeView={activeView} onViewChange={setActiveView} />
          <button
            className="flex items-center gap-2 px-5 py-2.5 rounded-2xl border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 text-xs font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 hover:border-primary-200 dark:hover:border-primary-700 transition-all shadow-sm"
            onClick={() => { setCustomizeOpen(true); }}
          >
            <Settings2 size={14} />
            <span className="hidden sm:inline">{t("dashboard.customize.btn")}</span>
          </button>
        </div>
      )}
    </header>
  );
}

function DashboardHrTab({ hrStats, isLoading, visibleWidgets, collapsedSections, toggleSection, donutData, hasDonutData, formatCurrency, chartElements, t }: any) {
  return (
    <div className="space-y-8">
      {!!visibleWidgets.kpi_cards && (
        <CollapsibleSection
          id="summary"
          isCollapsed={collapsedSections.summary}
          onToggle={toggleSection}
          title={t("dashboard.sections.summary")}
        >
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4 sm:gap-6">
            {[
              { label: "total_employees", value: hrStats?.total_employees, icon: Users2, color: "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800" },
              { label: "present_today", value: hrStats?.present_today, icon: UserCheck, color: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" },
              { label: "late_today", value: hrStats?.late_today, icon: Clock, color: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800" },
              { label: "absent_today", value: hrStats?.absent_today, icon: UserX, color: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-300 border-rose-100 dark:border-rose-800" },
              { label: "on_leave_today", value: hrStats?.on_leave_today, icon: CalendarClock, color: "bg-indigo-50 dark:bg-indigo-950 text-indigo-600 dark:text-indigo-300 border-indigo-100 dark:border-indigo-800" },
              { label: "pending_requests", value: hrStats?.pending_leave_requests, icon: AlertTriangle, color: "bg-orange-50 dark:bg-orange-950 text-orange-600 dark:text-orange-300 border-orange-100 dark:border-orange-800" },
            ].map((stat, index) => (
              <StatCard
                colorClasses={stat.color}
                icon={stat.icon}
                isLoading={isLoading}
                key={index}
                label={t(`dashboard.hr.${stat.label}`)}
                size="sm"
                value={stat.value ?? 0}
              />
            ))}
          </div>
        </CollapsibleSection>
      )}

      {(visibleWidgets.donut_chart || visibleWidgets.quick_stats) && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {!!visibleWidgets.donut_chart && (
            <div className="bg-white dark:bg-slate-900 rounded-[3rem] p-8 sm:p-10 border border-slate-100 dark:border-slate-800 shadow-sm">
              <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mb-8">{t("dashboard.hr.attendance_breakdown")}</h2>
              {hasDonutData ? (
                <div className="flex flex-col items-center">
                  <ResponsiveContainer height={260} width="100%">
                    <PieChart>
                      <Pie cx="50%" cy="50%" data={donutData} dataKey="value" innerRadius={70} outerRadius={110} paddingAngle={3} stroke="none">
                        {donutData.map((entry: any, index: number) => (
                          <Cell fill={entry.color} key={`cell-${index}`} />
                        ))}
                      </Pie>
                      <Tooltip contentStyle={{ borderRadius: "1rem", border: "none", boxShadow: "0 8px 30px rgba(0,0,0,.1)", fontSize: "12px", fontWeight: 600 }} />
                    </PieChart>
                  </ResponsiveContainer>
                  <div className="flex flex-wrap justify-center gap-4 mt-4">
                    {donutData.map((entry: any, index: number) => (
                      <div className="flex items-center gap-2" key={index}>
                        <div className="w-3 h-3 rounded-full" style={{ backgroundColor: entry.color }} />
                        <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{entry.name}: {entry.value}</span>
                      </div>
                    ))}
                  </div>
                </div>
              ) : (
                <div className="flex flex-col items-center justify-center h-[260px]">
                  <Users2 className="text-slate-200 dark:text-slate-700 mb-4" size={48} />
                  <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">{t("common.no_data")}</p>
                </div>
              )}
            </div>
          )}

          {!!visibleWidgets.quick_stats && (
            <div className="space-y-4 sm:space-y-6">
              <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] px-4">{t("dashboard.hr.quick_stats")}</h2>
              {[
                { label: "avg_work_hours", value: isLoading ? "..." : `${(hrStats?.avg_work_hours_today ?? 0).toFixed(1)}h`, icon: Timer, color: "bg-sky-50 dark:bg-sky-950 text-sky-600 dark:text-sky-400 border-sky-100 dark:border-sky-800", mono: true },
                { label: "early_departures", value: isLoading ? "..." : hrStats?.early_departure_today ?? 0, icon: LogOut, color: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400 border-emerald-100 dark:border-emerald-800", mono: false },
                { label: "estimated_payroll", value: isLoading ? "..." : formatCurrency(hrStats?.estimated_monthly_payroll ?? 0), icon: Banknote, color: "bg-rose-50 dark:bg-rose-950 text-rose-600 dark:text-rose-400 border-rose-100 dark:border-rose-800", mono: true },
              ].map((item, index) => (
                <StatCard
                  colorClasses={item.color}
                  icon={item.icon}
                  key={index}
                  label={t(`dashboard.hr.${item.label}`)}
                  mono={item.mono}
                  size="lg"
                  value={item.value}
                />
              ))}
            </div>
          )}
        </div>
      )}

      {chartElements.length > 0 && (
        <CollapsibleSection
          id="charts"
          isCollapsed={collapsedSections.charts}
          onToggle={toggleSection}
          title={t("dashboard.sections.charts")}
        >
          <div className="space-y-8">
            {chartElements}
          </div>
        </CollapsibleSection>
      )}

      {!!visibleWidgets.promo_cards && (
        <CollapsibleSection
          id="actions"
          isCollapsed={collapsedSections.actions}
          onToggle={toggleSection}
          title={t("dashboard.sections.actions")}
        >
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            <PromoCard
              buttonText={t("dashboard.hr.hr_management_btn").toUpperCase()}
              description={t("dashboard.hr.hr_management_desc")}
              eyebrow={t("dashboard.hr.hr_management")}
              linkElement={
                <Link className="bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-200 px-8 sm:px-10 py-3 sm:py-4 rounded-2xl font-bold text-xs uppercase tracking-widest transition-all inline-flex items-center gap-4 hover:gap-6 shadow-2xl hover:shadow-primary-500/20 active:scale-95" to={ROUTES.EMPLOYEES}>
                  {t("dashboard.hr.hr_management_btn").toUpperCase()} <ArrowRight size={20} />
                </Link>
              }
              title={t("dashboard.hr.hr_management_title")}
              variant="dark"
              watermarkIcon={Users2}
            />
            <PromoCard
              buttonText={t("dashboard.hr.payroll_btn").toUpperCase()}
              description={t("dashboard.hr.payroll_desc")}
              eyebrow={t("dashboard.hr.payroll")}
              linkElement={
                <Link className="bg-slate-900 text-white px-8 sm:px-10 py-3 sm:py-4 rounded-2xl font-bold text-xs uppercase tracking-widest transition-all inline-flex items-center gap-4 hover:gap-6 shadow-2xl shadow-black/20 active:scale-95" to={ROUTES.ATTENDANCE_LOGS}>
                  {t("dashboard.hr.payroll_btn").toUpperCase()} <ArrowRight size={20} />
                </Link>
              }
              title={t("dashboard.hr.payroll_title")}
              variant="primary"
              watermarkIcon={Banknote}
            />
          </div>
        </CollapsibleSection>
      )}
    </div>
  );
}

function DashboardSystemTab({ agents, devices, onlineAgents, activeDevices, isLoading, formatTime, t }: any) {
  return (
    <div className="space-y-12 animate-in fade-in duration-500">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 sm:gap-8">
        {[
          { label: "total_agents", value: agents.length, icon: Users, color: "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800" },
          { label: "online_agents", value: onlineAgents, icon: Activity, color: "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" },
          { label: "total_devices", value: devices.length, icon: Cpu, color: "bg-indigo-50 dark:bg-indigo-950 text-indigo-600 border-indigo-100" },
          { label: "active_devices", value: activeDevices, icon: ShieldCheck, color: "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800" }
        ].map((stat, index) => (
          <div className="bg-white dark:bg-slate-900 p-6 sm:p-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden" key={index}>
            <div className={`${stat.color} w-14 sm:w-16 h-14 sm:h-16 rounded-[1.5rem] flex items-center justify-center mb-6 sm:mb-8 group-hover:scale-110 group-hover:rotate-3 transition-all shadow-inner border`}><stat.icon size={28} /></div>
            <p className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.25em] mb-2">{t(`dashboard.system.${stat.label}`)}</p>
            <p className="text-4xl sm:text-5xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter">{isLoading ? "..." : stat.value}</p>
            <stat.icon className="absolute -right-6 -bottom-6 w-28 sm:w-32 h-28 sm:h-32 text-slate-50 opacity-10 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-10">
        <section className="space-y-8">
          <div className="flex justify-between items-center px-4">
            <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 tracking-[0.3em] uppercase">{t("dashboard.system.edge_agents")}</h2>
            <span className="text-[9px] font-bold text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 px-4 py-1.5 rounded-full uppercase tracking-widest border border-primary-100 dark:border-primary-800 shadow-sm">{t("dashboard.system.live_data")}</span>
          </div>
          <div className="space-y-4">
            {agents.map((agent: any) => (
              <div className="bg-white dark:bg-slate-900 p-6 px-8 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 flex items-center justify-between hover:shadow-2xl hover:shadow-primary-100/10 transition-all group relative overflow-hidden" key={agent.agent_id}>
                <div className="flex items-center gap-6">
                  <div className="bg-slate-50 dark:bg-slate-800 p-5 rounded-[1.5rem] text-slate-500 dark:text-slate-400 group-hover:bg-primary-50 dark:group-hover:bg-primary-950 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-all border border-slate-100 dark:border-slate-800 shadow-inner">
                    <Monitor size={28} />
                  </div>
                  <div>
                    <h3 className="font-bold text-slate-700 dark:text-slate-200 text-xl uppercase tracking-tight leading-none">{agent.agent_id}</h3>
                    <p className="text-[10px] font-bold uppercase text-slate-500 dark:text-slate-400 tracking-widest mt-2 opacity-70">{t("dashboard.system.seen")}: {formatTime(agent.last_seen)}</p>
                  </div>
                </div>
                <span className={cn("px-5 py-2 rounded-full text-[10px] font-bold uppercase tracking-widest border shadow-sm", agent.is_online ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800" : "bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60")}>
                  {agent.is_online ? t("dashboard.system.connected").toUpperCase() : t("dashboard.system.disconnected").toUpperCase()}
                </span>
              </div>
            ))}
            {agents.length === 0 && !isLoading && <div className="py-32 text-center border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[3rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center"><Monitor className="text-slate-200 dark:text-slate-700 mb-4" size={40} /><p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest">{t("dashboard.system.no_agents")}</p></div>}
          </div>
        </section>

        <section className="space-y-8">
          <h2 className="text-xs font-bold text-slate-500 dark:text-slate-400 tracking-[0.3em] uppercase px-4">{t("dashboard.system.quick_actions")}</h2>
          <div className="bg-slate-900 rounded-[3rem] p-10 sm:p-14 text-white relative overflow-hidden shadow-2xl shadow-slate-200 dark:shadow-black/40 group">
            <div className="relative z-10">
              <h3 className="text-3xl sm:text-4xl font-bold mb-6 tracking-tighter uppercase leading-[1.1]">{t("dashboard.system.device_management")}</h3>
              <p className="text-slate-500 dark:text-slate-400 mb-8 sm:mb-12 text-base sm:text-lg leading-relaxed max-w-sm font-bold uppercase tracking-tight opacity-80">{t("dashboard.system.device_management_desc")}</p>
              <Link className="bg-primary-600 hover:bg-primary-50 dark:bg-primary-950 text-white px-10 py-4 rounded-2xl font-bold text-xs uppercase tracking-widest transition-all inline-block shadow-2xl shadow-primary-900/40 active:scale-95" to={ROUTES.SETTINGS.CONNECTIVITY}>
                {t("dashboard.system.device_management_btn").toUpperCase()}
              </Link>
            </div>
            <Cpu className="absolute -right-16 -bottom-16 w-60 sm:w-80 h-60 sm:h-80 text-primary-400 opacity-5 -rotate-12 group-hover:rotate-0 transition-transform duration-1000" />
          </div>
        </section>
      </div>
    </div>
  );
}

export default function DashboardPage() {
  const { t } = useTranslation();
  const [showSystemTab, setShowSystemTab] = useState(false);
  const [customizeOpen, setCustomizeOpen] = useState(false);
  const { formatTime } = useDateTimeFormatter();
  const { formatCurrency } = useCurrency();
  const { agents, isLoading: agentsLoading, error: agentsError } = useAgentsStatus();
  const { devices, isLoading: devicesLoading, error: devicesError } = useDevices();
  const { data: hrStats, isLoading: hrLoading, error: hrError } = useDashboardStats();

  const {
    activeView,
    visibleWidgets,
    widgetOrder,
    collapsedSections,
    isSaving,
    setActiveView,
    toggleWidget,
    setWidgetOrder,
    toggleSection,
    resetToDefault,
    allWidgets,
  } = useDashboardLayout();

  const isLoading = agentsLoading || devicesLoading || hrLoading;
  const error = (agentsError ?? devicesError) ?? hrError;

  const onlineAgents = agents.filter((a) => a.is_online).length;
  const activeDevices = devices.filter((d) => d.is_enabled).length;

  const donutData = useMemo(() => {
    const onTimeCount = Math.max(0, (hrStats?.present_today ?? 0) - (hrStats?.late_today ?? 0));
    return [
      { name: t("dashboard.hr.on_time"), value: onTimeCount, color: DONUT_COLORS.on_time },
      { name: t("dashboard.hr.late_today"), value: hrStats?.late_today ?? 0, color: DONUT_COLORS.late },
      { name: t("dashboard.hr.absent_today"), value: hrStats?.absent_today ?? 0, color: DONUT_COLORS.absent },
      { name: t("dashboard.hr.on_leave_today"), value: hrStats?.on_leave_today ?? 0, color: DONUT_COLORS.on_leave },
    ];
  }, [hrStats, t]);
  const hasDonutData = donutData.some((d) => d.value > 0);

  const chartWidgetIds: WidgetId[] = [
    "attendance_trend",
    "department_chart",
    "overtime_trend",
    "peak_times",
    "leave_types",
    "lateness_by_day",
    "recruitment_sources",
    "turnover_analytics",
    "unexcused_absences",
  ];

  const visibleCharts = useMemo(() =>
    widgetOrder.filter((id) => chartWidgetIds.includes(id) && visibleWidgets[id]),
    [widgetOrder, visibleWidgets]
  );

  const pairedCharts = useMemo(() => {
    const set = new Set<WidgetId>();
    if (visibleWidgets.leave_types && visibleWidgets.lateness_by_day) {
      set.add("leave_types");
      set.add("lateness_by_day");
    }
    return set;
  }, [visibleWidgets.leave_types, visibleWidgets.lateness_by_day]);

  const chartElements = useMemo(() => {
    const rendered = new Set<WidgetId>();
    const elements: React.ReactNode[] = [];

    visibleCharts.forEach((id) => {
      if (rendered.has(id)) return;

      if (pairedCharts.has(id)) {
        const otherId = id === "leave_types" ? "lateness_by_day" : "leave_types";
        if (visibleCharts.includes(otherId) && !rendered.has(otherId)) {
          rendered.add(id);
          rendered.add(otherId);
          elements.push(
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8" key={`pair-${id}`}>
              <WidgetRenderer widgetId="leave_types" />
              <WidgetRenderer widgetId="lateness_by_day" />
            </div>
          );
          return;
        }
      }

      rendered.add(id);
      elements.push(<WidgetRenderer key={id} widgetId={id} />);
    });

    return elements;
  }, [visibleCharts, pairedCharts]);

  return (
    <div className="space-y-8 animate-in fade-in duration-700">
      <DashboardHeader activeView={activeView} setActiveView={setActiveView} setCustomizeOpen={setCustomizeOpen} setShowSystemTab={setShowSystemTab} showSystemTab={showSystemTab} t={t} />

      <div className="animate-in slide-in-from-bottom-4 duration-700">
        {!showSystemTab ? (
          <DashboardHrTab
            chartElements={chartElements}
            collapsedSections={collapsedSections}
            donutData={donutData}
            formatCurrency={formatCurrency}
            hasDonutData={hasDonutData}
            hrStats={hrStats}
            isLoading={isLoading}
            t={t}
            toggleSection={toggleSection}
            visibleWidgets={visibleWidgets}
          />
        ) : (
          <DashboardSystemTab
            activeDevices={activeDevices}
            agents={agents}
            devices={devices}
            formatTime={formatTime}
            isLoading={isLoading}
            onlineAgents={onlineAgents}
            t={t}
          />
        )}
      </div>

      {!!error && (
        <div className="bg-rose-50 dark:bg-rose-950 border-2 border-rose-100 dark:border-rose-800 p-8 sm:p-10 rounded-[3rem] flex items-center gap-6 sm:gap-8 text-rose-700 dark:text-rose-300 animate-in slide-in-from-bottom-8 shadow-2xl shadow-rose-100/50">
          <div className="bg-white dark:bg-slate-900 p-4 sm:p-5 rounded-[1.5rem] shadow-sm border border-rose-100 dark:border-rose-800"><ShieldAlert className="text-rose-500" size={40} /></div>
          <div>
            <p className="font-bold text-xl sm:text-2xl uppercase tracking-tighter leading-none mb-2">{t("dashboard.connection_error")}</p>
            <p className="font-bold opacity-70 uppercase tracking-tight text-sm">{t("dashboard.connection_error_desc")}</p>
          </div>
        </div>
      )}

      <CustomizePanel
        allWidgets={allWidgets}
        isOpen={customizeOpen}
        isSaving={isSaving}
        onClose={() => { setCustomizeOpen(false); }}
        onReorder={setWidgetOrder}
        onReset={resetToDefault}
        onToggleWidget={toggleWidget}
        visibleWidgets={visibleWidgets}
        widgetOrder={widgetOrder}
      />
    </div>
  );
}
