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
      setCurrentWorkspaceId(ws.id);
      navigate("/", { replace: true });
    } catch (error) {
      console.error("Failed to create workspace:", error);
    } finally { setIsSubmitting(false); }
  };

  return (
    <div className="min-h-screen bg-slate-50 flex flex-col items-center justify-center p-6 animate-in fade-in duration-1000">
      <div className="w-full max-w-md bg-white rounded-[2.5rem] shadow-2xl shadow-slate-200 border border-slate-100 p-10 animate-in zoom-in-95 duration-700">
        <div className="flex flex-col items-center gap-6 mb-10">
          <div className="bg-primary-600 p-4 rounded-3xl shadow-xl shadow-primary-100">
            <LayoutDashboard className="text-white w-10 h-10" />
          </div>
          <div className="text-center space-y-2">
            <h1 className="text-3xl font-black text-slate-900 tracking-tight leading-tight uppercase">
              {t("onboarding.welcome_title")}
            </h1>
            <p className="text-slate-500 font-medium px-4">
              {t("onboarding.welcome_subtitle")}
            </p>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="space-y-8">
          <div className="space-y-3">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">
              {t("onboarding.org_name_label")}
            </label>
            <input
              autoFocus
              type="text"
              placeholder={t("onboarding.org_name_placeholder")}
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-5 py-4 bg-slate-50 border border-slate-100 rounded-2xl text-sm focus:outline-none focus:ring-4 focus:ring-primary-500/10 focus:bg-white focus:border-primary-200 transition-all font-bold tracking-tight"
              disabled={isSubmitting}
            />
          </div>

          <button
            type="submit"
            disabled={isSubmitting || !name.trim()}
            className="w-full py-4 bg-primary-600 text-white rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-primary-700 transition-all flex items-center justify-center gap-3 disabled:opacity-50 shadow-xl shadow-primary-100 active:scale-95"
          >
            {isSubmitting ? (
              t("common.creating").toUpperCase()
            ) : (
              <>
                <Plus className="w-5 h-5" />
                {t("onboarding.start_now_btn")}
              </>
            )}
          </button>
        </form>

        <div className="mt-10 p-6 bg-slate-50 rounded-[2rem] border border-slate-100">
          <p className="text-[10px] text-slate-400 text-center uppercase font-black tracking-widest leading-relaxed opacity-70">
            {t("onboarding.workspace_desc")}
          </p>
        </div>
      </div>
    </div>
  );
}
