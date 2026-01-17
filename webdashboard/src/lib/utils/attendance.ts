export const formatUnitsToHours = (units: number, workUnitMinutes: number): string => {
  const totalMinutes = units * workUnitMinutes;
  const hours = totalMinutes / 60;
  
  // Format as number with fixed decimal places, but hide trailing zeros if not needed
  return new Intl.NumberFormat(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  }).format(hours);
};
