import { Mail, UserPlus, Loader2, AlertCircle } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Typography } from "@/components/ui/typography";
import { useWorkspaceMembers, useWorkspaceRoles } from "@/hooks/use-workspace";

import Modal from "../ui/modal";

interface InviteMemberDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function InviteMemberDialog({
  isOpen,
  onClose,
}: InviteMemberDialogProps) {
  const { t } = useTranslation();
  const { addMember, isAdding } = useWorkspaceMembers();
  const { roles } = useWorkspaceRoles();
  const [email, setEmail] = useState("");
  const [roleId, setRoleId] = useState("");
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.SyntheticEvent) {
    e.preventDefault(); setError(null);
    try {
      const selectedRoleId = roleId || roles[0]?.id;
      if (!selectedRoleId) throw new Error("No role selected");
      await addMember({ email, roleId: selectedRoleId });
      setEmail(""); setRoleId(""); onClose();
    } catch {
      setError(t("members.invite.error_message"));
    }
  }

  return (
    <Modal icon={<UserPlus size={24} />} isOpen={isOpen} maxWidth="md" onClose={onClose} subtitle={t("members.invite.subtitle")} title={t("members.invite.title")}>
      <form className="space-y-8 pt-4 animate-in fade-in duration-500" onSubmit={handleSubmit}>
        {!!error && (
          <div className="p-4 bg-rose-50 dark:bg-rose-950/30 border border-rose-100 dark:border-rose-800 rounded-2xl flex items-center gap-3 animate-in fade-in slide-in-from-top-2 duration-300">
            <AlertCircle className="text-rose-600 dark:text-rose-300" size={18} />
            <Typography className="text-[10px] font-bold text-rose-600 dark:text-rose-300 uppercase tracking-widest" variant="p">{error}</Typography>
          </div>
        )}

        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("members.invite.email_label")}</label>
          <div className="relative group">
            <div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors">
              <Mail size={18} />
            </div>
            <input className="w-full pl-14 pr-6 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" onChange={(e) => { setEmail(e.target.value); }} placeholder="dongnghiep@congty.com" required type="email" value={email} />
          </div>
        </div>

        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("members.invite.role_label")}</label>
          <select className="w-full px-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm uppercase tracking-tight shadow-sm appearance-none cursor-pointer" onChange={(e) => { setRoleId(e.target.value); }} value={roleId || (roles[0]?.id ?? "")}>
            {roles.map(r => (
              <option key={r.id} value={r.id}>{r.name}</option>
            ))}
          </select>
        </div>

        <div className="pt-2">
          <button className="w-full flex items-center justify-center gap-3 bg-slate-900 text-white py-4 rounded-2xl font-bold text-xs uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-slate-200 dark:shadow-black/30 active:scale-95 disabled:opacity-50" data-testid="submit-invite-member-btn" disabled={isAdding} type="submit">
            {isAdding ? <Loader2 className="animate-spin" size={18} /> : <UserPlus size={18} />}
            {t("members.invite.submit_btn").toUpperCase()}
          </button>
        </div>
      </form>
    </Modal>
  );
}
