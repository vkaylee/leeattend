import { useCallback } from "react";
import { useTranslation } from "react-i18next";

// ── Currency Configuration ─────────────────────────────────────────

/**
 * Maps i18n language codes to their corresponding locale and currency.
 * Extend this record when adding new languages — single source of truth.
 */
const LOCALE_CURRENCY_MAP: Partial<Record<string, { locale: string; currency: string }>> = {
  vi: { locale: "vi-VN", currency: "VND" },
  "vi-VN": { locale: "vi-VN", currency: "VND" },
  zh: { locale: "zh-CN", currency: "CNY" },
  "zh-CN": { locale: "zh-CN", currency: "CNY" },
  en: { locale: "en-US", currency: "USD" },
  "en-US": { locale: "en-US", currency: "USD" },
};

const DEFAULT_LOCALE = { locale: "vi-VN", currency: "VND" };

// ── Hook ───────────────────────────────────────────────────────────

/**
 * Returns a memoized `formatCurrency` function that formats numbers
 * according to the current i18n language's locale and currency.
 *
 * @example
 * const { formatCurrency } = useCurrency();
 * formatCurrency(1500000); // "₫1.500.000" (when lang=vi)
 */
export function useCurrency() {
  const { i18n } = useTranslation();

  const formatCurrency = useCallback(
    (value: string | number): string => {
      const rawLang = i18n.language || "vi";
      const baseLang = rawLang.split("-")[0].toLowerCase();
      const { locale, currency } =
        LOCALE_CURRENCY_MAP[rawLang] ??
        LOCALE_CURRENCY_MAP[baseLang] ??
        DEFAULT_LOCALE;
      return new Intl.NumberFormat(locale, {
        style: "currency",
        currency,
        maximumFractionDigits: 0,
      }).format(Number(value));
    },
    [i18n.language],
  );

  return { formatCurrency };
}
