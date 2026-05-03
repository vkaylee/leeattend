import { useState, useMemo, useCallback } from "react";

import { AuthorizedAgent, Device, DeviceStatus, AgentStatus } from "@/lib/api";

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
