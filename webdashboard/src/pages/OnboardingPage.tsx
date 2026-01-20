import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { LayoutDashboard, Plus } from "lucide-react";
import { useWorkspaces } from "@/hooks/use-workspace";
import { setCurrentWorkspaceId } from "@/lib/api";
import { useNavigate } from "react-router-dom";

export default function OnboardingPage() {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const { createWorkspace } = useWorkspaces();
  const navigate = useNavigate();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    setIsSubmitting(true);
    try {
      const ws = await createWorkspace({ name });
      setCurrentWorkspaceId(ws.id); navigate("/", { replace: true });
    } catch (error) {
    } finally { setIsSubmitting(false); }
  };

  return (
    <div className="min-h-screen bg-[#f8fafc] flex flex-col items-center justify-center p-6 relative overflow-hidden animate-in fade-in duration-1000">
      <div className="absolute top-[-10%] left-[-10%] w-[50%] h-[50%] bg-primary-100/30 rounded-full blur-[140px] animate-pulse duration-[12s]" />
      <div className="absolute bottom-[-10%] right-[-10%] w-[50%] h-[50%] bg-indigo-100/30 rounded-full blur-[140px] animate-pulse duration-[10s]" />

      <div className="w-full max-w-lg bg-white rounded-[3.5rem] shadow-premium-2xl border border-white/60 p-12 animate-in zoom-in-95 duration-1000 relative z-10">
        <div className="flex flex-col items-center gap-8 mb-12">
          <div className="bg-slate-900 w-24 h-24 rounded-[2.5rem] flex items-center justify-center shadow-2xl shadow-slate-400 rotate-3 transform hover:rotate-0 transition-all duration-700 group">
            <LayoutDashboard size={40} className="text-white group-hover:scale-110 transition-transform" />
          </div>
          <div className="text-center space-y-3">
            <h1 className="text-4xl font-black text-slate-900 tracking-tighter leading-none uppercase">
              {t("onboarding.welcome_title")}
            </h1>
            <p className="text-slate-400 font-black uppercase tracking-[0.25em] text-[10px] opacity-60 leading-relaxed max-w-xs mx-auto">
              {t("onboarding.welcome_subtitle")}
            </p>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="space-y-10">
          <div className="space-y-3 px-1">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1 opacity-70">
              {t("onboarding.org_name_label")}
            </label>
            <input
              autoFocus
              type="text"
              placeholder={t("onboarding.org_name_placeholder")}
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-6 py-4.5 bg-slate-50 border border-slate-100 rounded-[1.5rem] text-sm focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white focus:border-primary-200 transition-all font-black uppercase tracking-tight shadow-inner"
              disabled={isSubmitting}
            />
          </div>

          <button
            type="submit"
            disabled={isSubmitting || !name.trim()}
            className="w-full py-5 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-[0.2em] hover:bg-black transition-all shadow-2xl shadow-slate-400 active:scale-95 disabled:opacity-50 flex items-center justify-center gap-4"
          >
            {isSubmitting ? (
              <Loader2 size={18} className="animate-spin" />
            ) : (
              <Plus size={18} />
            )}
            {t("onboarding.start_now_btn").toUpperCase()}
          </button>
        </form>

        <div className="mt-12 p-8 bg-slate-50/50 rounded-[2.5rem] border border-slate-100 shadow-inner">
          <p className="text-[10px] text-slate-400 text-center uppercase font-bold tracking-[0.2em] leading-relaxed opacity-60 italic">
            {t("onboarding.workspace_desc")}
          </p>
        </div>
      </div>
    </div>
  );
}
