import { Shield } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { DashboardMockup } from "./dashboard-mockup";
import { FeaturePills } from "./feature-pills";

const DOT_GRID_BG = `url("data:image/svg+xml,%3Csvg width='28' height='28' viewBox='0 0 28 28' xmlns='http://www.w3.org/2000/svg'%3E%3Ccircle cx='1.5' cy='1.5' r='1.5' fill='rgba(255,255,255,0.05)'/%3E%3C/svg%3E")`;

export function BrandPanel() {
  const { t } = useTranslation();

  return (
    <aside
      aria-label="LeeAttend brand panel"
      className="hidden lg:flex flex-col relative overflow-hidden"
      style={{
        backgroundImage: `${DOT_GRID_BG}, linear-gradient(145deg, var(--primary-950) 0%, var(--primary-900) 50%, hsl(from var(--primary-800) h s calc(l - 5)) 100%)`,
      }}
    >
      {/* Decorative blobs */}
      <div
        aria-hidden="true"
        className="absolute top-[-15%] right-[-25%] w-[70%] h-[55%] rounded-full blur-[140px] pointer-events-none"
        style={{ background: "var(--primary-600)", opacity: 0.18 }}
      />
      <div
        aria-hidden="true"
        className="absolute bottom-[-20%] left-[-15%] w-[55%] h-[55%] rounded-full blur-[120px] pointer-events-none"
        style={{ background: "var(--primary-500)", opacity: 0.12 }}
      />
      <div
        aria-hidden="true"
        className="absolute top-[40%] left-[10%] w-[40%] h-[30%] rounded-full blur-[80px] pointer-events-none"
        style={{ background: "var(--primary-400)", opacity: 0.08 }}
      />

      {/* Content */}
      <div className="relative z-10 flex flex-col h-full px-10 py-12 gap-8">

        {/* Logo */}
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-white/12 border border-white/18 flex items-center justify-center shadow-lg">
            <span className="text-white text-lg font-bold italic tracking-tighter pl-0.5">L</span>
          </div>
          <span className="text-white/75 font-semibold text-sm tracking-widest uppercase">LeeAttend</span>
        </div>

        {/* Headline */}
        <div>
          <Typography
            className="text-white font-bold leading-tight tracking-tight"
            style={{
              fontSize: "clamp(1.6rem, 2.2vw, 2.4rem)",
              animation: "auth-fade-in 0.6s ease-out both",
            }}
            variant="h2"
          >
            {t("auth.brand.headline")}
          </Typography>
          <Typography
            className="text-white/40 text-xs font-semibold uppercase tracking-[0.25em] mt-2"
            style={{ animation: "auth-fade-in 0.6s ease-out 0.1s both" }}
            variant="p"
          >
            {t("auth.brand.subline")}
          </Typography>
        </div>

        {/* Dashboard mockup */}
        <div
          className="flex-1 flex items-center"
          style={{ animation: "auth-fade-in 0.7s ease-out 0.2s both" }}
        >
          <DashboardMockup />
        </div>

        {/* Feature pills */}
        <FeaturePills />

        {/* Trust badge */}
        <div className="flex items-center gap-2">
          <Shield className="w-3.5 h-3.5 text-white/30" strokeWidth={2} />
          <span className="text-white/28 text-[10px] font-medium tracking-wider uppercase">
            WCAG 2.1 AA · Enterprise-Grade Security
          </span>
        </div>

      </div>
    </aside>
  );
}
