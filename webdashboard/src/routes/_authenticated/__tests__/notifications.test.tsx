import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';

import { describe, expect, it, vi, beforeEach } from 'vitest';

import { NotificationsPage } from '../notifications';

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  createFileRoute: () => () => ({}),
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
const mockFetchNextPage = vi.fn();

vi.mock('@/hooks/use-notifications', () => ({
  useNotifications: () => ({
    notifications: [
      {
        id: '1',
        title: 'Test Notification 1',
        content: 'Content 1',
        type: 'system',
        is_read: false,
        created_at: new Date().toISOString(),
      },
      {
        id: '2',
        title: 'Test Notification 2',
        content: 'Content 2',
        type: 'leave_approved',
        is_read: true,
        created_at: new Date().toISOString(),
      },
    ],
    unreadCount: 1,
    isLoading: false,
    hasNextPage: true,
    isFetchingNextPage: false,
    markAsRead: mockMarkAsRead,
    markAllAsRead: mockMarkAllAsRead,
    fetchNextPage: mockFetchNextPage,
  }),
}));

describe('NotificationsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the notifications list and header', () => {
    render(<NotificationsPage />);
    
    // Check if the title is rendered
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(/common\.notifications/i);
    expect(screen.getByText(/notifications\.description/i)).toBeInTheDocument();

    // Check if notifications are rendered
    expect(screen.getByText('Test Notification 1')).toBeInTheDocument();
    expect(screen.getByText('Test Notification 2')).toBeInTheDocument();
  });

  it('calls markAllAsRead when the button is clicked', () => {
    render(<NotificationsPage />);
    
    const markAllBtn = screen.getByText('notifications.mark_all_as_read');
    fireEvent.click(markAllBtn);

    expect(mockMarkAllAsRead).toHaveBeenCalledTimes(1);
  });

  it('calls fetchNextPage when "Load More" is clicked', () => {
    render(<NotificationsPage />);
    
    const loadMoreBtn = screen.getByText('common.load_more');
    fireEvent.click(loadMoreBtn);

    expect(mockFetchNextPage).toHaveBeenCalledTimes(1);
  });

  it('calls markAsRead when a notification mark as read button is clicked', () => {
    render(<NotificationsPage />);
    
    // Find all "Mark as read" buttons (t mock returns key)
    const markAsReadBtns = screen.getAllByText(/notifications\.mark_as_read/i);
    fireEvent.click(markAsReadBtns[0]);

    expect(mockMarkAsRead).toHaveBeenCalledWith('1');
  });
});
