import { useState, useEffect } from 'react';

export type NotificationType = 'leave_approved' | 'leave_rejected' | 'attendance_missing' | 'system' | 'message';

export interface AppNotification {
  content: string;
  createdAt: string;
  id: string;
  isRead: boolean;
  link?: string;
  title: string;
  type: NotificationType;
}

export const MOCK_NOTIFICATIONS: AppNotification[] = [
  {
    id: 'n1',
    type: 'leave_approved',
    title: 'Nghỉ phép được phê duyệt',
    content: 'Đơn xin nghỉ phép ngày 05/05/2026 của bạn đã được Admin phê duyệt.',
    isRead: false,
    createdAt: new Date(Date.now() - 1000 * 60 * 5).toISOString(),
    link: '/leave-requests'
  },
  {
    id: 'n2',
    type: 'attendance_missing',
    title: 'Quên chấm công',
    content: 'Bạn chưa chấm công chiều ngày hôm qua. Vui lòng gửi yêu cầu bổ sung.',
    isRead: false,
    createdAt: new Date(Date.now() - 1000 * 60 * 60 * 2).toISOString(),
    link: '/attendance-adjustments'
  },
  {
    id: 'n3',
    type: 'system',
    title: 'Bảo trì hệ thống',
    content: 'Hệ thống sẽ bảo trì từ 22:00 đến 23:00 tối nay. Vui lòng lưu lại công việc.',
    isRead: true,
    createdAt: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(),
  },
];

export function useNotifications() {
  const [notifications, setNotifications] = useState<AppNotification[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  // Simulate API fetch
  useEffect(() => {
    const timer = setTimeout(() => {
      setNotifications(MOCK_NOTIFICATIONS);
      setIsLoading(false);
    }, 800);
    return () => { clearTimeout(timer); };
  }, []);

  const unreadCount = notifications.filter(n => !n.isRead).length;

  const markAsRead = (id: string) => {
    setNotifications(prev => 
      prev.map(n => n.id === id ? { ...n, isRead: true } : n)
    );
  };

  const markAllAsRead = () => {
    setNotifications(prev => 
      prev.map(n => ({ ...n, isRead: true }))
    );
  };

  return {
    notifications,
    unreadCount,
    isLoading,
    markAsRead,
    markAllAsRead
  };
}
