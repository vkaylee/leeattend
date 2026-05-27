import { describe, it, expect } from 'vitest';

import { formatHours, getUnitTypeI18nKey, buildWorkUnitsTooltip } from './attendance';

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

describe('getUnitTypeI18nKey', () => {
  it('should map known unit types to i18n keys', () => {
    expect(getUnitTypeI18nKey('DAY')).toBe('reports.timekeeping_unit.day');
    expect(getUnitTypeI18nKey('SHIFT')).toBe('reports.timekeeping_unit.shift');
    expect(getUnitTypeI18nKey('HOUR')).toBe('reports.timekeeping_unit.hour');
    expect(getUnitTypeI18nKey('MINUTE')).toBe('reports.timekeeping_unit.minute');
  });

  it('should fall back to the raw value for unknown types', () => {
    expect(getUnitTypeI18nKey('UNKNOWN')).toBe('UNKNOWN');
    expect(getUnitTypeI18nKey('')).toBe('');
  });
});

describe('buildWorkUnitsTooltip', () => {
  it('should build a tooltip with the unit-to-hours conversion', () => {
    // 16 Công × 0.5h = 8h
    expect(buildWorkUnitsTooltip(16, 'Công', 0.5, 8)).toBe('16 Công × 0.5h = 8h');
  });

  it('should handle whole hour coefficients', () => {
    // 1 Day × 8h = 8h
    expect(buildWorkUnitsTooltip(1, 'Day', 8, 8)).toBe('1 Day × 8h = 8h');
  });

  it('should handle zero values', () => {
    expect(buildWorkUnitsTooltip(0, 'Day', 8, 0)).toBe('0 Day × 8h = 0h');
  });
});
