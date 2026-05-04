import { createFileRoute } from "@tanstack/react-router";
import { Bell } from "lucide-react";
import { useTranslation } from "react-i18next";

import PageHeader from "@/components/ui/page-header";

export const Route = createFileRoute("/_authenticated/notifications")({
  component: NotificationsPage,
});

function NotificationsPage() {
  const { t } = useTranslation();

  return (
    <div className="flex h-[calc(100vh-5rem)] flex-col bg-slate-50 dark:bg-slate-900/50">
      <PageHeader
        icon={<Bell />}
        subtitle={t('notifications.description', 'Manage all your notifications here.')}
        title={t('common.notifications', 'Thông báo')}
      />
      <div className="flex-1 p-6 flex items-center justify-center">
        <div className="text-center text-slate-500 dark:text-slate-400">
          <Bell className="mx-auto mb-4 h-12 w-12 opacity-20" />
          <h2 className="text-lg font-medium text-slate-900 dark:text-slate-100">Tính năng đang được phát triển</h2>
          <p className="mt-2 text-sm">Trang View All dành cho Notifications sẽ sớm ra mắt ở Phase 3.</p>
        </div>
      </div>
    </div>
  );
}
