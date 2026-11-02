import { screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import { Device } from "@/lib/api";
import { renderWithClient } from "@/test/utils";

import DeviceListItem from "../device-list-item";

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));

const mockDevice: Device = {
  id: "dev-1",
  agent_id: "agent-1",
  name: "Office Device",
  ip: "192.168.1.100",
  port: 4370,
  brand: "zkteco",
  is_enabled: true,
  timezone: "UTC",
  last_status: "online",
  max_concurrent_workers: 1,
  created_at: new Date().toISOString(),
  workspace_id: "ws-1",
};

describe("DeviceListItem", () => {
  it("renders static device fields", () => {
    renderWithClient(<ToastProvider><DeviceListItem device={mockDevice} onDelete={vi.fn()} onUpdate={vi.fn()} /></ToastProvider>);
    expect(screen.getByText("Office Device")).toBeDefined();
    expect(screen.getByText("192.168.1.100")).toBeDefined();
  });
});
