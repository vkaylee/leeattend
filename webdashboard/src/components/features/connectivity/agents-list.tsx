import React from "react";

import { AuthorizedAgent, Device, DeviceStatus } from "@/lib/api";

import AgentAccordionCard from "./agent-accordion-card";

interface AgentsListProps {
  agentOnlineMap: Record<string, boolean>;
  agents: AuthorizedAgent[];
  deviceStatuses: Record<string, DeviceStatus>;
  expandedAgents: Set<string>;
  groupedDevices: Record<string, Device[]>;
  onAddDevice: (agentId: string) => void;
  onDeleteAgent: (agent: AuthorizedAgent) => Promise<void>;
  onDeleteDevice: (id: string) => Promise<void>;
  onEditAgent: (agent: AuthorizedAgent) => void;
  onEditDevice: (device: Device) => void;
  onRefreshDevice: (agentId: string, deviceId: string) => void;
  onToggleExpand: (id: string) => void;
  onToggleTrust: (agent: AuthorizedAgent) => Promise<void>;
  onUpdateDevice: (id: string, data: Partial<Device>) => Promise<void>;
}

export function AgentsList({
  agentOnlineMap,
  agents,
  deviceStatuses,
  expandedAgents,
  groupedDevices,
  onAddDevice,
  onDeleteAgent,
  onDeleteDevice,
  onEditAgent,
  onEditDevice,
  onRefreshDevice,
  onToggleExpand,
  onToggleTrust,
  onUpdateDevice
}: AgentsListProps) {
  if (agents.length === 0) return null;

  return (
    <div className="grid grid-cols-1 gap-6">
      {agents.map(agent => (
        <AgentAccordionCard
          agent={agent}
          devices={groupedDevices[agent.id] ?? []}
          deviceStatuses={deviceStatuses}
          isExpanded={expandedAgents.has(agent.id)}
          isOnline={agentOnlineMap[agent.id] ?? false}
          key={agent.id}
          onAddDevice={onAddDevice}
          onDelete={onDeleteAgent}
          onDeleteDevice={onDeleteDevice}
          onEdit={onEditAgent}
          onEditDevice={onEditDevice}
          onRefreshDevice={(deviceId) => { onRefreshDevice(agent.id, deviceId); }}
          onToggleExpand={() => { onToggleExpand(agent.id); }}
          onToggleTrust={onToggleTrust}
          onUpdateDevice={onUpdateDevice}
        />
      ))}
    </div>
  );
}
