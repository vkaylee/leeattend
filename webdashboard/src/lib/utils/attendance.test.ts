import { describe, it, expect } from 'vitest';
import { formatUnitsToHours } from './attendance';

describe('formatUnitsToHours', () => {
  it('should convert 30-minute units correctly', () => {
    // 17 units * 30 mins = 510 mins = 8.5 hours
    expect(formatUnitsToHours(17, 30)).toBe('8.5');
    // 1 unit * 30 mins = 0.5 hours
    expect(formatUnitsToHours(1, 30)).toBe('0.5');
  });

  it('should convert 60-minute units correctly', () => {
    expect(formatUnitsToHours(8, 60)).toBe('8');
    expect(formatUnitsToHours(1, 60)).toBe('1');
  });

  it('should convert 15-minute units correctly', () => {
    // 1 unit * 15 mins = 0.25 hours
    expect(formatUnitsToHours(1, 15)).toBe('0.25');
    // 3 units * 15 mins = 0.75 hours
    expect(formatUnitsToHours(3, 15)).toBe('0.75');
    // 4 units * 15 mins = 1 hour
    expect(formatUnitsToHours(4, 15)).toBe('1');
  });

  it('should handle zero units', () => {
    expect(formatUnitsToHours(0, 30)).toBe('0');
  });

  it('should handle 1-minute high-precision units', () => {
    expect(formatUnitsToHours(45, 1)).toBe('0.75');
    expect(formatUnitsToHours(60, 1)).toBe('1');
    expect(formatUnitsToHours(90, 1)).toBe('1.5');
  });

  it('should handle rounding of recurring decimals to 2 places', () => {
    // 1 unit of 7 minutes = 0.11666... hours -> 0.12
    expect(formatUnitsToHours(1, 7)).toBe('0.12');
  });
});
