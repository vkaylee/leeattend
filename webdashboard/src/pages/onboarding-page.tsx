import { useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { LayoutDashboard, Plus, LogOut, Database, Check, Loader2, DatabaseZap } from "lucide-react";
import React, { useState, useRef } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Typography } from "@/components/ui/typography";
import { useWorkspaces, workspaceKeys } from "@/hooks/use-workspace";
import { setCurrentWorkspaceId } from "@/lib/api";
import { ROUTES } from "@/lib/routes";

interface ProgressStepProps {
  label: string;
  progressStep: number;
  stepNum: number;
}

function ProgressStep({ label, progressStep, stepNum }: ProgressStepProps) {
  const status: "completed" | "active" | "pending" =
    progressStep === 5 || progressStep > stepNum
      ? "completed"
      : progressStep === stepNum
      ? "active"
      : "pending";

  const badgeContents = {
    completed: <Check size={12} />,
    active: <Loader2 className="animate-spin" size={12} />,
    pending: stepNum,
  };

  const badgeColors = {
    completed: "bg-emerald-500 text-white",
    active: "bg-primary-500 text-white animate-pulse",
    pending: "bg-slate-100 dark:bg-slate-800 text-slate-400 dark:text-slate-600",
  };

  const textColors = {
    completed: "text-slate-800 dark:text-slate-200",
    active: "text-primary-600 dark:text-primary-400 font-bold",
    pending: "text-slate-500 dark:text-slate-400",
  };

  const opacityClass = status === "pending" ? "opacity-40" : "opacity-100";

  return (
    <div className={`flex items-center gap-3 transition-opacity duration-300 ${opacityClass}`}>
      <div className={`w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold transition-all ${badgeColors[status]}`}>
        {badgeContents[status]}
      </div>
      <span className={`text-xs font-semibold ${textColors[status]}`}>
        {label}
      </span>
    </div>
  );
}

interface ProgressOverlayProps {
  dbMode: "auto" | "byod";
  progressPercent: number;
  progressStep: number;
  showProgress: boolean;
  t: (key: string) => string;
}

function ProgressOverlay({
  dbMode,
  progressPercent,
  progressStep,
  showProgress,
  t,
}: ProgressOverlayProps) {
  if (!showProgress) return null;

  return (
    <div className="fixed inset-0 bg-slate-950/90 dark:bg-slate-950/95 backdrop-blur-md flex items-center justify-center z-50 animate-in fade-in duration-500">
      <div className="w-full max-w-md p-8 bg-white dark:bg-slate-900 rounded-[2.5rem] shadow-premium-2xl border border-white/10 dark:border-slate-800/50 text-center space-y-8 animate-in zoom-in-95 duration-500 relative overflow-hidden">
        {/* Top glowing background blob inside loader card */}
        <div className="absolute top-[-20%] left-[-20%] w-[140%] h-[140%] bg-primary-500/10 rounded-full blur-[80px] pointer-events-none" />

        <div className="flex flex-col items-center gap-4 relative z-10">
          <div className="relative bg-slate-100 dark:bg-slate-800 w-16 h-16 rounded-2xl flex items-center justify-center shadow-inner mb-2 animate-bounce">
            {progressStep === 5 ? (
              <Check className="text-emerald-500" size={32} />
            ) : (
              <DatabaseZap className="text-primary-500 dark:text-primary-400 animate-pulse" size={28} />
            )}
          </div>
          <Typography className="text-2xl font-bold text-slate-800 dark:text-slate-100 tracking-tight" variant="h2">
            {t("onboarding.progress.title")}
          </Typography>
          <Typography className="text-xs text-slate-500 dark:text-slate-400 max-w-xs leading-relaxed" variant="p">
            {t("onboarding.progress.subtitle")}
          </Typography>
        </div>

        {/* Circular Progress Bar Indicator */}
        <div className="relative w-full h-2.5 bg-slate-100 dark:bg-slate-800 rounded-full overflow-hidden z-10 shadow-inner">
          <div
            className="h-full bg-gradient-to-r from-primary-500 to-blue-500 rounded-full transition-all duration-300 shadow-lg"
            style={{ width: `${progressPercent}%` }}
          />
        </div>
        
        <div className="flex justify-between items-center text-[10px] font-bold text-slate-400 dark:text-slate-500 px-1 relative z-10">
          <span>{dbMode === "auto" ? "AUTO-PROVISION" : "BYOD DATABASE"}</span>
          <span className="text-primary-500 dark:text-primary-400">{Math.round(progressPercent)}%</span>
        </div>

        {/* Steps Timeline Grid */}
        <div className="text-left space-y-4 pt-2 border-t border-slate-100 dark:border-slate-800 relative z-10">
          {[
            { label: t("onboarding.progress.step_1"), stepNum: 1 },
            { label: t("onboarding.progress.step_2"), stepNum: 2 },
            { label: t("onboarding.progress.step_3"), stepNum: 3 },
            { label: t("onboarding.progress.step_4"), stepNum: 4 },
          ].map((step) => (
            <ProgressStep
              key={step.stepNum}
              label={step.label}
              progressStep={progressStep}
              stepNum={step.stepNum}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

function getValidationError(dbMode: "auto" | "byod", databaseUrl: string, t: (key: string) => string): string | null {
  if (dbMode === "byod" && databaseUrl) {
    const isValid = /^postgres(ql)?:\/\/.+/i.test(databaseUrl.trim());
    return isValid ? null : t("onboarding.db_validation_error");
  }
  return null;
}

function clearIntervalRef(ref: { current: NodeJS.Timeout | null }) {
  if (ref.current !== null) {
    clearInterval(ref.current);
    ref.current = null;
  }
}

interface OnboardingHeaderProps {
  onLogout: () => void;
  t: (key: string) => string;
}

function OnboardingHeader({ onLogout, t }: OnboardingHeaderProps) {
  return (
    <>
      {/* Top Right Actions */}
      <div className="absolute top-6 right-6 z-40">
        <Button className="flex items-center gap-2 px-4 py-2 rounded-full text-xs font-bold tracking-widest uppercase text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-800/50 transition-all duration-300"
          onClick={onLogout}
          variant="ghost"
        >
          {t("common.logout")}
          <LogOut size={14} />
        </Button>
      </div>

      {/* Background Blobs */}
      <div className="absolute top-[-10%] left-[-10%] w-[50%] h-[50%] bg-primary-100 dark:bg-primary-900/20 rounded-full blur-[140px] animate-pulse duration-[12s]" />
      <div className="absolute bottom-[-10%] right-[-10%] w-[50%] h-[50%] bg-blue-100/30 dark:bg-blue-900/10 rounded-full blur-[140px] animate-pulse duration-[10s]" />
    </>
  );
}

interface DatabaseConfigProps {
  databaseUrl: string;
  dbMode: "auto" | "byod";
  isSubmitting: boolean;
  setDatabaseUrl: (url: string) => void;
  setDbMode: (mode: "auto" | "byod") => void;
  t: (key: string) => string;
  validationError: string | null;
}

function DatabaseConfig({
  databaseUrl,
  dbMode,
  isSubmitting,
  setDatabaseUrl,
  setDbMode,
  t,
  validationError,
}: DatabaseConfigProps) {
  return (
    <div className="space-y-2 px-1">
      <label className={LABEL_BASE}>
        {t("onboarding.db_mode_label")}
      </label>
      
      <Tabs onValueChange={(val) => { setDbMode(val as "auto" | "byod"); }} value={dbMode}>
        <TabsList className="grid grid-cols-2 p-1 bg-slate-100 dark:bg-slate-800/50 rounded-2xl border-none">
          <TabsTrigger
            className={`py-3 text-xs font-bold rounded-xl transition-all border-none ${
              dbMode === "auto"
                ? "bg-white dark:bg-slate-700 text-slate-800 dark:text-white shadow-sm"
                : "text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
            }`}
            value="auto"
          >
            {t("onboarding.db_mode_auto")}
          </TabsTrigger>
          <TabsTrigger
            className={`py-3 text-xs font-bold rounded-xl transition-all border-none ${
              dbMode === "byod"
                ? "bg-white dark:bg-slate-700 text-slate-800 dark:text-white shadow-sm"
                : "text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
            }`}
            value="byod"
          >
            {t("onboarding.db_mode_byod")}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="auto">
          <div className="p-4 bg-slate-50 dark:bg-slate-800/30 rounded-2xl border border-slate-100 dark:border-slate-800/50">
            <Typography className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed" variant="p">
              {t("onboarding.db_mode_auto_desc")}
            </Typography>
          </div>
        </TabsContent>

        <TabsContent value="byod">
          <div className="space-y-3 p-4 bg-slate-50 dark:bg-slate-800/30 rounded-2xl border border-slate-100 dark:border-slate-800/50">
            <Typography className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed mb-1" variant="p">
              {t("onboarding.db_mode_byod_desc")}
            </Typography>
            
            <div className="space-y-1">
              <div className="relative flex items-center">
                <Database className="absolute left-4 text-slate-400 dark:text-slate-500" size={14} />
                <input
                  className={`w-full pl-11 pr-6 py-3.5 bg-white dark:bg-slate-800 border ${
                    validationError
                      ? "border-rose-400 dark:border-rose-800 focus:ring-rose-500/10"
                      : "border-slate-200 dark:border-slate-700 focus:ring-primary-500/10"
                  } rounded-xl text-xs focus:outline-none focus:ring-4 focus:bg-white dark:focus:bg-slate-900 transition-all font-mono tracking-tight shadow-sm`}
                  disabled={isSubmitting}
                  onChange={(e) => { setDatabaseUrl(e.target.value); }}
                  placeholder={t("onboarding.db_placeholder")}
                  required={dbMode === "byod"}
                  type="text"
                  value={databaseUrl}
                />
              </div>
              {validationError ? (
                <Typography className="text-[10px] text-rose-500 font-bold ml-1 tracking-tight" variant="p">
                  {validationError}
                </Typography>
              ) : (
                <Typography className="text-[9px] text-slate-400 dark:text-slate-500 ml-1 tracking-wide" variant="p">
                  {t("common.database_url_hint")}
                </Typography>
              )}
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

 
export default function OnboardingPage() {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [dbMode, setDbMode] = useState<"auto" | "byod">("auto");
  const [databaseUrl, setDatabaseUrl] = useState("");
  const [error, setError] = useState<string | null>(null);

  // Progress screen states
  const [showProgress, setShowProgress] = useState(false);
  const [progressPercent, setProgressPercent] = useState(0);
  const [progressStep, setProgressStep] = useState(0);

  const { createWorkspace } = useWorkspaces();
  const navigate = useNavigate();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const queryClient = useQueryClient();

  const progressIntervalRef = useRef<NodeJS.Timeout | null>(null);

  const validationError = getValidationError(dbMode, databaseUrl, t);

  const startProgressInterval = () => {
    progressIntervalRef.current = setInterval(() => {
      setProgressPercent((prev) => {
        if (prev >= 98) {
          return 98;
        }
        const step = Math.random() * 4 + 1;
        const nextPercent = Math.min(prev + step, 98);

        const currentStep = nextPercent < 25 ? 1 : nextPercent < 60 ? 2 : nextPercent < 85 ? 3 : 4;
        setProgressStep(currentStep);

        return nextPercent;
      });
    }, 150);
  };

  const handleSubmit = async (e: React.SyntheticEvent) => {
    e.preventDefault();
    const isInvalid = !name.trim() || (dbMode === "byod" && (!databaseUrl.trim() || validationError));
    if (isInvalid) return;

    setError(null);
    setIsSubmitting(true);
    setShowProgress(true);
    setProgressPercent(5);
    setProgressStep(1);

    startProgressInterval();

    try {
      const ws = await createWorkspace({
        name,
        database_url: dbMode === "byod" ? databaseUrl.trim() : undefined,
      });

      clearIntervalRef(progressIntervalRef);

      setProgressPercent(100);
      setProgressStep(5);

      await new Promise((resolve) => setTimeout(resolve, 800));

      setCurrentWorkspaceId(ws.id);
      queryClient.setQueryData(workspaceKeys.all, [ws]);
      
      await navigate({ to: ROUTES.HOME, replace: true });
    } catch (error_: unknown) {
      clearIntervalRef(progressIntervalRef);
      setShowProgress(false);
      setIsSubmitting(false);

      const message = error_ instanceof Error ? error_.message : String(error_);
      setError(message || t("onboarding.messages.create_error"));
    }
  };

  const handleLogout = () => {
    localStorage.clear();
    window.location.href = ROUTES.LOGIN;
  };

  return (
    <div className="min-h-screen bg-slate-50 dark:bg-slate-950 flex flex-col items-center justify-center p-6 relative overflow-hidden animate-in fade-in duration-1000">
      <OnboardingHeader onLogout={handleLogout} t={t} />

      <div className="w-full max-w-lg bg-white dark:bg-slate-900 rounded-[3.5rem] shadow-premium-2xl border border-white/60 dark:border-slate-800/60 p-12 animate-in zoom-in-95 duration-1000 relative z-10">
        <div className="flex flex-col items-center gap-6 mb-10">
          <div className="bg-slate-900 w-20 h-20 rounded-[2.2rem] flex items-center justify-center shadow-2xl shadow-slate-400 dark:shadow-black/40 rotate-3 transform hover:rotate-0 transition-all duration-700 group">
            <LayoutDashboard className="text-white group-hover:scale-110 transition-transform" size={32} />
          </div>
          <div className="text-center space-y-2">
            <Typography className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter leading-none uppercase" variant="h1">
              {t("onboarding.welcome_title")}
            </Typography>
            <Typography className="text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.25em] text-[9px] opacity-60 leading-relaxed max-w-xs mx-auto" variant="p">
              {t("onboarding.welcome_subtitle")}
            </Typography>
          </div>
        </div>

        {error ? (
          <div className="mb-6 bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-800 text-rose-600 dark:text-rose-400 p-4 rounded-2xl text-xs font-semibold text-center animate-in slide-in-from-top-4" role="alert">
            {error}
          </div>
        ) : null}

        <form className="space-y-6" onSubmit={handleSubmit}>
          {/* Org Name Input */}
          <div className="space-y-2 px-1">
            <label className={LABEL_BASE}>
              {t("onboarding.org_name_label")}
            </label>
            <input
              className="w-full px-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl text-sm focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 focus:border-primary-200 dark:focus:border-primary-800 transition-all font-bold tracking-tight shadow-inner"
              disabled={isSubmitting}
              onChange={(e) => { setName(e.target.value); }}
              placeholder={t("onboarding.org_name_placeholder")}
              required
              type="text"
              value={name}
            />
          </div>

          <DatabaseConfig
            databaseUrl={databaseUrl}
            dbMode={dbMode}
            isSubmitting={isSubmitting}
            setDatabaseUrl={setDatabaseUrl}
            setDbMode={setDbMode}
            t={t}
            validationError={validationError}
          />

          <Button
            className="w-full h-14"
            disabled={
              isSubmitting ||
              !name.trim() ||
              (dbMode === "byod" && (!databaseUrl.trim() || !!validationError))
            }
            icon={isSubmitting ? undefined : <Plus size={18} />}
            loading={isSubmitting}
            size="lg"
            type="submit"
            variant="secondary"
          >
            {t("onboarding.start_now_btn").toUpperCase()}
          </Button>
        </form>

        <div className="mt-10 p-6 bg-slate-50 dark:bg-slate-800/30 rounded-[2rem] border border-slate-100 dark:border-slate-800/50 shadow-inner">
          <Typography className="text-[9px] text-slate-500 dark:text-slate-400 text-center uppercase font-bold tracking-[0.2em] leading-relaxed opacity-60 italic" variant="p">
            {t("onboarding.workspace_desc")}
          </Typography>
        </div>
      </div>

      <ProgressOverlay
        dbMode={dbMode}
        progressPercent={progressPercent}
        progressStep={progressStep}
        showProgress={showProgress}
        t={t}
      />
    </div>
  );
}
