import { useTranslation } from "react-i18next";
import {
  ChevronDown,
  ChevronRight,
  Server,
  Edit2,
  Trash2,
  RefreshCw,
  HardDrive,
  Plus
} from "lucide-react";
import { AuthorizedAgent, Device, DeviceStatus } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useToast } from "@/components/ui/Toast";
import DeviceListItem from "./DeviceListItem";

interface AgentAccordionCardProps {
  agent: AuthorizedAgent;
  devices: Device[];
  deviceStatuses?: Record<string, DeviceStatus>;
  isExpanded: boolean;
  onToggleExpand: () => void;
  onEdit: (agent: AuthorizedAgent) => void;
  onToggleTrust: (agent: AuthorizedAgent) => Promise<void>;
  onDelete: (agent: AuthorizedAgent) => Promise<void>;
  onUpdateDevice: (id: string, data: Partial<Device>) => Promise<void>;
  onDeleteDevice: (id: string) => Promise<void>;
  onEditDevice?: (device: Device) => void;
  onRefreshDevice?: (deviceId: string) => void;
  onAddDevice?: (agentId: string) => void;
  isOnline: boolean;
}

export default function AgentAccordionCard({
  agent,
  devices,
  deviceStatuses = {},
  isExpanded,
  onToggleExpand,
  onEdit,
  onToggleTrust,
  onDelete,
  onUpdateDevice,
  onDeleteDevice,
  onEditDevice,
  onRefreshDevice,
  onAddDevice,
  isOnline
}: AgentAccordionCardProps) {
  const { t } = useTranslation();
  const toast = useToast();

  async function handleToggleTrust() {
    const confirmKey = agent.is_trusted ? "agents.messages.confirm_revoke" : "agents.messages.confirm_approve";
    if (!confirm(t(confirmKey, { name: agent.name }))) return;
    try {
      await onToggleTrust(agent);
      toast.success(t(agent.is_trusted ? "agents.messages.revoke_success" : "agents.messages.approve_success", { name: agent.name }));
    } catch (e) {
      console.error(e);
    }
  }

  async function handleDelete() {
    if (!confirm(t("agents.messages.confirm_delete", { name: agent.name }))) return;
    try {
      await onDelete(agent);
      toast.success(t("agents.messages.delete_success"));
    } catch (e) {
      console.error(e);
    }
  }

  return (
    <div
      data-testid={`agent-card-${agent.id}`}
      className={cn(
        "bg-white dark:bg-slate-900 rounded-[2.5rem] border border-slate-100 dark:border-slate-800 overflow-hidden transition-all",
        "hover:shadow-lg hover:shadow-primary-100/20",
        !agent.is_active && "opacity-50 grayscale"
      )}
    >
      {/* Header row - clickable to expand */}
      <div
        data-testid={`agent-header-${agent.id}`}
        onClick={onToggleExpand}
        className="flex items-center gap-6 p-8 cursor-pointer group"
      >
        {/* Chevron */}
        <div className="text-slate-400 dark:text-slate-500 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
          {isExpanded ? <ChevronDown size={20} /> : <ChevronRight size={20} />}
        </div>

        {/* Agent icon */}
        <div className={cn(
          "w-14 h-14 rounded-2xl flex items-center justify-center shadow-inner border transition-all",
          agent.is_active
            ? "bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 border-primary-100 dark:border-primary-800/50 group-hover:scale-110"
            : "bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700"
        )}>
          <Server size={24} />
        </div>

        {/* Agent info */}
        <div className="flex-1 min-w-0">
          <div data-testid={`agent-name-${agent.id}`} className="font-bold text-slate-700 dark:text-slate-200 text-sm uppercase tracking-tight truncate">
            {agent.name}
          </div>
          <div className="text-[10px] text-slate-500 dark:text-slate-400 font-mono font-bold uppercase opacity-70">
            ID: {agent.id}
          </div>
        </div>

        {/* Status badges */}
        <div className="flex items-center gap-2 shrink-0">
          <span
            data-testid={`agent-trust-status-${agent.id}`}
            className={cn(
              "px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm",
              agent.is_trusted
                ? "bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800"
                : "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800"
            )}
          >
            {agent.is_trusted ? t("common.agents.status.trusted") : t("common.agents.status.pending")}
          </span>
          {!agent.is_active && (
            <span data-testid={`agent-inactive-badge-${agent.id}`} className="px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-slate-100 dark:bg-slate-700 text-slate-500 dark:text-slate-400 border-slate-200 dark:border-slate-700">
              {t("common.status_inactive")}
            </span>
          )}
        </div>

        {/* Device count badge */}
        <div data-testid={`agent-device-count-${agent.id}`} className="flex items-center gap-1.5 bg-slate-100 dark:bg-slate-700 px-3 py-1.5 rounded-lg shrink-0">
          <HardDrive size={14} className="text-slate-400 dark:text-slate-500" />
          <span className="text-xs font-bold text-slate-600 dark:text-slate-300">{devices.length}</span>
        </div>

        {/* Last seen */}
        <div className="flex items-center gap-2 text-slate-500 dark:text-slate-400 text-[10px] font-mono font-bold uppercase bg-slate-50 dark:bg-slate-800 px-3 py-1.5 rounded-lg border border-slate-100 dark:border-slate-800 shrink-0">
          <RefreshCw size={12} />
          {agent.last_seen_at ? new Date(agent.last_seen_at).toLocaleString() : t("common.never")}
        </div>

        {/* Action buttons - show on hover */}
        <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-all shrink-0" onClick={e => e.stopPropagation()}>
          <button data-testid={`agent-edit-${agent.id}`} onClick={() => onEdit(agent)} className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-primary-600 dark:text-primary-400 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("common.edit")}>
            <Edit2 size={18} />
          </button>
          <button
            data-testid={`agent-trust-toggle-${agent.id}`}
            onClick={handleToggleTrust}
            className={cn(
              "px-4 py-2 rounded-xl text-[9px] font-bold uppercase tracking-widest border shadow-sm transition-all active:scale-95",
              agent.is_trusted
                ? "text-rose-600 dark:text-rose-300 hover:bg-rose-50 dark:bg-rose-950 border-rose-100 dark:border-rose-800 bg-white dark:bg-slate-900"
                : "bg-emerald-600 text-white border-transparent hover:bg-emerald-700"
            )}
          >
            {agent.is_trusted ? t("common.revoke") : t("common.approve")}
          </button>
          <button data-testid={`agent-delete-${agent.id}`} onClick={handleDelete} className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" title={t("common.delete")}>
            <Trash2 size={18} />
          </button>
        </div>
      </div>

      {/* Expandable content */}
      <div className={cn(
        "overflow-hidden transition-all duration-500",
        isExpanded ? "max-h-[5000px] opacity-100" : "max-h-0 opacity-0"
      )}>
        <div className="border-t border-slate-100 dark:border-slate-800 mx-8" />
        <div className="p-8 pt-6">
          {devices.length === 0 ? (
            <div className="text-center py-8 text-slate-400 dark:text-slate-500 text-xs font-bold uppercase tracking-widest">
              {t("devices.no_devices_for_agent")}
            </div>
          ) : (
            <div className="flex flex-col gap-4">
              {devices.map(device => (
                <DeviceListItem
                  key={device.id}
                  device={device}
                  status={deviceStatuses[device.id.toLowerCase()]}
                  parentAgentActive={isOnline}
                  onUpdate={onUpdateDevice}
                  onDelete={onDeleteDevice}
                  onEdit={onEditDevice}
                  onRefresh={onRefreshDevice}
                />
              ))}

              {/* Add Device Button - Horizontal Style */}
              {onAddDevice && (
                <button
                  data-testid={`add-device-btn-${agent.id}`}
                  onClick={() => onAddDevice(agent.id)}
                  className="group relative flex items-center justify-center p-6 border-2 border-dashed border-slate-200 dark:border-slate-700 rounded-[1.5rem] text-slate-400 dark:text-slate-500 hover:text-primary-600 dark:hover:text-primary-400 hover:border-primary-300 hover:bg-primary-50 dark:bg-primary-950/30 transition-all gap-4"
                >
                  <div className="w-10 h-10 rounded-xl bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 flex items-center justify-center shadow-sm group-hover:scale-110 transition-transform">
                    <Plus size={20} />
                  </div>
                  <span className="text-xs font-black uppercase tracking-widest">{t("devices.add_btn")}</span>
                </button>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
