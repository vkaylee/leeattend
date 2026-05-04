import {
  ChevronDown,
  ChevronRight,
  HardDrive,
  Pencil,
  RefreshCw,
  Server,
  Trash2
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { AuthorizedAgent, Device, DeviceStatus } from "@/lib/api";
import { cn } from "@/lib/utils";

import { AgentDeviceList } from "./agent-device-list";

interface AgentAccordionCardProps {
  agent: AuthorizedAgent;
  devices: Device[];
  deviceStatuses?: Record<string, DeviceStatus>;
  isExpanded: boolean;
  isOnline: boolean;
  onAddDevice?: (agentId: string) => void;
  onDelete: (agent: AuthorizedAgent) => Promise<void>;
  onDeleteDevice: (id: string) => Promise<void>;
  onEdit: (agent: AuthorizedAgent) => void;
  onEditDevice?: (device: Device) => void;
  onRefreshDevice?: (deviceId: string) => void;
  onToggleExpand: () => void;
  onToggleTrust: (agent: AuthorizedAgent) => Promise<void>;
  onUpdateDevice: (id: string, data: Partial<Device>) => Promise<void>;
}

interface AgentHeaderProps {
  agent: AuthorizedAgent;
  deviceCount: number;
  isExpanded: boolean;
  onDelete: () => void;
  onEdit: (agent: AuthorizedAgent) => void;
  onToggleExpand: () => void;
  onToggleTrust: () => void;
}

function AgentHeader({
  agent,
  deviceCount,
  isExpanded,
  onDelete,
  onEdit,
  onToggleExpand,
  onToggleTrust,
}: AgentHeaderProps) {
  const { t } = useTranslation();

  return (
    <div
      className="flex items-center gap-6 p-8 cursor-pointer group focus:outline-none focus:ring-2 focus:ring-inset focus:ring-primary-500"
      data-testid={`agent-header-${agent.id}`}
      onClick={onToggleExpand}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onToggleExpand();
        }
      }}
      role="button"
      tabIndex={0}
    >
      <div className="text-slate-400 dark:text-slate-500 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
        {isExpanded ? <ChevronDown size={20} /> : <ChevronRight size={20} />}
      </div>

      <div className={cn(
        "w-14 h-14 rounded-2xl flex items-center justify-center shadow-inner border transition-all",
        agent.is_active
          ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800/50 group-hover:scale-110"
          : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700"
      )}>
        <Server size={24} />
      </div>

      <div className="flex-1 min-w-0">
        <div className="font-bold text-slate-700 dark:text-slate-200 text-sm uppercase tracking-tight truncate" data-testid={`agent-name-${agent.id}`}>
          {agent.name}
        </div>
        <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold uppercase opacity-70">
          ID: {agent.id}
        </div>
      </div>

      <div className="flex items-center gap-2 shrink-0">
        <span className={cn(
          "px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm",
          agent.is_trusted
            ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
            : "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800"
        )}>
          {agent.is_trusted ? t("common.agents.status.trusted") : t("common.agents.status.pending")}
        </span>
        {!agent.is_active && (
          <span className="px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700">
            {t("common.status_inactive")}
          </span>
        )}
      </div>

      <div className="flex items-center gap-1.5 bg-slate-100 dark:bg-slate-700 px-3 py-1.5 rounded-lg shrink-0">
        <HardDrive className="text-slate-400 dark:text-slate-500" size={14} />
        <span className="text-xs font-bold text-slate-600 dark:text-slate-300">{deviceCount}</span>
      </div>

      <div className="flex items-center gap-2 text-slate-500 dark:text-slate-400 text-[10px] font-mono font-bold uppercase bg-slate-50 dark:bg-slate-800 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-800 shrink-0">
        <RefreshCw size={12} />
        {agent.last_seen_at ? new Date(agent.last_seen_at).toLocaleString() : t("common.never")}
      </div>

      <div
        className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-all shrink-0"
        onClick={e => { e.stopPropagation(); }}
        onKeyDown={e => { e.stopPropagation(); }}
        role="none"
      >
        <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" data-testid={`agent-edit-${agent.id}`} onClick={() => { onEdit(agent); }}>
          <Pencil size={18} />
        </button>
        <button
          className={cn(
            "px-4 py-2 rounded-xl text-[9px] font-bold uppercase tracking-widest border shadow-sm transition-all active:scale-95",
            agent.is_trusted
              ? "text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:bg-rose-950 border-rose-100 dark:border-rose-800 bg-white dark:bg-slate-900"
              : "bg-emerald-600 text-white border-transparent hover:bg-emerald-700"
          )}
          data-testid={`agent-trust-toggle-${agent.id}`}
          onClick={onToggleTrust}
        >
          {agent.is_trusted ? t("common.revoke") : t("common.approve")}
        </button>
        <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" data-testid={`agent-delete-${agent.id}`} onClick={onDelete}>
          <Trash2 size={18} />
        </button>
      </div>
    </div>
  );
}

export default function AgentAccordionCard({
  agent,
  deviceStatuses = {},
  devices,
  isExpanded,
  isOnline,
  onAddDevice,
  onDelete,
  onDeleteDevice,
  onEdit,
  onEditDevice,
  onRefreshDevice,
  onToggleExpand,
  onToggleTrust,
  onUpdateDevice
}: AgentAccordionCardProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const { confirm, ConfirmationDialog } = useConfirm();

  async function handleToggleTrust() {
    const confirmKey = agent.is_trusted ? "agents.messages.confirm_revoke" : "agents.messages.confirm_approve";
    const ok = await confirm({
      title: t("common.warning"),
      message: t(confirmKey, { name: agent.name }),
      variant: agent.is_trusted ? "danger" : "info"
    });
    if (!ok) return;
    try {
      await onToggleTrust(agent);
      toast.success(t(agent.is_trusted ? "agents.messages.revoke_success" : "agents.messages.approve_success", { name: agent.name }));
    } catch (e) {
      console.error(e);
    }
  }

  async function handleDelete() {
    const ok = await confirm({
      title: t("common.warning"),
      message: t("agents.messages.confirm_delete", { name: agent.name }),
      variant: "danger"
    });
    if (!ok) return;
    try {
      await onDelete(agent);
      toast.success(t("agents.messages.delete_success"));
    } catch (e) {
      console.error(e);
    }
  }

  return (
    <div
      className={cn(
        "bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden transition-all shadow-sm",
        "hover:shadow-lg hover:shadow-primary-100/20",
        !agent.is_active && "opacity-50 grayscale"
      )}
      data-testid={`agent-card-${agent.id}`}
    >
      <AgentHeader
        agent={agent}
        deviceCount={devices.length}
        isExpanded={isExpanded}
        onDelete={() => { void handleDelete(); }}
        onEdit={onEdit}
        onToggleExpand={onToggleExpand}
        onToggleTrust={() => { void handleToggleTrust(); }}
      />

      <div className={cn(
        "overflow-hidden transition-all duration-500",
        isExpanded ? "max-h-[5000px] opacity-100" : "max-h-0 opacity-0"
      )}>
        <div className="border-t border-slate-100 dark:border-slate-800 mx-8" />
        <div className="p-8 pt-6">
          <AgentDeviceList
            agentId={agent.id}
            devices={devices}
            deviceStatuses={deviceStatuses}
            isExpanded={isExpanded}
            isOnline={isOnline}
            onAddDevice={onAddDevice}
            onDeleteDevice={onDeleteDevice}
            onEditDevice={onEditDevice}
            onRefreshDevice={onRefreshDevice}
            onUpdateDevice={onUpdateDevice}
          />
        </div>
      </div>
      <ConfirmationDialog />
    </div>
  );
}
