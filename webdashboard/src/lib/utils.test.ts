import { describe, it, expect } from 'vitest';
import { formatTime, formatDateTime } from './utils';

describe('Utility Functions', () => {
  describe('formatTime', () => {
    it('should format time in 24h format by default', () => {
      expect(formatTime('14:30:00', '24h')).toBe('14:30');
      expect(formatTime('09:15', '24h')).toBe('09:15');
    });

    it('should format time in 12h format when configured', () => {
      expect(formatTime('14:30:00', '12h')).toBe('02:30 PM');
      expect(formatTime('09:15', '12h')).toBe('09:15 AM');
      expect(formatTime('00:00', '12h')).toBe('12:00 AM');
      expect(formatTime('12:00', '12h')).toBe('12:00 PM');
    });

    it('should handle null or undefined timeStr', () => {
      expect(formatTime(null, '24h')).toBe('--:--');
      expect(formatTime(undefined, '12h')).toBe('--:--');
    });

    it('should handle invalid time format gracefully', () => {
      expect(formatTime('invalid', '24h')).toBe('inval');
    });
  });

  describe('formatDateTime', () => {
    it('should format date and time in 24h format', () => {
      const dt = '2026-01-13T14:30:00Z';
      // formatDateTime uses local time, so we need to be careful with UTC in tests
      // But for simple verification of the pattern:
      const result = formatDateTime(dt, '24h');
      expect(result).toMatch(/\d{2}\/\d{2}\/\d{4} \d{2}:\d{2}/);
    });

    it('should format date and time in 12h format', () => {
      const dt = '2026-01-13T14:30:00Z';
      const result = formatDateTime(dt, '12h');
      expect(result).toMatch(/\d{2}\/\d{2}\/\d{4} \d{2}:\d{2} [AP]M/);
    });

    it('should handle null or undefined dateTimeStr', () => {
      expect(formatDateTime(null, '24h')).toBe('--:--');
      expect(formatDateTime(undefined, '12h')).toBe('--:--');
    });
  });
});
