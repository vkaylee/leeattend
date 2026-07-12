import { useState, useMemo, useCallback } from "react";

import { AuthorizedAgent, Device, DeviceStatus, AgentStatus } from "@/lib/api";
import { logError } from "@/lib/logger";

export function useConnectivityState() {
  const [expandedAgents, setExpandedAgents] = useState<Set<string>>(new Set());
  const [showAddForm, setShowAddForm] = useState(false);
  const [showAddDeviceForm, setShowAddDeviceForm] = useState(false);
  const [newAgent, setNewAgent] = useState({ id: "", name: "", token: "" });
  const [newDevice, setNewDevice] = useState({ name: "", agent_id: "", ip: "", port: 4370, brand: "zkteco", password: "", timezone: "UTC" });
  const [editingAgent, setEditingAgent] = useState<AuthorizedAgent | null>(null);
  const [editingDevice, setEditingDevice] = useState<Device | null>(null);

  const toggleAgent = useCallback((id: string) => {
    setExpandedAgents(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  return {
    expandedAgents,
    setExpandedAgents,
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
  };
}

export function useConnectivityMaps(agents: AuthorizedAgent[], agentsStatus: AgentStatus[], devices: Device[]) {
  const agentOnlineMap = useMemo(() => {
    const map: Record<string, boolean> = {};
    agentsStatus.forEach(a => { map[a.agent_id] = a.is_online; });
    return map;
  }, [agentsStatus]);

  const deviceStatuses = useMemo(() => {
    const map: Record<string, DeviceStatus> = {};
    agentsStatus.forEach(a => {
      a.devices.forEach(d => { map[d.device_id.toLowerCase()] = d; });
    });
    return map;
  }, [agentsStatus]);

  const { groupedDevices, unassignedDevices } = useMemo(() => {
    const grouped: Record<string, Device[]> = {};
    const unassigned: Device[] = [];
    for (const device of devices) {
      if (agents.some(a => a.id === device.agent_id)) {
        grouped[device.agent_id] ??= [];
        grouped[device.agent_id].push(device);
      } else {
        unassigned.push(device);
      }
    }
    return { groupedDevices: grouped, unassignedDevices: unassigned };
  }, [devices, agents]);

  return { agentOnlineMap, deviceStatuses, groupedDevices, unassignedDevices };
}

interface ConnectivityActionsProps {
  createAgent: (agent: { id: string; name: string; token: string }) => Promise<unknown>;
  createDevice: (device: Partial<Device>) => Promise<unknown>;
  editingDevice: Device | null;
  newAgent: { id: string; name: string; token: string };
  newDevice: Partial<Device>;
  refetchAgents: () => Promise<unknown>;
  refetchStatus: () => Promise<unknown>;
  sendCommand: (args: { id: string; command: unknown }) => Promise<unknown>;
  setEditingDevice: (device: Device | null) => void;
  setNewAgent: (agent: { id: string; name: string; token: string }) => void;
  setShowAddDeviceForm: (show: boolean) => void;
  setShowAddForm: (show: boolean) => void;
  t: (key: string) => string;
  toast: { success: (msg: string) => void; error: (msg: string) => void };
  updateDevice: (id: string, device: Partial<Device>) => Promise<unknown>;
}

export function useConnectivityActions({
  createAgent,
  createDevice,
  editingDevice,
  newAgent,
  newDevice,
  refetchAgents,
  refetchStatus,
  sendCommand,
  setEditingDevice,
  setNewAgent,
  setShowAddDeviceForm,
  setShowAddForm,
  t,
  toast,
  updateDevice
}: ConnectivityActionsProps) {
  const handleAddAgent = async (e: React.SyntheticEvent) => {
    e.preventDefault();
    try {
      await createAgent({ id: newAgent.id, name: newAgent.name, token: newAgent.token });
      toast.success(t("agents.messages.add_success"));
      setShowAddForm(false);
      setNewAgent({ id: "", name: "", token: "" });
      void refetchAgents();
    } catch (err: unknown) {
      logError(err, { component: "useConnectivityLogic" });
      toast.error(t("agents.messages.add_error"));
    }
  };

  const handleSaveDevice = async (e: React.SyntheticEvent) => {
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
    } catch (err: unknown) {
      logError(err, { component: "useConnectivityLogic" });
      toast.error(editingDevice ? t("devices.toast.update_error") : t("devices.toast.add_error"));
    }
  };

  const handleRefreshDevice = async (agentId: string, deviceId: string) => {
    try {
      await sendCommand({ id: agentId, command: { method: "get_device_info", device_id: deviceId, data: {} } });
      toast.success(t("devices.toast.refresh_sent"));
      setTimeout(() => { void refetchStatus(); }, 2000);
    } catch (err: unknown) {
      logError(err, { component: "useConnectivityLogic" });
      toast.error(t("devices.toast.refresh_error"));
    }
  };

  return { handleAddAgent, handleSaveDevice, handleRefreshDevice };
}
