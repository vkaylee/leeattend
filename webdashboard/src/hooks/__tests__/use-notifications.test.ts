import { act, renderHook } from '@testing-library/react';

import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';

import { MOCK_NOTIFICATIONS, useNotifications } from '../use-notifications';

describe('useNotifications', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should initialize with loading state and empty notifications', () => {
    const { result } = renderHook(() => useNotifications());

    expect(result.current.isLoading).toBe(true);
    expect(result.current.notifications).toEqual([]);
    expect(result.current.unreadCount).toBe(0);
  });

  it('should populate notifications after load delay', () => {
    const { result } = renderHook(() => useNotifications());

    // Fast-forward past the setTimeout delay (800ms)
    act(() => {
      vi.advanceTimersByTime(800);
    });

    expect(result.current.isLoading).toBe(false);
    expect(result.current.notifications).toEqual(MOCK_NOTIFICATIONS);
    
    const unreadCount = MOCK_NOTIFICATIONS.filter(n => !n.isRead).length;
    expect(result.current.unreadCount).toBe(unreadCount);
  });

  it('should mark a specific notification as read', () => {
    const { result } = renderHook(() => useNotifications());

    act(() => {
      vi.advanceTimersByTime(800);
    });

    const initialUnreadCount = result.current.unreadCount;
    const firstUnread = result.current.notifications.find(n => !n.isRead);
    expect(firstUnread).toBeDefined();

    if (firstUnread) {
      act(() => {
        result.current.markAsRead(firstUnread.id);
      });

      const updatedNotification = result.current.notifications.find(n => n.id === firstUnread.id);
      expect(updatedNotification?.isRead).toBe(true);
      expect(result.current.unreadCount).toBe(initialUnreadCount - 1);
    }
  });

  it('should mark all notifications as read', () => {
    const { result } = renderHook(() => useNotifications());

    act(() => {
      vi.advanceTimersByTime(800);
    });

    expect(result.current.unreadCount).toBeGreaterThan(0);

    act(() => {
      result.current.markAllAsRead();
    });

    expect(result.current.unreadCount).toBe(0);
    expect(result.current.notifications.every(n => n.isRead)).toBe(true);
  });
});
