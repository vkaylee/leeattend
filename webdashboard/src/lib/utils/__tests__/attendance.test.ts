import { describe, it, expect } from 'vitest';
import { formatUnitsToHours } from '../attendance';

describe('formatUnitsToHours', () => {
  it('returns "0" for 0 units', () => {
    expect(formatUnitsToHours(0, 30)).toBe('0');
  });

  it('converts 1 unit at 30min to 0.5 hours', () => {
    expect(formatUnitsToHours(1, 30)).toBe('0.5');
  });

  it('converts 2 units at 30min to 1 hour', () => {
    expect(formatUnitsToHours(2, 30)).toBe('1');
  });

  it('converts 16 units at 30min to 8 hours', () => {
    expect(formatUnitsToHours(16, 30)).toBe('8');
  });

  it('converts 1 unit at 60min to 1 hour', () => {
    expect(formatUnitsToHours(1, 60)).toBe('1');
  });

  it('converts 1 unit at 15min to 0.25 hours', () => {
    expect(formatUnitsToHours(1, 15)).toBe('0.25');
  });

  it('handles large values', () => {
    // 320 units * 30min = 160 hours
    expect(formatUnitsToHours(320, 30)).toBe('160');
  });

  it('handles fractional results with proper formatting', () => {
    // 3 units * 30min = 90min = 1.5 hours
    expect(formatUnitsToHours(3, 30)).toBe('1.5');
  });
});
