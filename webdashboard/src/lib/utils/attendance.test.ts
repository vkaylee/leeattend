import { describe, it, expect } from 'vitest';

import { formatHours } from './attendance';

describe('formatHours', () => {
  it('should format whole numbers without decimals', () => {
    expect(formatHours(8)).toBe('8');
    expect(formatHours(1)).toBe('1');
    expect(formatHours(0)).toBe('0');
  });

  it('should format floating point numbers up to 2 decimal places', () => {
    expect(formatHours(8.5)).toBe('8.5');
    expect(formatHours(0.5)).toBe('0.5');
    expect(formatHours(0.25)).toBe('0.25');
    expect(formatHours(0.75)).toBe('0.75');
    expect(formatHours(1.5)).toBe('1.5');
  });

  it('should handle rounding of recurring decimals to 2 places', () => {
    // 0.11666... hours -> 0.12
    expect(formatHours(0.116666)).toBe('0.12');
    expect(formatHours(1.125)).toBe('1.13'); // rounds half up depending on Intl API, usually up
  });
});
