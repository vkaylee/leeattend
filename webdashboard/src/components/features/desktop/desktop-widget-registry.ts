import {
  Clock,
  Activity,
  Users,
  CheckSquare,
  Sparkles,
  TrendingUp,
} from "lucide-react";

import type { DesktopWidgetDefinition } from "./types";

import { ExecutiveKpiWidget } from "./widgets/executive-kpi-widget";
import { DesktopPendingTasksWidget } from "./widgets/pending-tasks-widget";
import { QuickPunchWidget } from "./widgets/quick-punch-widget";
import { QuickShortcutsWidget } from "./widgets/quick-shortcuts-widget";
import { SystemHealthWidget } from "./widgets/system-health-widget";
import { TodayOverviewWidget } from "./widgets/today-overview-widget";

export const DESKTOP_WIDGET_REGISTRY: DesktopWidgetDefinition[] = [
  {
    id: "quick-punch",
    titleKey: "desktop.widgets.quickPunch.title",
    titleFallback: "Bấm giờ chấm công",
    descriptionKey: "desktop.widgets.quickPunch.description",
    descriptionFallback: "Đồng hồ thời gian thực và ghi nhận vào/ra ca nhanh",
    category: "attendance",
    defaultEnabled: true,
    defaultOrder: 1,
    colSpan: "col-span-1",
    icon: Clock,
    component: QuickPunchWidget,
  },
  {
    id: "today-overview",
    titleKey: "desktop.widgets.todayOverview.title",
    titleFallback: "Chuyên cần hôm nay",
    descriptionKey: "desktop.widgets.todayOverview.description",
    descriptionFallback: "Số lượng nhân sự có mặt, vắng mặt, nghỉ phép và tỷ lệ chuyên cần",
    category: "overview",
    defaultEnabled: true,
    defaultOrder: 2,
    colSpan: "col-span-1",
    icon: Users,
    component: TodayOverviewWidget,
    permissions: ["attendance:logs:read"],
  },
  {
    id: "quick-shortcuts",
    titleKey: "desktop.widgets.quickShortcuts.title",
    titleFallback: "Thao tác nhanh",
    descriptionKey: "desktop.widgets.quickShortcuts.description",
    descriptionFallback: "Lối tắt đến các chức năng thường dùng",
    category: "overview",
    defaultEnabled: true,
    defaultOrder: 3,
    colSpan: "col-span-1",
    icon: Sparkles,
    component: QuickShortcutsWidget,
  },
  {
    id: "system-health",
    titleKey: "desktop.widgets.systemHealth.title",
    titleFallback: "Trạng thái hệ thống",
    descriptionKey: "desktop.widgets.systemHealth.description",
    descriptionFallback: "Giám sát thời gian thực API, Database, Redis, MinIO và Workers",
    category: "system",
    defaultEnabled: true,
    defaultOrder: 4,
    colSpan: "col-span-1",
    icon: Activity,
    component: SystemHealthWidget,
  },
  {
    id: "pending-tasks",
    titleKey: "desktop.widgets.pendingTasks.title",
    titleFallback: "Việc cần xử lý ngay",
    descriptionKey: "desktop.widgets.pendingTasks.description",
    descriptionFallback: "Checklist quy trình tiếp nhận và thủ tục nhân sự cần giải quyết",
    category: "tasks",
    defaultEnabled: true,
    defaultOrder: 5,
    colSpan: "col-span-1 xl:col-span-2",
    icon: CheckSquare,
    component: DesktopPendingTasksWidget,
  },
  {
    id: "executive-kpi",
    titleKey: "desktop.widgets.executiveKpi.title",
    titleFallback: "Chỉ số điều hành",
    descriptionKey: "desktop.widgets.executiveKpi.description",
    descriptionFallback: "Dự toán quỹ lương tháng và tỷ lệ hoàn thành công việc",
    category: "analytics",
    defaultEnabled: true,
    defaultOrder: 6,
    colSpan: "col-span-1 xl:col-span-2",
    icon: TrendingUp,
    component: ExecutiveKpiWidget,
    permissions: ["payroll:read", "reports:read"],
  },
];

export function filterDesktopWidgets(
  widgets: DesktopWidgetDefinition[],
  hasAnyPermission: (...perms: string[]) => boolean,
): DesktopWidgetDefinition[] {
  return widgets.filter((w) => {
    if (w.permissions && w.permissions.length > 0) {
      return hasAnyPermission(...w.permissions);
    }
    return true;
  });
}

export function getDefaultEnabledWidgetIds(): string[] {
  return DESKTOP_WIDGET_REGISTRY.filter((w) => w.defaultEnabled).map((w) => w.id);
}

export function getDefaultWidgetOrder(): string[] {
  return [...DESKTOP_WIDGET_REGISTRY]
    .sort((a, b) => a.defaultOrder - b.defaultOrder)
    .map((w) => w.id);
}
