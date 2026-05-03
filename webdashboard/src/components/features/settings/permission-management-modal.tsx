import { KeyRound, Plus, Trash2, Clock, Search } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import Modal from "@/components/ui/modal";
import { useConfirm } from "@/hooks/use-confirm";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useMemberPermissions } from "@/hooks/use-workspace";


interface PermissionManagementModalProps {
    isOpen: boolean;
    onClose: () => void;
    userId: string | null;
}

export default function PermissionManagementModal({
    userId,
    isOpen,
    onClose,
}: PermissionManagementModalProps) {
    const { t } = useTranslation();
    const { formatDateTime } = useDateTimeFormatter();
    const {
        permissions,
        available,
        isLoading,
        assignPermission,
        revokePermission,
    } = useMemberPermissions(userId ?? "", undefined);
    const [newPermission, setNewPermission] = useState("");
    const [expiresAt, setExpiresAt] = useState("");
    const [searchTerm, setSearchTerm] = useState("");
    const { confirm, ConfirmationDialog } = useConfirm();

    const filteredAvailable = available.filter(
        (p) =>
            p.toLowerCase().includes(searchTerm.toLowerCase()) &&
            !permissions.some((ep) => ep.permission === p),
    );

    async function handleAssign(e: React.SyntheticEvent) {
        e.preventDefault();
        if (!expiresAt || !newPermission || !userId) return;
        try {
            const isoExpiresAt = new Date(expiresAt).toISOString();
            await assignPermission({ permission: newPermission, expiresAt: isoExpiresAt });
            setExpiresAt("");
            setNewPermission("");
            setSearchTerm("");
        } catch (e) {
            console.error(e);
        }
    }

    async function handleRevoke(permId: string) {
        const ok = await confirm({
            title: t("common.warning", { defaultValue: "Warning" }),
            message: t("members.permissions.confirm_revoke", { defaultValue: "Revoke this permission?" }),
            variant: "danger"
        });
        if (!ok) return;
        try {
            await revokePermission(permId);
        } catch (e) {
            console.error(e);
        }
    }

    if (!isOpen) return null;

    return (
        <Modal
            icon={<KeyRound size={24} />}
            isOpen={isOpen}
            maxWidth="lg"
            onClose={onClose}
            subtitle={t("members.permissions.modal_subtitle", { defaultValue: "Grant or revoke individual permissions" })}
            title={t("members.permissions.modal_title", { defaultValue: "Manage Permissions" })}
        >
            <div className="space-y-10">
                    {isLoading ? (
                        <div className="py-20 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50 animate-pulse">{t("common.loading")}</div>
                    ) : (
                        <div className="space-y-10 animate-in fade-in duration-700">
                            {/* Active permissions list */}
                            <div className="space-y-5 px-1">
                                <label className="text-[10px] font-bold uppercase tracking-[0.25em] text-slate-500 dark:text-slate-400 ml-1">
                                    {t("members.permissions.active_label", { defaultValue: "Active Extra Permissions" }).toUpperCase()}
                                </label>

                                <div className="space-y-4 max-h-[300px] overflow-y-auto">
                                    {permissions.map((perm) => (
                                        <div className="flex items-center justify-between p-6 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-[2rem] hover:bg-white dark:hover:bg-slate-900 hover:shadow-xl hover:shadow-amber-100/10 transition-all group relative overflow-hidden shadow-inner" key={perm.id}>
                                            <div className="space-y-2 min-w-0 flex-1">
                                                <p className="font-bold text-slate-700 dark:text-slate-200 text-sm tracking-tight leading-none font-mono">{perm.permission}</p>
                                                <div className="flex items-center gap-2 text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest opacity-70">
                                                    <Clock className="text-amber-400" size={12} /> <span>{t("members.roles.expires_at", { defaultValue: "Expires" }).toUpperCase()}: {formatDateTime(perm.expires_at)}</span>
                                                </div>
                                            </div>
                                            <button className="p-3 bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all opacity-0 group-hover:opacity-100 translate-x-2 group-hover:translate-x-0 active:scale-90 flex-shrink-0 ml-4" onClick={() => handleRevoke(perm.id)}><Trash2 size={18} /></button>
                                        </div>
                                    ))}

                                    {permissions.length === 0 && (
                                        <div className="text-center py-12 border-2 border-dashed border-slate-100 dark:border-slate-800 rounded-[2.5rem] bg-slate-50 dark:bg-slate-800/30 flex flex-col items-center">
                                            <KeyRound className="text-slate-200 dark:text-slate-700 mb-4" size={32} />
                                            <p className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest opacity-60 italic">
                                                {t("members.permissions.no_extra_permissions", { defaultValue: "No extra permissions" })}
                                            </p>
                                        </div>
                                    )}
                                </div>
                            </div>
                        </div>
                    )}

                    {/* Assign new permission */}
                    <div className="pt-10 border-t border-slate-50 dark:border-slate-800 space-y-8">
                        <h3 className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] flex items-center gap-3 px-1">
                            <Plus className="text-amber-500" size={16} /> {t("members.permissions.assign_new_title", { defaultValue: "Grant Permission" }).toUpperCase()}
                        </h3>
                        <form className="space-y-6 animate-in slide-in-from-bottom-4 duration-700" onSubmit={handleAssign}>
                            <div className="space-y-3 px-1">
                                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">
                                    {t("members.permissions.permission_label", { defaultValue: "Permission" })}
                                </label>
                                <div className="relative">
                                    <Search className="absolute left-5 top-1/2 -translate-y-1/2 text-slate-400" size={14} />
                                    <input
                                        className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl pl-12 pr-6 py-4 focus:ring-4 focus:ring-amber-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm"
                                        onChange={(e) => {
                                            setSearchTerm(e.target.value);
                                            setNewPermission("");
                                        }}
                                        placeholder={t("members.permissions.search_placeholder", { defaultValue: "Search permissions..." })}
                                        type="text"
                                        value={searchTerm}
                                    />
                                </div>
                                {!!searchTerm && filteredAvailable.length > 0 && (
                                    <div className="max-h-48 overflow-y-auto rounded-2xl border border-slate-100 dark:border-slate-800 bg-white dark:bg-slate-900 shadow-lg">
                                        {filteredAvailable.map((p) => (
                                            <button
                                                className={`w-full text-left px-6 py-3 text-sm font-mono hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors ${newPermission === p ? "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-400 font-bold" : "text-slate-600 dark:text-slate-300"}`}
                                                key={p}
                                                onClick={() => { setNewPermission(p); setSearchTerm(p); }}
                                                type="button"
                                            >
                                                {p}
                                            </button>
                                        ))}
                                    </div>
                                  )}
                                {!!newPermission && (
                                    <p className="text-[10px] text-amber-600 dark:text-amber-400 font-bold uppercase tracking-widest ml-1">
                                        ✓ {newPermission}
                                    </p>
                                  )}
                            </div>
                            <div className="space-y-3 px-1">
                                <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1" htmlFor="perm-expires-at-input">
                                    {t("members.permissions.expiry_label", { defaultValue: "Expiration" })}
                                </label>
                                <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-6 py-4 focus:ring-4 focus:ring-amber-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight shadow-sm" id="perm-expires-at-input" onChange={(e) => { setExpiresAt(e.target.value); }} required type="datetime-local" value={expiresAt}
                                />
                            </div>
                            <div className="pt-2">
                                <button
                                    className="w-full bg-slate-900 text-white py-5 rounded-2xl font-bold text-xs uppercase tracking-[0.2em] hover:bg-amber-700 transition-all shadow-2xl shadow-slate-400 dark:shadow-black/40 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed"
                                    disabled={!newPermission || !expiresAt}
                                    type="submit"
                                >
                                    {t("members.permissions.grant_btn", { defaultValue: "Grant Permission" }).toUpperCase()}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            <ConfirmationDialog />
        </Modal>
    );
}
