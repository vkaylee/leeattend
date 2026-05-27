
type UnitType = 'DAY' | 'SHIFT' | 'HOUR' | 'MINUTE';

const UNIT_TYPE_I18N_MAP: Record<UnitType, string> = {
  DAY: 'reports.timekeeping_unit.day',
  SHIFT: 'reports.timekeeping_unit.shift',
  HOUR: 'reports.timekeeping_unit.hour',
  MINUTE: 'reports.timekeeping_unit.minute',
};

/**
 * Maps API enum unit_type (DAY/SHIFT/HOUR/MINUTE) to i18n translation key.
 * Falls back to the raw value if not found.
 */
export const getUnitTypeI18nKey = (unitType: string): string =>
  UNIT_TYPE_I18N_MAP[unitType as UnitType] || unitType;

/**
 * Build a tooltip string showing the detailed unit-to-hours conversion.
 * Example: "16 Công × 0.5h = 8h"
 */
export const buildWorkUnitsTooltip = (
  units: number,
  unitLabel: string,
  coefficientHours: number,
  computedHours: number,
): string => {
  const formattedHours = formatHours(computedHours);
  const formattedCoeff = formatHours(coefficientHours);
  return `${units} ${unitLabel} × ${formattedCoeff}h = ${formattedHours}h`;
};

export const formatHours = (hours: number): string => {
  return new Intl.NumberFormat(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  }).format(hours);
};
