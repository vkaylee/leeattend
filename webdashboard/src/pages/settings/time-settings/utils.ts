import { TFunction } from "i18next";

export function formatDate(iso: string) {
  try {
    return new Date(iso).toLocaleDateString(undefined, {
      day: "numeric",
      month: "short",
      year: "numeric",
    });
  } catch {
    return iso;
  }
}

export function workUnitLabel(v: number, t: TFunction) {
  if (v === 1) return t("common.workspace_settings.work_unit_options.1min");
  if (v === 15) return t("common.workspace_settings.work_unit_options.15min");
  if (v === 30) return t("common.workspace_settings.work_unit_options.30min");
  if (v === 60) return t("common.workspace_settings.work_unit_options.60min");
  return `${v.toString()} min`;
}

export function roundingLabel(v: string, t: TFunction) {
  const map: Record<string, string> = {
    ceil: t("common.workspace_settings.rounding_options.ceil"),
    floor: t("common.workspace_settings.rounding_options.floor"),
    none: t("common.workspace_settings.rounding_options.none"),
    round: t("common.workspace_settings.rounding_options.round"),
  };
  return map[v] || v;
}
