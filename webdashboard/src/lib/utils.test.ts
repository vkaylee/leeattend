import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { isAgentOnline } from './utils';

describe('isAgentOnline', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should return true if last seen is within 90 seconds', () => {
    const now = new Date('2026-01-08T10:00:00Z');
    vi.setSystemTime(now);

    const lastSeen = new Date('2026-01-08T09:59:00Z').toISOString(); // 60s ago
    expect(isAgentOnline(lastSeen)).toBe(true);
  });

  it('should return false if last seen is more than 90 seconds ago', () => {
    const now = new Date('2026-01-08T10:00:00Z');
    vi.setSystemTime(now);

    const lastSeen = new Date('2026-01-08T09:58:20Z').toISOString(); // 100s ago
    expect(isAgentOnline(lastSeen)).toBe(false);
  });

  it('should return false if last seen is null or undefined', () => {
    expect(isAgentOnline(null)).toBe(false);
    expect(isAgentOnline(undefined)).toBe(false);
    expect(isAgentOnline('')).toBe(false);
  });
});
