import type { TFunction } from "i18next";

import {
  X,
  Save,
  Key,
  UserCheck,
  ShieldCheck,
  ShieldAlert,
  Power,
  Cpu,
  Timer,
  Calendar
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useAuthorizedAgents } from "@/hooks/use-agents";
import { AuthorizedAgent } from "@/lib/api";
import { cn } from "@/lib/utils";

interface AgentEditModalProps {
  agent: AuthorizedAgent | null;
  isOpen: boolean;
  onClose: () => void;
  onSave: (updatedAgent: AuthorizedAgent) => void;
}

export default function AgentEditModal({ agent, isOpen, onClose, onSave }: AgentEditModalProps) {
  const { t } = useTranslation();
  const { updateAgent, isUpdating } = useAuthorizedAgents();
  
  if (!isOpen || !agent) return null;

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-slate-900/60 backdrop-blur-md animate-in fade-in duration-300">
      <div className="w-full max-w-xl bg-white dark:bg-slate-900 rounded-[2.5rem] shadow-premium-2xl border border-white overflow-hidden animate-in zoom-in-95 duration-300">
        <AgentEditModalHeader onClose={onClose} t={t} />

        <AgentEditModalForm
          agent={agent}
          isUpdating={isUpdating}
          onClose={onClose}
          onSave={onSave}
          t={t}
          updateAgent={updateAgent}
        />
      </div>
    </div>
  );
}

function AgentEditModalHeader({ t, onClose }: { onClose: () => void; t: TFunction }) {
  return (
    <div className="flex items-center justify-between p-10 border-b border-slate-50 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/30">
      <div className="flex items-center gap-5">
        <div className="bg-primary-50 dark:bg-primary-950 p-4 rounded-2xl text-primary-600 dark:text-primary-400 shadow-inner border border-primary-100 dark:border-primary-800/50">
          <UserCheck size={24} />
        </div>
        <div>
          <Typography className="text-xl font-bold text-slate-700 dark:text-slate-200 tracking-tight uppercase" variant="h2">
            {t("common.agents.edit_modal.title")}
          </Typography>
          <Typography className="text-[10px] text-slate-500 dark:text-slate-400 font-bold uppercase tracking-widest mt-1 opacity-70" variant="p">{t("common.agents.edit_modal.subtitle")}</Typography>
        </div>
      </div>
      <Button className="hover:text-rose-600 dark:hover:text-rose-300" icon={<X size={20} />} onClick={onClose} size="icon" variant="outline" />
    </div>
  );
}

interface AgentEditModalFormProps {
  agent: AuthorizedAgent;
  isUpdating: boolean;
  onClose: () => void;
  onSave: (updatedAgent: AuthorizedAgent) => void;
  t: TFunction;
  updateAgent: (params: { agent: Partial<AuthorizedAgent>; id: string }) => Promise<AuthorizedAgent>;
}

interface AgentEditModalSectionsProps {
  agent: AuthorizedAgent;
  isActive: boolean;
  isTrusted: boolean;
  maxConcurrentWorkers: number;
  name: string;
  retentionDays: number;
  setIsActive: (val: boolean) => void;
  setIsTrusted: (val: boolean) => void;
  setMaxConcurrentWorkers: (val: number) => void;
  setName: (val: string) => void;
  setRetentionDays: (val: number) => void;
  setSuccessCooldownSecs: (val: number) => void;
  setToken: (val: string) => void;
  successCooldownSecs: number;
  t: TFunction;
  token: string;
}

function AgentEditModalSections({
  agent,
  name,
  setName,
  token,
  setToken,
  maxConcurrentWorkers,
  setMaxConcurrentWorkers,
  successCooldownSecs,
  setSuccessCooldownSecs,
  retentionDays,
  setRetentionDays,
  isTrusted,
  setIsTrusted,
  isActive,
  setIsActive,
  t,
}: AgentEditModalSectionsProps) {
  return (
    <div className="space-y-6">
      <div className="space-y-2 px-1">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("common.agents.edit_modal.id_label")}</label>
        <div className="w-full bg-slate-100 dark:bg-slate-700/50 border border-slate-200 dark:border-slate-700 rounded-2xl px-5 py-4 text-slate-500 dark:text-slate-400 font-mono font-bold shadow-inner opacity-60 uppercase text-xs">{agent.id}</div>
      </div>

      <div className="space-y-2 px-1">
        <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("common.agents.edit_modal.name_label")}</label>
        <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-bold text-sm tracking-tight" onChange={(e) => { setName(e.target.value); }} placeholder={t("common.agents.edit_modal.name_placeholder")} required value={name} />
      </div>

      <div className="grid grid-cols-2 gap-8">
        <div className="space-y-2 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("common.agents.edit_modal.token_label")}</label>
          <div className="relative group">
            <Key className="absolute left-5 top-1/2 -translate-y-1/2 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" size={18} />
            <input className="w-full pl-12 pr-5 py-4 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono text-sm tracking-widest" onChange={(e) => { setToken(e.target.value); }} placeholder={t("common.agents.edit_modal.token_placeholder")} required value={token} />
          </div>
        </div>
        <div className="space-y-2 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("devices.form.strategy")}</label>
          <div className="relative group">
            <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all pr-12 font-mono font-bold text-sm text-slate-700 dark:text-slate-200" max="100" min="1" onChange={(e) => { setMaxConcurrentWorkers(parseInt(e.target.value)); }} required type="number" value={maxConcurrentWorkers} />
            <Cpu className="absolute right-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" />
          </div>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-8">
        <div className="space-y-2 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("common.agents.edit_modal.cooldown_label")}</label>
          <Typography className="text-[10px] text-slate-400 dark:text-slate-500 ml-1 -mt-1" variant="p">{t("common.agents.edit_modal.cooldown_hint")}</Typography>
          <div className="relative group">
            <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all pr-12 font-mono font-bold text-sm text-slate-700 dark:text-slate-200" max="3600" min="60" onChange={(e) => { setSuccessCooldownSecs(parseInt(e.target.value)); }} required step="60" type="number" value={successCooldownSecs} />
            <Timer className="absolute right-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" />
          </div>
        </div>
        <div className="space-y-2 px-1">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("common.agents.edit_modal.retention_label")}</label>
          <Typography className="text-[10px] text-slate-400 dark:text-slate-500 ml-1 -mt-1" variant="p">{t("common.agents.edit_modal.retention_hint")}</Typography>
          <div className="relative group">
            <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all pr-12 font-mono font-bold text-sm text-slate-700 dark:text-slate-200" max="3650" min="1" onChange={(e) => { setRetentionDays(parseInt(e.target.value)); }} required type="number" value={retentionDays} />
            <Calendar className="absolute right-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors" />
          </div>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-6 pt-2">
        <Button className={cn("h-auto flex-col gap-3 p-6 rounded-[2rem] border-2 shadow-inner", isTrusted ? "bg-emerald-50 dark:bg-emerald-950 border-emerald-100 dark:border-emerald-800 text-emerald-700 dark:text-emerald-300 shadow-emerald-900/5 hover:bg-emerald-100 dark:hover:bg-emerald-900" : "bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 text-slate-400 dark:text-slate-500")} onClick={() => { setIsTrusted(!isTrusted); }} type="button" variant="outline">
          {isTrusted ? <ShieldCheck size={24} /> : <ShieldAlert size={24} />}
          {isTrusted ? t("common.agents.status.trusted") : t("common.agents.status.pending")}
        </Button>

        <Button className={cn("h-auto flex-col gap-3 p-6 rounded-[2rem] border-2 shadow-inner", isActive ? "bg-primary-50 dark:bg-primary-950 border-primary-100 dark:border-primary-800 text-primary-700 dark:text-primary-300 shadow-primary-900/5 hover:bg-primary-100 dark:hover:bg-primary-900" : "bg-rose-50 dark:bg-rose-950 border-rose-100 dark:border-rose-800 text-rose-700 dark:text-rose-300 shadow-rose-900/5 hover:bg-rose-100 dark:hover:bg-rose-900")} onClick={() => { setIsActive(!isActive); }} type="button" variant="outline">
          <Power size={24} />
          {isActive ? t("common.status_active") : t("common.status_inactive")}
        </Button>
      </div>
    </div>
  );
}

function AgentEditModalActions({ 
  isUpdating, 
  onClose, 
  t 
}: { 
  isUpdating: boolean; 
  onClose: () => void; 
  t: TFunction 
}) {
  return (
    <div className="flex gap-4 pt-4 border-t border-slate-50 dark:border-slate-800">
      <Button className="flex-1" onClick={onClose} size="lg" type="button" variant="outline">{t("common.cancel").toUpperCase()}</Button>
      <Button className="flex-1" disabled={isUpdating} icon={<Save size={18} />} loading={isUpdating} size="lg" type="submit" variant="primary">
        {isUpdating ? t("common.saving").toUpperCase() : t("common.save_changes").toUpperCase()}
      </Button>
    </div>
  );
}

function AgentEditModalForm({ agent, isUpdating, updateAgent, onSave, onClose, t }: AgentEditModalFormProps) {
  const [name, setName] = useState(agent.name);
  const [token, setToken] = useState(agent.token);
  const [isTrusted, setIsTrusted] = useState(agent.is_trusted);
  const [isActive, setIsActive] = useState(agent.is_active);
  const [maxConcurrentWorkers, setMaxConcurrentWorkers] = useState(agent.max_concurrent_workers);
  const [successCooldownSecs, setSuccessCooldownSecs] = useState(agent.success_cooldown_secs);
  const [retentionDays, setRetentionDays] = useState(agent.retention_days);

  async function handleSubmit(e: React.SyntheticEvent) {
    e.preventDefault();
    try {
      const updated = await updateAgent({
        id: agent.id,
        agent: {
          name,
          token,
          is_trusted: isTrusted,
          is_active: isActive,
          max_concurrent_workers: maxConcurrentWorkers,
          success_cooldown_secs: successCooldownSecs,
          retention_days: retentionDays
        }
      });
      onSave(updated);
      onClose();
    } catch (err: unknown) {
      console.error(err);
    }
  }

  return (
    <form className="p-10 space-y-10 animate-in fade-in duration-500" onSubmit={handleSubmit}>
      <AgentEditModalSections
        agent={agent}
        isActive={isActive}
        isTrusted={isTrusted}
        maxConcurrentWorkers={maxConcurrentWorkers}
        name={name}
        retentionDays={retentionDays}
        setIsActive={setIsActive}
        setIsTrusted={setIsTrusted}
        setMaxConcurrentWorkers={setMaxConcurrentWorkers}
        setName={setName}
        setRetentionDays={setRetentionDays}
        setSuccessCooldownSecs={setSuccessCooldownSecs}
        setToken={setToken}
        successCooldownSecs={successCooldownSecs}
        t={t}
        token={token}
      />

      <AgentEditModalActions
        isUpdating={isUpdating}
        onClose={onClose}
        t={t}
      />
    </form>
  );
}
