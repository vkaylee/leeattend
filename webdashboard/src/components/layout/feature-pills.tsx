import { useTranslation } from "react-i18next";

import { FEATURE_PILLS } from "./auth-mock-data";

export function FeaturePills() {
  const { t } = useTranslation();
  return (
    <div className="flex flex-wrap gap-2">
      {FEATURE_PILLS.map(({ icon, text }, i) => (
        <span
          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[11px] font-semibold text-white/60 border border-white/10 bg-white/5"
          key={text}
          style={{
            animation: "auth-fade-in 0.5s ease-out both",
            animationDelay: `${600 + i * 80}ms`,
          }}
        >
          <span className="text-primary-300 text-[8px]">{icon}</span>
          {t(text)}
        </span>
      ))}
    </div>
  );
}
