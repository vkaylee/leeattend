import { Plus, LayoutDashboard, Loader2, AlertCircle, Database } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/modal";
import { useWorkspaces } from "@/hooks/use-workspace";


interface CreateWorkspaceDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function CreateWorkspaceDialog({
  isOpen,
  onClose,
}: CreateWorkspaceDialogProps) {
  const { t } = useTranslation();
  const { createWorkspace } = useWorkspaces();
  const [name, setName] = useState("");
  const [databaseUrl, setDatabaseUrl] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.SyntheticEvent) {
    e.preventDefault();
    if (!name.trim() || !databaseUrl.trim()) return;
    setIsSubmitting(true); setError(null);
    try {
      await createWorkspace({ name, database_url: databaseUrl });
      setName(""); setDatabaseUrl(""); onClose();
    } catch {
      setError(t("common.create_workspace_error"));
    } finally { setIsSubmitting(false); }
  }

  return (
    <Modal
      icon={<LayoutDashboard size={24} />}
      isOpen={isOpen}
      maxWidth="md"
      onClose={onClose}
      subtitle={t("common.leeattend_system")}
      title={t("common.create_workspace")}
    >
      <form className="space-y-6 pt-4" onSubmit={handleSubmit}>
        {!!error && (
          <div className="p-4 bg-rose-50 dark:bg-rose-950/30 border border-rose-100 dark:border-rose-800 rounded-2xl flex items-center gap-3 animate-in fade-in slide-in-from-top-2 duration-300">
            <AlertCircle className="text-rose-600 dark:text-rose-300" size={18} />
            <p className="text-[10px] font-bold text-rose-600 dark:text-rose-300 uppercase tracking-widest">{error}</p>
          </div>
        )}

        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="workspace-name">
            {t("common.org_name_label")}
          </label>
          <input className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" disabled={isSubmitting} id="workspace-name" onChange={(e) => { setName(e.target.value); }} placeholder={t("common.org_name_placeholder")} required type="text"
            value={name}
          />
        </div>

        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1 flex items-center gap-2" htmlFor="workspace-database-url">
            <Database className="text-slate-400" size={12} />
            {t("common.database_url_label")}
          </label>
          <input
            className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono text-xs tracking-tight"
            disabled={isSubmitting}
            id="workspace-database-url"
            onChange={(e) => { setDatabaseUrl(e.target.value); }}
            placeholder={t("common.database_url_placeholder")}
            required
            type="url"
            value={databaseUrl}
          />
          <p className="text-[9px] text-slate-400 dark:text-slate-500 ml-1 tracking-wide">
            {t("common.database_url_hint")}
          </p>
        </div>

        <div className="pt-2">
          <button
            className="w-full flex items-center justify-center gap-3 bg-slate-900 text-white py-4 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95 disabled:opacity-50"
            disabled={isSubmitting || !name.trim() || !databaseUrl.trim()}
            type="submit"
          >
            {isSubmitting ? (
              <Loader2 className="animate-spin" size={18} />
            ) : (
              <Plus size={18} />
            )}
            {t("common.create_workspace").toUpperCase()}
          </button>
        </div>
      </form>
    </Modal>
  );
}
