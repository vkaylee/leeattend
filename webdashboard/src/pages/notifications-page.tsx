import { Bell, CheckCircle2, Clock, Info, MessageSquare, AlertCircle, Check } from "lucide-react";
import React, { memo, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { formatDistanceToNow, type Locale } from "date-fns";
import { enUS, vi } from "date-fns/locale";

import { ErrorBoundary } from "@/components/common/error-boundary";
import Button from "@/components/ui/button";
import { PageHeader } from "@/components/ui/page-header";
import { Typography } from "@/components/ui/typography";
import { useNotifications } from "@/hooks/use-notifications";
import { Notification, NotificationType } from "@/lib/api";
import { cn } from "@/lib/utils";

const getNotificationIcon = (type: NotificationType) => {
  switch (type) {
    case "leave_approved":
      return <CheckCircle2 className="text-emerald-500" size={20} />;
    case "leave_rejected":
      return <AlertCircle className="text-rose-500" size={20} />;
    case "attendance_missing":
      return <Clock className="text-amber-500" size={20} />;
    case "message":
      return <MessageSquare className="text-blue-500" size={20} />;
    case "system":
    default:
      return <Info className="text-primary-500" size={20} />;
  }
};

const NotificationItem = memo(({
  notification,
  currentLocale,
  onMarkAsRead,
  t
}: {
  notification: Notification;
  currentLocale: Locale;
  onMarkAsRead: (id: string) => void;
  t: (key: string, defaultValue: string) => string;
}) => {
  const handleMarkAsRead = useCallback(() => {
    onMarkAsRead(notification.id);
  }, [onMarkAsRead, notification.id]);

  return (
    <div
      className={cn(
        "group p-4 sm:p-5 transition-all relative flex gap-4 hover:bg-slate-50 dark:hover:bg-slate-700/50",
        !notification.is_read ? "bg-primary-50/20 dark:bg-primary-900/10" : ""
      )}
    >
      {!notification.is_read && (
        <div className="absolute left-0 top-0 bottom-0 w-1 bg-primary-500" />
      )}

      <div className="mt-1 flex-shrink-0">
        <div className="w-10 h-10 rounded-full bg-white dark:bg-slate-900 shadow-sm border border-slate-100 dark:border-slate-700 flex items-center justify-center group-hover:scale-110 transition-transform">
          {getNotificationIcon(notification.type)}
        </div>
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-start justify-between gap-2 mb-1">
          <Typography className={cn(
            "text-sm font-bold truncate transition-colors",
            !notification.is_read ? "text-slate-900 dark:text-slate-100" : "text-slate-600 dark:text-slate-400"
          )} variant="h4">
            {notification.title}
          </Typography>
          <span className="text-[10px] text-slate-400 dark:text-slate-500 whitespace-nowrap font-medium">
            {formatDistanceToNow(new Date(notification.created_at), { addSuffix: true, locale: currentLocale })}
          </span>
        </div>

        <Typography className={cn(
          "text-sm leading-relaxed mb-3",
          !notification.is_read ? "text-slate-700 dark:text-slate-300" : "text-slate-500 dark:text-slate-500"
        )} variant="p">
          {notification.content}
        </Typography>

        {!notification.is_read && (
          <Button
            className="h-7 text-[11px] px-2 text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 hover:bg-primary-50 dark:hover:bg-primary-900/30 font-bold"
            onClick={handleMarkAsRead}
            size="sm"
            variant="ghost"
          >
            {t("notifications.mark_as_read", "Đánh dấu đã đọc")}
          </Button>
        )}
      </div>
    </div>
  );
});

NotificationItem.displayName = "NotificationItem";

const NotificationsList = memo(({
  notifications,
  isLoading,
  hasNextPage,
  isFetchingNextPage,
  fetchNextPage,
  markAsRead,
  currentLocale,
  t
}: {
  notifications: Notification[];
  isLoading: boolean;
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  fetchNextPage: () => void;
  markAsRead: (id: string) => void;
  currentLocale: Locale;
  t: (key: string, defaultValue: string) => string;
}) => {
  const handleFetchNext = useCallback(() => {
    fetchNextPage();
  }, [fetchNextPage]);

  if (isLoading) {
    return (
      <div className="space-y-4">
        {Array.from({ length: 5 }).map((_, i) => (
          <div className="h-24 bg-white dark:bg-slate-800 rounded-xl animate-pulse border border-slate-100 dark:border-slate-700" key={i} />
        ))}
      </div>
    );
  }

  if (notifications.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-20 text-center">
        <div className="w-20 h-20 bg-slate-100 dark:bg-slate-800 rounded-full flex items-center justify-center mb-4">
          <Bell className="h-10 w-10 text-slate-300 dark:text-slate-600" />
        </div>
        <Typography className="text-lg font-medium text-slate-900 dark:text-slate-100" variant="h3">
          {t("notifications.no_notifications", "Không có thông báo nào")}
        </Typography>
        <Typography className="text-sm text-slate-500 dark:text-slate-400 mt-1 max-w-xs mx-auto" variant="p">
          {t("notifications.empty_state", "Bạn chưa có thông báo nào. Các cập nhật quan trọng sẽ xuất hiện ở đây.")}
        </Typography>
      </div>
    );
  }

  return (
    <>
      <div className="bg-white dark:bg-slate-800 rounded-2xl shadow-sm border border-slate-200 dark:border-slate-700 divide-y divide-slate-100 dark:divide-slate-700 overflow-hidden">
        {notifications.map((notification) => (
          <NotificationItem
            currentLocale={currentLocale}
            key={notification.id}
            notification={notification}
            onMarkAsRead={markAsRead}
            t={t}
          />
        ))}
      </div>

      {hasNextPage ? (
        <div className="flex justify-center py-8">
          <Button
            className="min-w-[150px]"
            disabled={isFetchingNextPage}
            onClick={handleFetchNext}
            size="md"
            variant="outline"
          >
            {isFetchingNextPage ? (
              <div className="flex items-center gap-2">
                <div className="w-1.5 h-1.5 bg-current rounded-full animate-bounce [animation-delay:-0.3s]" />
                <div className="w-1.5 h-1.5 bg-current rounded-full animate-bounce [animation-delay:-0.15s]" />
                <div className="w-1.5 h-1.5 bg-current rounded-full animate-bounce" />
              </div>
            ) : (
              t("common.load_more", "Tải thêm")
            )}
          </Button>
        </div>
      ) : null}
    </>
  );
});

NotificationsList.displayName = "NotificationsList";

export default function NotificationsPage() {
  const { t, i18n } = useTranslation();
  const {
    notifications,
    isLoading,
    hasNextPage,
    fetchNextPage,
    isFetchingNextPage,
    markAsRead,
    markAllAsRead,
    isMarkingAllAsRead
  } = useNotifications(30);

  const currentLocale = i18n.language === "vi" ? vi : enUS;

  const handleMarkAllAsRead = useCallback(() => {
    void markAllAsRead();
  }, [markAllAsRead]);

  return (
    <ErrorBoundary variant="full">
      <div className="animate-in fade-in duration-700 space-y-6 pb-20">
        <PageHeader
        actions={
          notifications.length > 0 ? (
            <Button
              className="hidden sm:flex items-center gap-2"
              disabled={isMarkingAllAsRead}
              onClick={handleMarkAllAsRead}
              size="sm"
              variant="outline"
            >
              <Check size={16} />
              {t("notifications.mark_all_as_read", "Đánh dấu tất cả đã đọc")}
            </Button>
          ) : null
        }
        icon={<Bell className="text-primary-500" />}
        subtitle={t("notifications.description", "Quản lý tất cả thông báo của bạn tại đây.")}
        title={t("common.notifications", "Thông báo")}
      />

      <div className="max-w-4xl mx-auto space-y-4">
        <NotificationsList
          currentLocale={currentLocale}
          fetchNextPage={fetchNextPage}
          hasNextPage={hasNextPage}
          isFetchingNextPage={isFetchingNextPage}
          isLoading={isLoading}
          markAsRead={markAsRead}
          notifications={notifications}
          t={t}
        />
      </div>
    </div>
    </ErrorBoundary>
  );
}
