import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';

import { describe, expect, it, vi, beforeEach } from 'vitest';

import { NotificationsPopover } from '../notifications-popover';

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => (
    <a data-testid="mock-link" href={to} {...props}>
      {children}
    </a>
  ),
}));

// Mock react-i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string }) => options?.defaultValue ?? key,
    i18n: { language: 'vi' },
  }),
}));

// Mock useNotifications hook
const mockMarkAsRead = vi.fn();
const mockMarkAllAsRead = vi.fn();

vi.mock('@/hooks/use-notifications', () => ({
  useNotifications: () => ({
    notifications: [
      {
        id: '1',
        title: 'Test Notification 1',
        content: 'Content 1',
        type: 'system',
        isRead: false,
        createdAt: new Date().toISOString(),
      },
      {
        id: '2',
        title: 'Test Notification 2',
        content: 'Content 2',
        type: 'leave_approved',
        isRead: true,
        createdAt: new Date().toISOString(),
      },
    ],
    unreadCount: 1,
    isLoading: false,
    markAsRead: mockMarkAsRead,
    markAllAsRead: mockMarkAllAsRead,
  }),
  NotificationType: {
    SYSTEM: 'system',
    LEAVE_APPROVED: 'leave_approved',
  }
}));

describe('NotificationsPopover', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the bell icon and unread badge', () => {
    render(<NotificationsPopover />);
    
    // Check if the trigger button is rendered (it contains the bell icon)
    const triggerBtn = screen.getByRole('button');
    expect(triggerBtn).toBeInTheDocument();

    // Check if unread badge is rendered with '1'
    expect(screen.getByText('1')).toBeInTheDocument();
  });

  it('opens the popover and displays notifications when clicked', async () => {
    render(<NotificationsPopover />);
    
    const triggerBtn = screen.getByRole('button');
    fireEvent.click(triggerBtn);

    // Wait for the popover content to be rendered
    const title1 = await screen.findByText('Test Notification 1');
    expect(title1).toBeInTheDocument();
    
    const title2 = screen.getByText('Test Notification 2');
    expect(title2).toBeInTheDocument();

    // Check if "Xem tất cả thông báo" link is rendered
    expect(screen.getByText('Xem tất cả thông báo')).toBeInTheDocument();
  });

  it('calls markAllAsRead when "Đánh dấu đã đọc" is clicked', async () => {
    render(<NotificationsPopover />);
    
    const triggerBtn = screen.getByRole('button');
    fireEvent.click(triggerBtn);

    const markAllBtn = await screen.findByText('Đánh dấu đã đọc');
    fireEvent.click(markAllBtn);

    expect(mockMarkAllAsRead).toHaveBeenCalledTimes(1);
  });

  it('calls markAsRead when a specific notification is clicked', async () => {
    render(<NotificationsPopover />);
    
    const triggerBtn = screen.getByRole('button');
    fireEvent.click(triggerBtn);

    const notificationItem = await screen.findByText('Test Notification 1');
    fireEvent.click(notificationItem);

    expect(mockMarkAsRead).toHaveBeenCalledWith('1');
  });
});
