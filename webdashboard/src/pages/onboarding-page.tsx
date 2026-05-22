import { useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { LayoutDashboard, Plus, LogOut, Database } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { LABEL_BASE } from "@/components/ui/form-field";
import { useWorkspaces, workspaceKeys } from "@/hooks/use-workspace";
import { setCurrentWorkspaceId } from "@/lib/api";
import { ROUTES } from "@/lib/routes";


export default function OnboardingPage() {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [databaseUrl, setDatabaseUrl] = useState("");
  const { createWorkspace } = useWorkspaces();
  const navigate = useNavigate();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const queryClient = useQueryClient();

  const handleSubmit = async (e: React.SyntheticEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    setIsSubmitting(true);
    try {
      const ws = await createWorkspace({ name, database_url: databaseUrl });
      setCurrentWorkspaceId(ws.id);
      
      // Update cache immediately to prevent race condition redirect back to onboarding
      queryClient.setQueryData(workspaceKeys.all, [ws]);
      
      await navigate({ to: ROUTES.HOME, replace: true });
    } catch (error) {
      console.error(error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleLogout = () => {
    localStorage.clear();
    window.location.href = ROUTES.LOGIN;
  };

  return (
    <div className="min-h-screen bg-slate-50 dark:bg-slate-950 flex flex-col items-center justify-center p-6 relative overflow-hidden animate-in fade-in duration-1000">
      {/* Top Right Actions */}
      <div className="absolute top-6 right-6 z-50">
        <button
          className="flex items-center gap-2 px-4 py-2 rounded-full text-sm font-bold tracking-widest uppercase text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-800/50 transition-all duration-300"
          onClick={handleLogout}
        >
          {t("common.logout")}
          <LogOut size={16} />
        </button>
      </div>
      <div className="absolute top-[-10%] left-[-10%] w-[50%] h-[50%] bg-primary-100 dark:bg-primary-900/30 rounded-full blur-[140px] animate-pulse duration-[12s]" />
      <div className="absolute bottom-[-10%] right-[-10%] w-[50%] h-[50%] bg-blue-100/30 dark:bg-blue-900/20 rounded-full blur-[140px] animate-pulse duration-[10s]" />

      <div className="w-full max-w-lg bg-white dark:bg-slate-900 rounded-[3.5rem] shadow-premium-2xl border border-white/60 p-12 animate-in zoom-in-95 duration-1000 relative z-10">
        <div className="flex flex-col items-center gap-8 mb-12">
          <div className="bg-slate-900 w-24 h-24 rounded-[2.5rem] flex items-center justify-center shadow-2xl shadow-slate-400 dark:shadow-black/40 rotate-3 transform hover:rotate-0 transition-all duration-700 group">
            <LayoutDashboard className="text-white group-hover:scale-110 transition-transform" size={40} />
          </div>
          <div className="text-center space-y-3">
            <h1 className="text-4xl font-bold text-slate-700 dark:text-slate-200 tracking-tighter leading-none uppercase">
              {t("onboarding.welcome_title")}
            </h1>
            <p className="text-slate-500 dark:text-slate-400 font-bold uppercase tracking-[0.25em] text-[10px] opacity-60 leading-relaxed max-w-xs mx-auto">
              {t("onboarding.welcome_subtitle")}
            </p>
          </div>
        </div>

        <form className="space-y-8" onSubmit={handleSubmit}>
          <div className="space-y-3 px-1">
            <label className={LABEL_BASE}>
              {t("onboarding.org_name_label")}
            </label>
            <input
 
              className="w-full px-6 py-4.5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[1.5rem] text-sm focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 focus:border-primary-200 dark:border-primary-800 transition-all font-bold tracking-tight shadow-inner"
              disabled={isSubmitting}
              onChange={(e) => { setName(e.target.value); }}
              placeholder={t("onboarding.org_name_placeholder")}
              type="text"
              value={name}
            />
          </div>

          <div className="space-y-3 px-1">
            <label className={LABEL_BASE + " flex items-center gap-2"}>
              <Database className="text-slate-400" size={12} />
              {t("common.database_url_label")}
            </label>
            <input
              className="w-full px-6 py-4.5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[1.5rem] text-xs focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 focus:border-primary-200 dark:border-primary-800 transition-all font-mono tracking-tight shadow-inner"
              disabled={isSubmitting}
              id="onboarding-database-url"
              onChange={(e) => { setDatabaseUrl(e.target.value); }}
              placeholder={t("common.database_url_placeholder")}
              type="url"
              value={databaseUrl}
            />
            <p className="text-[9px] text-slate-400 dark:text-slate-500 ml-1 tracking-wide">
              {t("common.database_url_hint")}
            </p>
          </div>

          <Button
            className="w-full"
            disabled={isSubmitting || !name.trim() || !databaseUrl.trim()}
            icon={isSubmitting ? undefined : <Plus size={18} />}
            loading={isSubmitting}
            size="lg"
            type="submit"
            variant="secondary"
          >
            {t("onboarding.start_now_btn").toUpperCase()}
          </Button>
        </form>

        <div className="mt-12 p-8 bg-slate-50 dark:bg-slate-800/50 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 shadow-inner">
          <p className="text-[10px] text-slate-500 dark:text-slate-400 text-center uppercase font-bold tracking-[0.2em] leading-relaxed opacity-60 italic">
            {t("onboarding.workspace_desc")}
          </p>
        </div>
      </div>
    </div>
  );
}
