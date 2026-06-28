import {
  Shield,
  Globe,
  Check,
} from "lucide-react";
import { type ReactNode, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

import { VietnameseFlag, EnglishFlag, ChineseFlag } from "./icons/flags";

// ---------------------------------------------------------------------------
// Pre-auth Language Switcher — no API sync (safe before login)
// ---------------------------------------------------------------------------

type AuthLang = "vi" | "en" | "zh";

const AUTH_LANGUAGES: { code: AuthLang; nativeName: string; Flag: React.ComponentType<{ className?: string }> }[] = [
  { code: "vi", nativeName: "Tiếng Việt", Flag: VietnameseFlag },
  { code: "en", nativeName: "English",    Flag: EnglishFlag },
  { code: "zh", nativeName: "中文",        Flag: ChineseFlag },
];

function normalizeAuthLang(lang: string): AuthLang {
  const supported: AuthLang[] = ["vi", "en", "zh"];
  const base = lang.split("-")[0] as AuthLang;
  return supported.includes(base) ? base : "vi";
}

function AuthLanguageSwitcher() {
  const { i18n } = useTranslation();
  const [open, setOpen] = useState(false);
  const current = normalizeAuthLang(i18n.language);
  const currentDef = AUTH_LANGUAGES.find((l) => l.code === current) ?? AUTH_LANGUAGES[0];

  const handleSelect = useCallback(
    async (code: AuthLang) => {
      // Change i18n locale immediately + persist to localStorage
      await i18n.changeLanguage(code);
      localStorage.setItem("i18nextLng", code);
      setOpen(false);
    },
    [i18n]
  );

  return (
    <div className="relative">
      <button
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label="Switch language"
        className={cn(
          "flex items-center gap-2 rounded-xl px-3 py-2 text-sm font-semibold min-h-[40px] cursor-pointer",
          "bg-slate-100/80 dark:bg-slate-800/60 border border-slate-200/60 dark:border-slate-700/50",
          "text-slate-600 dark:text-slate-300 hover:text-primary-600 dark:hover:text-primary-400",
          "hover:bg-white dark:hover:bg-slate-800 hover:border-primary-200 dark:hover:border-primary-700",
          "transition-all duration-200 active:scale-95",
          "focus:outline-none focus:ring-2 focus:ring-primary-500/20"
        )}
        data-testid="auth-language-switcher"
        onClick={() => { setOpen((v) => !v); }}
        type="button"
      >
        <currentDef.Flag className="w-4 h-4" />
        <span className="hidden sm:inline">{currentDef.nativeName}</span>
        <Globe className="sm:hidden w-4 h-4 text-slate-400" />
      </button>

      {open ? (
        <>
          {/* Backdrop to close on outside click */}
          <div
            aria-hidden="true"
            className="fixed inset-0 z-[9998]"
            onClick={() => { setOpen(false); }}
          />
          <div
            className={cn(
              "absolute right-0 top-full mt-2 z-[9999] min-w-[180px] rounded-2xl p-1.5",
              "bg-white/95 dark:bg-slate-900/95 backdrop-blur-xl",
              "border border-slate-200/80 dark:border-slate-800/80 shadow-2xl",
              "animate-in fade-in-0 zoom-in-95 slide-in-from-top-2 duration-200"
            )}
            role="listbox"
          >
            {AUTH_LANGUAGES.map(({ code, nativeName, Flag }) => {
              const isSelected = current === code;
              return (
                <button
                  aria-selected={isSelected}
                  className={cn(
                    "flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-sm min-h-[44px] cursor-pointer transition-all duration-150",
                    isSelected
                      ? "bg-primary-50 dark:bg-primary-950/40 text-primary-600 dark:text-primary-400 font-bold"
                      : "text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800/60"
                  )}
                  data-testid={`auth-lang-${code}`}
                  key={code}
                  onClick={() => { void handleSelect(code); }}
                  role="option"
                  type="button"
                >
                  <Flag className="w-4 h-4" />
                  <span className="flex-1 text-left">{nativeName}</span>
                  {isSelected ? <Check className="text-primary-500" size={14} strokeWidth={2.5} /> : null}
                </button>
              );
            })}
          </div>
        </>
      ) : null}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Dashboard Mockup — premium visual showcase
// ---------------------------------------------------------------------------

const ATTENDANCE_BARS = [72, 88, 65, 92, 78, 95, 83];
const DAY_LABELS = ["M", "T", "W", "T", "F", "S", "S"];

const MOCK_EMPLOYEES: {
  name: string;
  role: string;
  status: "present" | "late" | "leave";
  time: string;
  initials: string;
  hue: string;
}[] = [
  { name: "Nguyen Van A", role: "Engineering",  status: "present", time: "08:02", initials: "NA", hue: "150" },
  { name: "Tran Thi B",  role: "Marketing",   status: "late",    time: "09:14", initials: "TB", hue: "220" },
  { name: "Le Van C",   role: "HR",           status: "leave",   time: "--:--", initials: "LC", hue: "280" },
  { name: "Pham Thi D", role: "Finance",      status: "present", time: "07:58", initials: "PD", hue: "35"  },
];

const STATUS_STYLE = {
  present: { dot: "bg-emerald-400", badge: "text-emerald-300 bg-emerald-400/15", label: "Present" },
  late:    { dot: "bg-amber-400",   badge: "text-amber-300   bg-amber-400/15",   label: "Late"    },
  leave:   { dot: "bg-slate-400",   badge: "text-slate-300   bg-slate-400/15",   label: "Leave"   },
} as const;

const STAT_CARDS = [
  { value: "247", label: "Employees",    color: "from-emerald-400/25 to-teal-400/10",    border: "border-emerald-400/20" },
  { value: "94%", label: "On Time",      color: "from-blue-400/25    to-indigo-400/10",   border: "border-blue-400/20"    },
  { value: "12",  label: "On Leave",     color: "from-violet-400/25  to-purple-400/10",  border: "border-violet-400/20"  },
] as const;

function DashboardMockup() {
  return (
    <div
      aria-hidden="true"
      className="w-full max-w-[400px] mx-auto"
    >
      {/* App window chrome */}
      <div className="rounded-2xl overflow-hidden shadow-2xl border border-white/10" style={{ background: "rgba(255,255,255,0.06)", backdropFilter: "blur(12px)" }}>

        {/* Title bar */}
        <div className="flex items-center gap-2 px-4 py-3 border-b border-white/8" style={{ background: "rgba(255,255,255,0.04)" }}>
          <div className="flex gap-1.5">
            <div className="w-2.5 h-2.5 rounded-full bg-rose-400/60" />
            <div className="w-2.5 h-2.5 rounded-full bg-amber-400/60" />
            <div className="w-2.5 h-2.5 rounded-full bg-emerald-400/60" />
          </div>
          <span className="text-white/40 text-[11px] font-medium ml-2 tracking-wide">LeeAttend — Attendance Dashboard</span>
        </div>

        <div className="p-4 space-y-4">

          {/* Stat cards row */}
          <div className="grid grid-cols-3 gap-2">
            {STAT_CARDS.map(({ value, label, color, border }) => (
              <div
                className={`rounded-xl p-3 bg-gradient-to-br ${color} border ${border}`}
                key={label}
                style={{ animation: "auth-fade-in 0.5s ease-out both" }}
              >
                <p className="text-white font-bold text-lg leading-none">{value}</p>
                <p className="text-white/50 text-[10px] font-medium mt-1 uppercase tracking-wide">{label}</p>
              </div>
            ))}
          </div>

          {/* Attendance chart */}
          <div className="rounded-xl p-3" style={{ background: "rgba(255,255,255,0.04)" }}>
            <p className="text-white/50 text-[10px] font-semibold uppercase tracking-widest mb-3">Weekly Attendance</p>
            <div className="flex items-end gap-1.5 h-14">
              {ATTENDANCE_BARS.map((h, i) => (
                <div className="flex-1 flex flex-col items-center gap-1" key={i}>
                  <div
                    className="w-full rounded-t-sm origin-bottom"
                    style={{
                      height: `${h}%`,
                      background: i === 4
                        ? "linear-gradient(to top, rgba(52,211,153,0.9), rgba(16,185,129,0.5))"
                        : "rgba(255,255,255,0.15)",
                      animation: "auth-bar-grow 0.5s cubic-bezier(0.34,1.4,0.64,1) both",
                      animationDelay: `${i * 60}ms`,
                    }}
                  />
                  <span className="text-white/25 text-[8px] font-medium">{DAY_LABELS[i]}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Employee list */}
          <div className="space-y-1.5">
            <p className="text-white/50 text-[10px] font-semibold uppercase tracking-widest mb-2">Today&apos;s Status</p>
            {MOCK_EMPLOYEES.map(({ name, role, status, time, initials, hue }, idx) => {
              const s = STATUS_STYLE[status];
              return (
                <div
                  className="flex items-center gap-3 rounded-xl px-3 py-2 transition-colors"
                  key={name}
                  style={{
                    background: "rgba(255,255,255,0.04)",
                    animation: "auth-fade-in 0.4s ease-out both",
                    animationDelay: `${200 + idx * 80}ms`,
                  }}
                >
                  {/* Avatar */}
                  <div
                    className="w-7 h-7 rounded-lg flex items-center justify-center text-white text-[10px] font-bold flex-shrink-0"
                    style={{ background: `hsl(${hue} 60% 45% / 0.6)` }}
                  >
                    {initials}
                  </div>
                  {/* Info */}
                  <div className="flex-1 min-w-0">
                    <p className="text-white/80 text-xs font-semibold truncate leading-none">{name}</p>
                    <p className="text-white/35 text-[10px] truncate mt-0.5">{role}</p>
                  </div>
                  {/* Status */}
                  <div className={`flex items-center gap-1.5 rounded-full px-2 py-0.5 text-[9px] font-bold uppercase tracking-wide ${s.badge}`}>
                    <div className={`w-1.5 h-1.5 rounded-full ${s.dot} ${status === "present" ? "animate-pulse" : ""}`} />
                    {time}
                  </div>
                </div>
              );
            })}
          </div>

        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Feature pill list — compact, text-forward
// ---------------------------------------------------------------------------

const FEATURE_PILLS = [
  { icon: "✦", text: "auth.brand.feature_1_title" },
  { icon: "✦", text: "auth.brand.feature_2_title" },
  { icon: "✦", text: "auth.brand.feature_3_title" },
  { icon: "✦", text: "auth.brand.feature_4_title" },
] as const;

function FeaturePills() {
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

// ---------------------------------------------------------------------------
// Dot grid SVG background pattern
// ---------------------------------------------------------------------------
const DOT_GRID_BG = `url("data:image/svg+xml,%3Csvg width='28' height='28' viewBox='0 0 28 28' xmlns='http://www.w3.org/2000/svg'%3E%3Ccircle cx='1.5' cy='1.5' r='1.5' fill='rgba(255,255,255,0.05)'/%3E%3C/svg%3E")`;

// ---------------------------------------------------------------------------
// Brand Panel (right side)
// ---------------------------------------------------------------------------

function BrandPanel() {
  const { t } = useTranslation();

  return (
    <aside
      aria-label="LeeAttend brand panel"
      className="hidden lg:flex flex-col relative overflow-hidden"
      style={{
        backgroundImage: `${DOT_GRID_BG}, linear-gradient(145deg, var(--primary-950) 0%, var(--primary-900) 50%, hsl(from var(--primary-800) h s calc(l - 5)) 100%)`,
      }}
    >
      {/* Decorative blobs — subtler, offset */}
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
      {/* Accent glow near mockup */}
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
          <h2
            className="text-white font-bold leading-tight tracking-tight"
            style={{
              fontSize: "clamp(1.6rem, 2.2vw, 2.4rem)",
              animation: "auth-fade-in 0.6s ease-out both",
            }}
          >
            {t("auth.brand.headline")}
          </h2>
          <p
            className="text-white/40 text-xs font-semibold uppercase tracking-[0.25em] mt-2"
            style={{ animation: "auth-fade-in 0.6s ease-out 0.1s both" }}
          >
            {t("auth.brand.subline")}
          </p>
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

// ---------------------------------------------------------------------------
// Auth Layout — exported public API
// ---------------------------------------------------------------------------

interface AuthLayoutProps {
  /** Form content to render in the left panel */
  children: ReactNode;
}

export function AuthLayout({ children }: AuthLayoutProps) {
  return (
    <>
      {/* Keyframe animations — injected once */}
      <style>{`
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
      `}</style>


      <div className="min-h-screen grid lg:grid-cols-[40%_60%] bg-slate-50 dark:bg-slate-950">
        {/* Left: Form panel */}
        <main className="flex items-center justify-center p-6 sm:p-10 relative overflow-hidden">
          {/* Subtle background glow — mobile only */}
          <div
            aria-hidden="true"
            className="absolute top-[-15%] left-[-15%] w-[60%] h-[60%] rounded-full blur-[120px] opacity-40 lg:hidden"
            style={{ background: "var(--primary-100)" }}
          />
          <div
            aria-hidden="true"
            className="absolute bottom-[-15%] right-[-15%] w-[60%] h-[60%] rounded-full blur-[100px] opacity-30 lg:hidden"
            style={{ background: "color-mix(in oklch, var(--primary-100) 40%, transparent)" }}
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
