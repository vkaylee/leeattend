import { describe, it, expect } from 'vitest';

import { toUTCISO } from '../datetime';

describe('toUTCISO', () => {
  it('converts UTC timezone correctly (no offset)', () => {
    const result = toUTCISO('2026-01-15', '14:30', 'UTC');
    const date = new Date(result);
    expect(date.getUTCHours()).toBe(14);
    expect(date.getUTCMinutes()).toBe(30);
    expect(date.getUTCFullYear()).toBe(2026);
    expect(date.getUTCMonth()).toBe(0); // January
    expect(date.getUTCDate()).toBe(15);
  });

  it('converts Asia/Ho_Chi_Minh (+7) correctly', () => {
    // 14:30 local VN = 07:30 UTC
    const result = toUTCISO('2026-01-15', '14:30', 'Asia/Ho_Chi_Minh');
    const date = new Date(result);
    expect(date.getUTCHours()).toBe(7);
    expect(date.getUTCMinutes()).toBe(30);
  });

  it('converts US/Eastern (-5 in winter) correctly', () => {
    // 14:30 EST = 19:30 UTC (January = EST, not EDT)
    const result = toUTCISO('2026-01-15', '14:30', 'US/Eastern');
    const date = new Date(result);
    expect(date.getUTCHours()).toBe(19);
    expect(date.getUTCMinutes()).toBe(30);
  });

  it('handles midnight conversion', () => {
    // 00:00 VN = 17:00 previous day UTC
    const result = toUTCISO('2026-01-15', '00:00', 'Asia/Ho_Chi_Minh');
    const date = new Date(result);
    expect(date.getUTCHours()).toBe(17);
    expect(date.getUTCDate()).toBe(14); // Previous day
  });

  it('handles end of day conversion', () => {
    // 23:59 VN = 16:59 UTC same day
    const result = toUTCISO('2026-01-15', '23:59', 'Asia/Ho_Chi_Minh');
    const date = new Date(result);
    expect(date.getUTCHours()).toBe(16);
    expect(date.getUTCMinutes()).toBe(59);
  });

  it('returns ISO string format', () => {
    const result = toUTCISO('2026-06-15', '10:00', 'UTC');
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{3}Z$/);
  });

  it('falls back gracefully for invalid timezone', () => {
    // Should not throw, returns a fallback
    const result = toUTCISO('2026-01-15', '14:30', 'Invalid/Timezone');
    expect(typeof result).toBe('string');
    expect(result.length).toBeGreaterThan(0);
  });
});
