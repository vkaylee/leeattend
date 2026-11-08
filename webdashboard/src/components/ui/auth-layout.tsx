import { type ReactNode } from "react";

import { AuthLanguageSwitcher } from "../layout/auth-language-switcher";
import { BrandPanel } from "../layout/brand-panel";

// ── Auth Layout Animations ────────────────────────────────────────────

const AUTH_KEYFRAMES = `
  @keyframes auth-bar-grow {
    from { transform: scaleY(0); opacity: 0; }
    to   { transform: scaleY(1); opacity: 1; }
  }
  @keyframes auth-fade-in {
    from { opacity: 0; transform: translateY(4px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  @media (prefers-reduced-motion: reduce) {
    @keyframes auth-bar-grow { from { opacity:1; transform:scaleY(1); } to { opacity:1; transform:scaleY(1); } }
    @keyframes auth-fade-in  { from { opacity:1; } to { opacity:1; } }
  }
`;

// ── Auth Layout ──────────────────────────────────────────────────────

interface AuthLayoutProps {
  /** Form content to render in the left panel */
  children: ReactNode;
}

export function AuthLayout({ children }: AuthLayoutProps) {
  return (
    <>
      <style>{AUTH_KEYFRAMES}</style>

      <div className="min-h-screen grid lg:grid-cols-[40%_60%] bg-slate-50 dark:bg-slate-950">
        {/* Left: Form panel */}
        <main className="flex min-h-screen items-center justify-center overflow-y-auto p-6 sm:p-10 relative">
          {/* Subtle background glow — mobile only */}
          <div
            aria-hidden="true"
            className="absolute top-[-15%] left-[-15%] w-[60%] h-[60%] rounded-full blur-[120px] opacity-40 lg:hidden bg-[var(--primary-100)]"
          />
          <div
            aria-hidden="true"
            className="absolute bottom-[-15%] right-[-15%] w-[60%] h-[60%] rounded-full blur-[100px] opacity-30 lg:hidden bg-[color-mix(in_oklch,var(--primary-100)_40%,transparent)]"
          />

          {/* Language switcher — top-right corner, pre-auth safe */}
          <div className="absolute top-4 right-4 sm:top-6 sm:right-6 z-20">
            <AuthLanguageSwitcher />
          </div>

          {/* Form container */}
          <div
            className="relative z-10 w-full max-w-sm lg:max-w-md animate-in fade-in slide-in-from-left-4 duration-700"
          >
            {children}
          </div>
        </main>

        {/* Right: Brand panel */}
        <BrandPanel />
      </div>
    </>
  );
}
