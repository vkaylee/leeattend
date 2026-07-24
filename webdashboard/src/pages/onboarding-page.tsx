import { useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { LayoutDashboard, Plus } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import { Typography } from "@/components/ui/typography";
import { useWorkspaces, workspaceKeys } from "@/hooks/use-workspace";
import { setCurrentWorkspaceId } from "@/lib/api";
import { ROUTES } from "@/lib/routes";
import { useOnboardingProgress } from "@/pages/hooks/use-onboarding-progress";

import { DatabaseConfig } from "./onboarding/database-config";
import { OnboardingHeader } from "./onboarding/onboarding-header";
import { getValidationError } from "./onboarding/onboarding-utils";
import { ProgressOverlay } from "./onboarding/progress-overlay";

export default function OnboardingPage() {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [dbMode, setDbMode] = useState<"auto" | "byod">("auto");
  const [databaseUrl, setDatabaseUrl] = useState("");
  const [error, setError] = useState<string | null>(null);

  const {
    showProgress,
    setShowProgress,
    progressPercent,
    setProgressPercent,
    progressStep,
    setProgressStep,
    startProgressInterval,
    clearProgressInterval,
  } = useOnboardingProgress();

  const { createWorkspace } = useWorkspaces();
  const navigate = useNavigate();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const queryClient = useQueryClient();

  const validationError = getValidationError(dbMode, databaseUrl, t);

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

      clearProgressInterval();

      setProgressPercent(100);
      setProgressStep(5);

      await new Promise((resolve) => setTimeout(resolve, 800));

      setCurrentWorkspaceId(ws.id);
      queryClient.setQueryData(workspaceKeys.all, [ws]);

      await navigate({ to: ROUTES.HOME, replace: true });
    } catch (error_: unknown) {
      clearProgressInterval();
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
