import { useTranslation } from "react-i18next";
import {
  ChevronDown,
  ChevronRight,
  Server,
  Edit2,
  Trash2,
  RefreshCw,
  HardDrive
} from "lucide-react";
import { AuthorizedAgent, Device } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useToast } from "@/components/ui/Toast";
import DeviceListItem from "./DeviceListItem";

interface AgentAccordionCardProps {
  agent: AuthorizedAgent;
  devices: Device[];
  isExpanded: boolean;
  onToggleExpand: () => void;
  onEdit: (agent: AuthorizedAgent) => void;
  onToggleTrust: (agent: AuthorizedAgent) => Promise<void>;
  onDelete: (agent: AuthorizedAgent) => Promise<void>;
  onUpdateDevice: (id: string, data: Partial<Device>) => Promise<void>;
  onDeleteDevice: (id: string) => Promise<void>;
  onEditDevice?: (device: Device) => void;
}

export default function AgentAccordionCard({
  agent,
  devices,
  isExpanded,
  onToggleExpand,
  onEdit,
  onToggleTrust,
  onDelete,
  onUpdateDevice,
  onDeleteDevice,
  onEditDevice
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
      className={cn(
        "bg-white rounded-[2.5rem] border border-slate-100 overflow-hidden transition-all",
        "hover:shadow-lg hover:shadow-primary-100/20",
        !agent.is_active && "opacity-50 grayscale"
      )}
    >
      {/* Header row - clickable to expand */}
      <div
        onClick={onToggleExpand}
        className="flex items-center gap-6 p-8 cursor-pointer group"
      >
        {/* Chevron */}
        <div className="text-slate-400 group-hover:text-primary-600 transition-colors">
          {isExpanded ? <ChevronDown size={20} /> : <ChevronRight size={20} />}
        </div>

        {/* Agent icon */}
        <div className={cn(
          "w-14 h-14 rounded-2xl flex items-center justify-center shadow-inner border transition-all",
          agent.is_active
            ? "bg-primary-50 text-primary-600 border-primary-100/50 group-hover:scale-110"
            : "bg-slate-100 text-slate-500 border-slate-200"
        )}>
          <Server size={24} />
        </div>

        {/* Agent info */}
        <div className="flex-1 min-w-0">
          <div className="font-bold text-slate-700 text-sm uppercase tracking-tight truncate">
            {agent.name}
          </div>
          <div className="text-[10px] text-slate-500 font-mono font-bold uppercase opacity-70">
            ID: {agent.id}
          </div>
        </div>

        {/* Status badges */}
        <div className="flex items-center gap-2 shrink-0">
          <span className={cn(
            "px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm",
            agent.is_trusted
              ? "bg-emerald-50 text-emerald-600 border-emerald-100"
              : "bg-amber-50 text-amber-600 border-amber-100"
          )}>
            {agent.is_trusted ? t("common.agents.status.trusted") : t("common.agents.status.pending")}
          </span>
          {!agent.is_active && (
            <span className="px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-slate-100 text-slate-500 border-slate-200">
              {t("common.status_inactive")}
            </span>
          )}
        </div>

        {/* Device count badge */}
        <div className="flex items-center gap-1.5 bg-slate-100 px-3 py-1.5 rounded-lg shrink-0">
          <HardDrive size={14} className="text-slate-400" />
          <span className="text-xs font-bold text-slate-600">{devices.length}</span>
        </div>

        {/* Last seen */}
        <div className="flex items-center gap-2 text-slate-500 text-[10px] font-mono font-bold uppercase bg-slate-50 px-3 py-1.5 rounded-lg border border-slate-100 shrink-0">
          <RefreshCw size={12} />
          {agent.last_seen_at ? new Date(agent.last_seen_at).toLocaleString() : t("common.never")}
        </div>

        {/* Action buttons - show on hover */}
        <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-all shrink-0" onClick={e => e.stopPropagation()}>
          <button onClick={() => onEdit(agent)} className="p-3 bg-white text-slate-500 hover:text-primary-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.edit")}>
            <Edit2 size={18} />
          </button>
          <button onClick={handleToggleTrust} className={cn(
            "px-4 py-2 rounded-xl text-[9px] font-bold uppercase tracking-widest border shadow-sm transition-all active:scale-95",
            agent.is_trusted
              ? "text-rose-600 hover:bg-rose-50 border-rose-100 bg-white"
              : "bg-emerald-600 text-white border-transparent hover:bg-emerald-700"
          )}>
            {agent.is_trusted ? t("common.revoke") : t("common.approve")}
          </button>
          <button onClick={handleDelete} className="p-3 bg-white text-slate-500 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.delete")}>
            <Trash2 size={18} />
          </button>
        </div>
      </div>

      {/* Expandable content */}
      <div className={cn(
        "overflow-hidden transition-all duration-300",
        isExpanded ? "max-h-[2000px] opacity-100" : "max-h-0 opacity-0"
      )}>
        <div className="border-t border-slate-100 mx-8" />
        <div className="p-8 pt-6 space-y-3">
          {devices.length === 0 ? (
            <div className="text-center py-8 text-slate-400 text-xs font-bold uppercase tracking-widest">
              {t("devices.no_devices_for_agent")}
            </div>
          ) : (
            devices.map(device => (
              <DeviceListItem
                key={device.id}
                device={device}
                parentAgentActive={agent.is_active}
                onUpdate={onUpdateDevice}
                onDelete={onDeleteDevice}
                onEdit={onEditDevice}
              />
            ))
          )}
        </div>
      </div>
    </div>
  );
}
