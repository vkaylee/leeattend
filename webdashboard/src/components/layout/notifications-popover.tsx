import { Link } from "@tanstack/react-router";
import { Bell, Check, Clock, Info, CheckCircle2, AlertCircle, MessageSquare } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { formatDistanceToNow } from "date-fns";
import { vi, enUS } from "date-fns/locale";

import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { NotificationType, useNotifications } from "@/hooks/use-notifications";
import { ROUTES } from "@/lib/routes";
import { cn } from "@/lib/utils";

const getNotificationIcon = (type: NotificationType) => {
  switch (type) {
    case 'leave_approved':
      return <CheckCircle2 className="text-emerald-500" size={16} />;
    case 'leave_rejected':
      return <AlertCircle className="text-rose-500" size={16} />;
    case 'attendance_missing':
      return <Clock className="text-amber-500" size={16} />;
    case 'message':
      return <MessageSquare className="text-blue-500" size={16} />;
    case 'system':
    default:
      return <Info className="text-primary-500" size={16} />;
  }
};

export function NotificationsPopover() {
  const { t, i18n } = useTranslation();
  const { notifications, unreadCount, isLoading, markAsRead, markAllAsRead } = useNotifications();
  const [isOpen, setIsOpen] = useState(false);

  const currentLocale = i18n.language === 'vi' ? vi : enUS;

  return (
    <Popover onOpenChange={setIsOpen} open={isOpen}>
      <PopoverTrigger asChild>
        <button className="p-3 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-white dark:hover:bg-slate-800 rounded-[1.25rem] transition-all relative group border border-transparent hover:border-slate-100 dark:hover:border-slate-700 hover:shadow-xl hover:shadow-primary-900/5 outline-none">
          {unreadCount > 0 && (
            <div className="absolute top-2 right-2 flex h-4 w-4 items-center justify-center rounded-full bg-rose-500 border-2 border-white dark:border-slate-900 shadow-sm group-hover:scale-110 transition-transform">
              <span className="text-[9px] font-bold text-white leading-none">
                {unreadCount > 9 ? '9+' : unreadCount}
              </span>
            </div>
          )}
          <Bell size={20} strokeWidth={2.5} />
        </button>
      </PopoverTrigger>
      
      <PopoverContent align="end" className="w-80 p-0 overflow-hidden shadow-2xl border-slate-100 dark:border-slate-800 rounded-2xl" sideOffset={8}>
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-slate-100 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-900/50">
          <h3 className="font-bold text-sm text-slate-800 dark:text-slate-200">
            {t('common.notifications', { defaultValue: 'Thông báo' })}
          </h3>
          {unreadCount > 0 && (
            <button 
              className="text-xs font-medium text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 flex items-center gap-1 transition-colors"
              onClick={markAllAsRead}
            >
              <Check size={14} />
              Đánh dấu đã đọc
            </button>
          )}
        </div>

        {/* Body */}
        <div className="max-h-[350px] overflow-y-auto">
          {isLoading ? (
            <div className="p-8 text-center text-slate-500 dark:text-slate-400">
              <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary-600 mx-auto mb-2"></div>
              <p className="text-xs">Đang tải...</p>
            </div>
          ) : notifications.length === 0 ? (
            <div className="p-8 text-center text-slate-500 dark:text-slate-400 flex flex-col items-center">
              <Bell className="opacity-20 mb-2" size={24} />
              <p className="text-sm font-medium">Chưa có thông báo nào</p>
              <p className="text-xs opacity-70 mt-1">Bạn đã xem hết tất cả thông báo.</p>
            </div>
          ) : (
            <div className="flex flex-col">
              {notifications.slice(0, 5).map((notification) => (
                <button 
                  className={cn(
                    "flex gap-3 p-4 border-b border-slate-100 dark:border-slate-800/50 transition-colors cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800/50 relative group text-left w-full",
                    !notification.isRead ? "bg-primary-50/30 dark:bg-primary-900/10" : ""
                  )}
                  key={notification.id}
                  onClick={() => {
                    markAsRead(notification.id);
                    // Could add routing logic if notification.link exists
                  }}
                >
                  {!notification.isRead && (
                    <div className="absolute left-0 top-0 bottom-0 w-1 bg-primary-500 rounded-r-full"></div>
                  )}
                  
                  <div className="mt-0.5 flex-shrink-0">
                    {getNotificationIcon(notification.type)}
                  </div>
                  
                  <div className="flex-1 min-w-0">
                    <p className={cn(
                      "text-sm font-medium leading-tight mb-1",
                      !notification.isRead ? "text-slate-900 dark:text-slate-100" : "text-slate-700 dark:text-slate-300"
                    )}>
                      {notification.title}
                    </p>
                    <p className="text-xs text-slate-500 dark:text-slate-400 line-clamp-2 mb-1.5 leading-relaxed">
                      {notification.content}
                    </p>
                    <p className="text-[10px] text-slate-400 dark:text-slate-500 font-medium">
                      {formatDistanceToNow(new Date(notification.createdAt), { addSuffix: true, locale: currentLocale })}
                    </p>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-2 border-t border-slate-100 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-900/50 text-center">
          <Link 
            className="text-xs font-bold text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 py-2 px-4 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors inline-block w-full"
            onClick={() => { setIsOpen(false); }}
            // @ts-expect-error - Route generated dynamically
            to={ROUTES.NOTIFICATIONS}
          >
            Xem tất cả thông báo
          </Link>
        </div>
      </PopoverContent>
    </Popover>
  );
}
