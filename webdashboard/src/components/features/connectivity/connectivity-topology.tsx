import React from "react";
import { useTranslation } from "react-i18next";

import AgentEditModal from "@/components/features/agents/agent-edit-modal";
import { useToast } from "@/components/ui/toast";
import { useAuthorizedAgents, useAgentsStatus } from "@/hooks/use-agents";
import { useDevices } from "@/hooks/use-devices";

import { AddAgentForm } from "./add-agent-form";
import { AddEditDeviceModal } from "./add-edit-device-modal";
import { AgentsList } from "./agents-list";
import { ConnectivityHeader } from "./connectivity-header";
import { EmptyAgentsState } from "./empty-agents-state";
import { UnassignedDevicesSection } from "./unassigned-devices-section";
import { useConnectivityState, useConnectivityMaps, useConnectivityActions } from "./use-connectivity-logic";

export default function ConnectivityTopology() {
  const { t } = useTranslation();
  const toast = useToast();
  const {
    agents, isLoading: agentsLoading, refetch: refetchAgents, includeInactive, setIncludeInactive,
    updateTrust, deleteAgent, createAgent, sendCommand
  } = useAuthorizedAgents();

  const { agents: agentsStatus, refetch: refetchStatus } = useAgentsStatus();
  const { devices, isLoading: devicesLoading, createDevice, updateDevice, deleteDevice } = useDevices();

  const state = useConnectivityState();
  const maps = useConnectivityMaps(agents, agentsStatus, devices);

  const { handleAddAgent, handleSaveDevice, handleRefreshDevice } = useConnectivityActions({
    createAgent, createDevice, editingDevice: state.editingDevice, newAgent: state.newAgent,
    newDevice: state.newDevice, refetchAgents, refetchStatus, sendCommand,
    setEditingDevice: state.setEditingDevice, setNewAgent: state.setNewAgent,
    setShowAddDeviceForm: state.setShowAddDeviceForm, setShowAddForm: state.setShowAddForm,
    t, toast, updateDevice
  });

  if (agentsLoading || devicesLoading) {
    return <div className="p-8 animate-pulse text-slate-400 font-bold uppercase tracking-widest">{t("common.loading")}</div>;
  }

  return (
    <div className="p-8 space-y-12 max-w-[1600px] mx-auto" data-testid="connectivity-topology">
      <ConnectivityHeader
        agents={agents} includeInactive={includeInactive}
        onAddAgent={() => { state.setShowAddForm(!state.showAddForm); }}
        onAddDevice={() => { state.setEditingDevice(null); state.setShowAddDeviceForm(true); }}
        onCollapseAll={() => { state.expandedAgents.forEach(id => { state.toggleAgent(id); }); }}
        onExpandAll={() => { agents.forEach(a => { if (!state.expandedAgents.has(a.id)) { state.toggleAgent(a.id); } }); }}
        onToggleInactive={() => { setIncludeInactive(!includeInactive); }}
      />

      {state.showAddForm ? (
        <AddAgentForm
          newAgent={state.newAgent}
          onCancel={() => { state.setShowAddForm(false); }}
          onChange={(field, value) => { state.setNewAgent({ ...state.newAgent, [field]: value }); }}
          onSubmit={handleAddAgent}
        />
      ) : null}

      {agents.length === 0 ? (
        <EmptyAgentsState onAddAgent={() => { state.setShowAddForm(true); }} />
      ) : (
        <AgentsList
          agentOnlineMap={maps.agentOnlineMap} agents={agents} deviceStatuses={maps.deviceStatuses}
          expandedAgents={state.expandedAgents} groupedDevices={maps.groupedDevices}
          onAddDevice={(agentId) => { state.setNewDevice({ ...state.newDevice, agent_id: agentId }); state.setShowAddDeviceForm(true); }}
          onDeleteAgent={async (a) => { await deleteAgent(a.id); void refetchAgents(); }}
          onDeleteDevice={async (id) => { await deleteDevice(id); void refetchStatus(); }}
          onEditAgent={(a) => { state.setEditingAgent(a); }}
          onEditDevice={(device) => { state.setEditingDevice(device); state.setShowAddDeviceForm(true); }}
          onRefreshDevice={handleRefreshDevice} onToggleExpand={(id) => { state.toggleAgent(id); }}
          onToggleTrust={async (a) => { await updateTrust({ id: a.id, is_trusted: !a.is_trusted }); void refetchAgents(); }}
          onUpdateDevice={async (id, data) => { await updateDevice(id, data); void refetchStatus(); }}
        />
      )}

      {maps.unassignedDevices.length > 0 ? (
        <UnassignedDevicesSection
          onDeleteDevice={async (id) => { await deleteDevice(id); void refetchStatus(); }}
          onEditDevice={(device) => { state.setEditingDevice(device); state.setShowAddDeviceForm(true); }}
          onUpdateDevice={async (id, data) => { await updateDevice(id, data); void refetchStatus(); }}
          unassignedDevices={maps.unassignedDevices}
        />
      ) : null}

      <AddEditDeviceModal
        agents={agents} editingDevice={state.editingDevice} isOpen={state.showAddDeviceForm}
        newDevice={state.newDevice} onClose={() => { state.setShowAddDeviceForm(false); state.setEditingDevice(null); }}
        onNewDeviceChange={(field, value) => { state.setNewDevice({ ...state.newDevice, [field]: value as string }); }}
        onSave={handleSaveDevice} onSetEditingDevice={(d) => { state.setEditingDevice(d); }}
      />

      <AgentEditModal
        agent={state.editingAgent} isOpen={!!state.editingAgent}
        onClose={() => { state.setEditingAgent(null); }}
        onSave={() => { void refetchAgents(); state.setEditingAgent(null); }}
      />
    </div>
  );
}
