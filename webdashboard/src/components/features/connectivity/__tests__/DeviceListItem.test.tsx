import { screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import DeviceListItem from "../DeviceListItem";
import { Device, DeviceStatus } from "@/lib/api";
import { renderWithClient } from "@/test/utils";
import { ToastProvider } from "@/components/ui/Toast";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

const mockDevice: Device = {
  id: "dev-1",
  agent_id: "agent-1",
  name: "Office Device",
  ip: "192.168.1.100",
  port: 4370,
  brand: "zkteco",
  is_enabled: true,
  last_status: "online",
  created_at: new Date().toISOString(),
  workspace_id: "ws-1",
};

const mockStatus: DeviceStatus = {
  device_id: "dev-1",
  status: "online",
  info: {
    users_count: 42,
    logs_count: 1337,
    device_time: "2023-01-01 10:00:00",
    firmware: "FW 1.2",
    last_sync: "2023-01-01 10:00:01",
  },
};

describe("DeviceListItem", () => {
  it("renders basic device info", () => {
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onUpdate={async () => {}}
          onDelete={async () => {}}
        />
      </ToastProvider>
    );
    expect(screen.getByText("Office Device")).toBeDefined();
    expect(screen.getByText("192.168.1.100")).toBeDefined();
  });

  it("renders metadata when status info is provided", () => {
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          status={mockStatus}
          onUpdate={async () => {}}
          onDelete={async () => {}}
        />
      </ToastProvider>
    );
    expect(screen.getByText("2023-01-01 10:00:00")).toBeDefined();
    expect(screen.getByText("42")).toBeDefined();
    expect(screen.getByText("1337")).toBeDefined();
    expect(screen.getByText("vFW 1.2")).toBeDefined();
  });

  it("renders '--' when status is missing", () => {
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          status={undefined}
          onUpdate={async () => {}}
          onDelete={async () => {}}
        />
      </ToastProvider>
    );
    // Find by the exact text "--"
    const logsValue = screen.getByText("Logs").nextElementSibling?.textContent;
    expect(logsValue).toContain("--");
  });

  it("renders 'N/A' when logs_count is null or invalid", () => {
    const statusWithNull: any = {
      ...mockStatus,
      info: { ...mockStatus.info, logs_count: null }
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          status={statusWithNull}
          onUpdate={async () => {}}
          onDelete={async () => {}}
        />
      </ToastProvider>
    );
    // Check text inside the Logs section specifically
    const logsValue = screen.getByText("Logs").nextElementSibling?.textContent;
    expect(logsValue).toContain("N/A");
  });

  it("renders '0' when logs_count is exactly 0", () => {
    const statusWithZero: any = {
      ...mockStatus,
      info: { ...mockStatus.info, logs_count: 0 }
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          status={statusWithZero}
          onUpdate={async () => {}}
          onDelete={async () => {}}
        />
      </ToastProvider>
    );
    const logsValue = screen.getByText("Logs").nextElementSibling?.textContent;
    expect(logsValue).toContain("0");
  });
});
