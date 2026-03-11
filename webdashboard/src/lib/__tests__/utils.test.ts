import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { formatTime, formatDateTime, isAgentOnline } from '../utils';

// ---------------------------------------------------------------------------
// formatTime
// ---------------------------------------------------------------------------
describe('formatTime', () => {
  it('returns "--:--" for null', () => {
    expect(formatTime(null, '24h')).toBe('--:--');
  });

  it('returns "--:--" for undefined', () => {
    expect(formatTime(undefined, '24h')).toBe('--:--');
  });

  it('returns "--:--" for empty string', () => {
    expect(formatTime('', '24h')).toBe('--:--');
  });

  it('formats HH:mm in 24h mode', () => {
    expect(formatTime('14:30', '24h')).toBe('14:30');
  });

  it('formats HH:mm in 12h mode', () => {
    expect(formatTime('14:30', '12h')).toBe('02:30 PM');
  });

  it('formats HH:mm:ss in 24h mode', () => {
    expect(formatTime('08:05:30', '24h')).toBe('08:05');
  });

  it('formats HH:mm:ss in 12h mode', () => {
    expect(formatTime('08:05:30', '12h')).toBe('08:05 AM');
  });

  it('handles midnight in 24h', () => {
    expect(formatTime('00:00', '24h')).toBe('00:00');
  });

  it('handles midnight in 12h', () => {
    expect(formatTime('00:00', '12h')).toBe('12:00 AM');
  });

  it('handles undefined timeFormat (defaults to 24h)', () => {
    expect(formatTime('14:30', undefined)).toBe('14:30');
  });
});

// ---------------------------------------------------------------------------
// formatDateTime
// ---------------------------------------------------------------------------
describe('formatDateTime', () => {
  it('returns "--:--" for null', () => {
    expect(formatDateTime(null, '24h')).toBe('--:--');
  });

  it('returns "--:--" for undefined', () => {
    expect(formatDateTime(undefined, '24h')).toBe('--:--');
  });

  it('formats ISO string in 24h mode', () => {
    // Use a fixed UTC date to avoid timezone flakiness
    const result = formatDateTime('2026-01-15T14:30:00Z', '24h');
    expect(result).toMatch(/\d{2}\/\d{2}\/\d{4} \d{2}:\d{2}/);
  });

  it('formats ISO string in 12h mode', () => {
    const result = formatDateTime('2026-01-15T14:30:00Z', '12h');
    expect(result).toMatch(/\d{2}\/\d{2}\/\d{4} \d{2}:\d{2} (AM|PM)/);
  });
});

// ---------------------------------------------------------------------------
// isAgentOnline
// ---------------------------------------------------------------------------
describe('isAgentOnline', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-03-11T10:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns false for null', () => {
    expect(isAgentOnline(null)).toBe(false);
  });

  it('returns false for undefined', () => {
    expect(isAgentOnline(undefined)).toBe(false);
  });

  it('returns false for empty string', () => {
    expect(isAgentOnline('')).toBe(false);
  });

  it('returns true when last seen < 90 seconds ago', () => {
    // 30 seconds ago
    const recent = new Date('2026-03-11T09:59:30Z').toISOString();
    expect(isAgentOnline(recent)).toBe(true);
  });

  it('returns false when last seen > 90 seconds ago', () => {
    // 2 minutes ago
    const stale = new Date('2026-03-11T09:58:00Z').toISOString();
    expect(isAgentOnline(stale)).toBe(false);
  });

  it('returns false when last seen exactly 90 seconds ago', () => {
    // Exactly at threshold (>= 90000ms → false)
    const atThreshold = new Date('2026-03-11T09:58:30Z').toISOString();
    expect(isAgentOnline(atThreshold)).toBe(false);
  });

  it('returns true when last seen 89 seconds ago', () => {
    const justUnder = new Date('2026-03-11T09:58:31Z').toISOString();
    expect(isAgentOnline(justUnder)).toBe(true);
  });
});
