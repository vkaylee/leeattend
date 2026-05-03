import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import AgentEditModal from "@/components/features/agents/agent-edit-modal";
import { useToast } from "@/components/ui/toast";
import { useAuthorizedAgents, useAgentsStatus } from "@/hooks/use-agents";
import { useDevices } from "@/hooks/use-devices";
import { AuthorizedAgent, Device, AgentStatus } from "@/lib/api";

import AgentAccordionCard from "./agent-accordion-card";
import { AddAgentForm } from "./add-agent-form";
import { AddEditDeviceModal } from "./add-edit-device-modal";
import { ConnectivityHeader } from "./connectivity-header";
import { EmptyAgentsState } from "./empty-agents-state";
import { UnassignedDevicesSection } from "./unassigned-devices-section";
import { useConnectivityState, useConnectivityMaps } from "./use-connectivity-logic";

export default function ConnectivityTopology() {
  const { t } = useTranslation();
  const toast = useToast();
  const {
    agents,
    isLoading: agentsLoading,
    refetch: refetchAgents,
    includeInactive,
    setIncludeInactive,
    updateTrust,
    deleteAgent,
    createAgent,
    sendCommand
  } = useAuthorizedAgents();

  const { agents: agentsStatus, refetch: refetchStatus } = useAgentsStatus();
  const { devices, isLoading: devicesLoading, createDevice, updateDevice, deleteDevice } = useDevices();

  const {
    expandedAgents,
    showAddForm,
    setShowAddForm,
    showAddDeviceForm,
    setShowAddDeviceForm,
    newAgent,
    setNewAgent,
    newDevice,
    setNewDevice,
    editingAgent,
    setEditingAgent,
    editingDevice,
    setEditingDevice,
    toggleAgent
  } = useConnectivityState();

  const { agentOnlineMap, deviceStatuses, groupedDevices, unassignedDevices } = useConnectivityMaps(agents, agentsStatus as AgentStatus[], devices);

  async function handleAddAgent(e: React.SyntheticEvent) {
    e.preventDefault();
    try {
      await createAgent({ id: newAgent.id, name: newAgent.name, token: newAgent.token });
      toast.success(t("agents.messages.add_success"));
      setShowAddForm(false);
      setNewAgent({ id: "", name: "", token: "" });
      void refetchAgents();
    } catch (e) {
      console.error(e);
      toast.error(t("agents.messages.add_error"));
    }
  }

  async function handleSaveDevice(e: React.SyntheticEvent) {
    e.preventDefault();
    try {
      if (editingDevice) {
        await updateDevice(editingDevice.id, editingDevice);
        toast.success(t("devices.toast.update_success"));
        setEditingDevice(null);
      } else {
        await createDevice(newDevice);
        toast.success(t("devices.toast.add_success"));
      }
      setShowAddDeviceForm(false);
      void refetchStatus();
    } catch (e) {
      console.error(e);
      toast.error(editingDevice ? t("devices.toast.update_error") : t("devices.toast.add_error"));
    }
  }

  async function handleRefreshDevice(agentId: string, deviceId: string) {
    try {
      await sendCommand({ id: agentId, command: { method: "get_device_info", device_id: deviceId, data: {} } });
      toast.success(t("devices.toast.refresh_sent"));
      setTimeout(() => { void refetchStatus(); }, 2000);
    } catch (e: unknown) {
      console.error("Refresh failed:", e);
      const message = e instanceof Error ? e.message : "";
      toast.error(`${t("devices.toast.refresh_error")} ${message ? `(${message})` : ""}`);
    }
  }

  if (agentsLoading || devicesLoading) {
    return <div className="p-8 animate-pulse text-slate-400 font-bold uppercase tracking-widest">{t("common.loading")}</div>;
  }

  return (
    <div className="p-8 space-y-12 max-w-[1600px] mx-auto" data-testid="connectivity-topology">
      <ConnectivityHeader
        agents={agents}
        includeInactive={includeInactive}
        onAddAgent={() => setShowAddForm(!showAddForm)}
        onAddDevice={() => {
          setEditingDevice(null);
          setShowAddDeviceForm(true);
        }}
        onCollapseAll={() => expandedAgents.forEach(id => toggleAgent(id))}
        onExpandAll={() => agents.forEach(a => { if (!expandedAgents.has(a.id)) toggleAgent(a.id); })}
        onToggleInactive={() => setIncludeInactive(!includeInactive)}
      />

      {showAddForm && (
        <AddAgentForm
          newAgent={newAgent}
          onCancel={() => setShowAddForm(false)}
          onChange={(field, value) => setNewAgent({ ...newAgent, [field]: value })}
          onSubmit={handleAddAgent}
        />
      )}

      {agents.length === 0 ? (
        <EmptyAgentsState onAddAgent={() => setShowAddForm(true)} />
      ) : (
        <div className="grid grid-cols-1 gap-6">
          {agents.map(agent => (
            <AgentAccordionCard
              agent={agent}
              devices={groupedDevices[agent.id] ?? []}
              deviceStatuses={deviceStatuses}
              isExpanded={expandedAgents.has(agent.id)}
              isOnline={agentOnlineMap[agent.id] ?? false}
              key={agent.id}
              onAddDevice={(agentId) => {
                setNewDevice({ ...newDevice, agent_id: agentId });
                setShowAddDeviceForm(true);
              }}
              onDelete={async (a) => { await deleteAgent(a.id); void refetchAgents(); }}
              onDeleteDevice={async (id) => { await deleteDevice(id); void refetchStatus(); }}
              onEdit={setEditingAgent}
              onEditDevice={(device) => {
                setEditingDevice(device);
                setShowAddDeviceForm(true);
              }}
              onRefreshDevice={(deviceId) => { void handleRefreshDevice(agent.id, deviceId); }}
              onToggleExpand={() => toggleAgent(agent.id)}
              onToggleTrust={async (a) => { await updateTrust({ id: a.id, is_trusted: !a.is_trusted }); void refetchAgents(); }}
              onUpdateDevice={async (id, data) => {
                await updateDevice(id, data);
                void refetchStatus();
              }}
            />
          ))}
        </div>
      )}

      {unassignedDevices.length > 0 && (
        <UnassignedDevicesSection
          onDeleteDevice={async (id) => { await deleteDevice(id); void refetchStatus(); }}
          onEditDevice={(device) => {
            setEditingDevice(device);
            setShowAddDeviceForm(true);
          }}
          onUpdateDevice={async (id, data) => {
            await updateDevice(id, data);
            void refetchStatus();
          }}
          unassignedDevices={unassignedDevices}
        />
      )}

      <AddEditDeviceModal
        agents={agents}
        editingDevice={editingDevice}
        isOpen={showAddDeviceForm}
        newDevice={newDevice}
        onClose={() => {
          setShowAddDeviceForm(false);
          setEditingDevice(null);
        }}
        onNewDeviceChange={(field, value) => setNewDevice({ ...newDevice, [field]: value as string })}
        onSave={handleSaveDevice}
        onSetEditingDevice={setEditingDevice}
      />

      <AgentEditModal
        agent={editingAgent}
        isOpen={!!editingAgent}
        onClose={() => setEditingAgent(null)}
        onSave={() => {
          void refetchAgents();
          setEditingAgent(null);
        }}
      />
    </div>
  );
}
