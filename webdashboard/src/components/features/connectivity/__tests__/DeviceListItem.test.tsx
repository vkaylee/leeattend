import { screen } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/Toast";
import { Device, DeviceStatus } from "@/lib/api";
import { renderWithClient } from "@/test/utils";

import DeviceListItem from "../DeviceListItem";


// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

// Mock useTickingDeviceTime — returns the deviceTime as-is for simplicity in tests
const mockUseTickingDeviceTime = vi.fn();
vi.mock("@/hooks/useTickingDeviceTime", () => ({
  useTickingDeviceTime: (...arguments_: unknown[]) => mockUseTickingDeviceTime(...arguments_),
}));

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
  created_at: new Date().toISOString(),
  workspace_id: "ws-1",
};

const mockStatus: DeviceStatus = {
  device_id: "dev-1",
  status: "online",
  info: {
    users_count: 42,
    logs_count: 1337,
    logs_capacity: 50000,
    device_time: "2023-01-01 10:00:00",
    firmware: "FW 1.2",
    last_sync: "2023-01-01 10:00:01",
  },
};

const LOGS_LABEL = "devices.info.logs_capacity";

describe("DeviceListItem", () => {
  beforeEach(() => {
    // Default: return the deviceTime argument as-is (passthrough)
    mockUseTickingDeviceTime.mockImplementation((dt: string | undefined) => dt ?? null);
  });

  it("renders basic device info", () => {
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
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
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={mockStatus}
        />
      </ToastProvider>
    );
    expect(screen.getByText("2023-01-01 10:00:00")).toBeDefined();
    expect(screen.getByText("42")).toBeDefined();
    expect(screen.getByText("vFW 1.2")).toBeDefined();
  });

  it("renders '--' when status is missing", () => {
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={undefined}
        />
      </ToastProvider>
    );
    const logsValue = screen.getByText(LOGS_LABEL).nextElementSibling?.textContent;
    expect(logsValue).toContain("--");
  });

  it("renders 'N/A' when logs_count is null or invalid", () => {
 
    const statusWithNull: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, logs_count: null as unknown as number }
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusWithNull}
        />
      </ToastProvider>
    );
    const logsValue = screen.getByText(LOGS_LABEL).nextElementSibling?.textContent;
    expect(logsValue).toContain("N/A");
  });

  it("renders '0' when logs_count is exactly 0", () => {
 
    const statusWithZero: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, logs_count: 0 }
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusWithZero}
        />
      </ToastProvider>
    );
    const logsValue = screen.getByText(LOGS_LABEL).nextElementSibling?.textContent;
    expect(logsValue).toContain("0");
  });

  // ===== LOGS/CAPACITY TESTS =====

  it("renders logs count with capacity when both are available", () => {
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={mockStatus}
        />
      </ToastProvider>
    );
    const logsSection = screen.getByText(LOGS_LABEL).nextElementSibling;
    const text = logsSection?.textContent || "";
    // Should show logs_count and capacity with separator
    expect(text).toContain("1,337");
    expect(text).toContain("50,000");
    expect(text).toContain("/");
  });

  it("renders only logs count when logs_capacity is missing", () => {
    const statusNoCapacity: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, logs_capacity: undefined },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusNoCapacity}
        />
      </ToastProvider>
    );
    const logsSection = screen.getByText(LOGS_LABEL).nextElementSibling;
    const text = logsSection?.textContent || "";
    expect(text).toContain("1,337");
    expect(text).not.toContain("/");
  });

  it("renders only logs count when logs_capacity is 0", () => {
    const statusZeroCapacity: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, logs_capacity: 0 },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusZeroCapacity}
        />
      </ToastProvider>
    );
    const logsSection = screen.getByText(LOGS_LABEL).nextElementSibling;
    const text = logsSection?.textContent || "";
    expect(text).toContain("1,337");
    expect(text).not.toContain("/");
  });

  // ===== LAST SYNC TESTS =====

  it("renders last_sync converted to locale string when UTC timestamp provided", () => {
    const statusWithSync: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, last_sync: "2026-02-25T15:43:52" },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusWithSync}
        />
      </ToastProvider>
    );
    // The raw UTC string should NOT appear — it should be converted via toLocaleString()
    const allText = document.body.textContent || "";
    expect(allText).not.toContain("2026-02-25T15:43:52");
  });

  it("renders '—' when last_sync is missing", () => {
    const statusNoSync: DeviceStatus = {
      ...mockStatus,
 
      info: { ...mockStatus.info!, last_sync: undefined as unknown as string },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusNoSync}
        />
      </ToastProvider>
    );
    // With the mock t() which only returns keys, assert the key is rendered
    // (the "—" fallback is inside the i18n interpolation params, not in the key)
    const allText = document.body.textContent || "";
    expect(allText).toContain("devices.info.last_synced");
  });

  it("handles last_sync with Z suffix correctly", () => {
    const statusWithZ: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, last_sync: "2026-02-25T15:43:52Z" },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusWithZ}
        />
      </ToastProvider>
    );
    // Raw UTC string should NOT appear
    const allText = document.body.textContent || "";
    expect(allText).not.toContain("2026-02-25T15:43:52Z");
  });

  it("handles last_sync with timezone offset correctly", () => {
    const statusWithOffset: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, last_sync: "2026-02-25T22:43:52+07:00" },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusWithOffset}
        />
      </ToastProvider>
    );
    // Raw ISO string with offset should NOT appear — should be converted via toLocaleString()
    const allText = document.body.textContent || "";
    expect(allText).not.toContain("2026-02-25T22:43:52+07:00");
  });

  // ===== DEVICE INFO FIELDS TESTS =====

  it("renders serial number from status info", () => {
    const statusWithSerial: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, serial: "BSXQ203760345" },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusWithSerial}
        />
      </ToastProvider>
    );
    expect(screen.getByText("BSXQ203760345")).toBeDefined();
  });

  it("renders MAC address from status info", () => {
    const statusWithMac: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, mac: "AA:BB:CC:DD:EE:FF" },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusWithMac}
        />
      </ToastProvider>
    );
    expect(screen.getByText("AA:BB:CC:DD:EE:FF")).toBeDefined();
  });

  it("renders hw_name from status info", () => {
    const statusWithHw: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, hw_name: "ZK-F22" },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusWithHw}
        />
      </ToastProvider>
    );
    expect(screen.getByText("ZK-F22")).toBeDefined();
  });

  it("renders 'N/A' for serial when not provided", () => {
    const statusNoSerial: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, serial: undefined },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusNoSerial}
        />
      </ToastProvider>
    );
    const serialLabel = screen.getByText("devices.info.serial");
    const serialValue = serialLabel.nextElementSibling?.textContent;
    expect(serialValue).toContain("N/A");
  });

  // ===== USERS/CAPACITY TESTS =====

  it("renders users count with capacity when both available", () => {
    const statusWithCap: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, users_count: 120, users_capacity: 500 },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusWithCap}
        />
      </ToastProvider>
    );
    const usersSection = screen.getByText("devices.info.users_capacity").nextElementSibling;
    const text = usersSection?.textContent || "";
    expect(text).toContain("120");
    expect(text).toContain("500");
    expect(text).toContain("/");
  });

  it("renders only users count when users_capacity is missing", () => {
    const statusNoCap: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, users_count: 42, users_capacity: undefined },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusNoCap}
        />
      </ToastProvider>
    );
    const usersSection = screen.getByText("devices.info.users_capacity").nextElementSibling;
    const text = usersSection?.textContent || "";
    expect(text).toContain("42");
    expect(text).not.toContain("/");
  });

  // ===== DEVICE TIME TESTS =====

  it("renders ticking device_time when provided", () => {
    mockUseTickingDeviceTime.mockReturnValue("2026-02-27 14:30:05");
    const statusWithTime: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, device_time: "2026-02-27 14:30:00" },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusWithTime}
        />
      </ToastProvider>
    );
    expect(screen.getByText("2026-02-27 14:30:05")).toBeDefined();
  });

  it("renders '--:--:--' when device_time is missing", () => {
    mockUseTickingDeviceTime.mockReturnValue(null);
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={undefined}
        />
      </ToastProvider>
    );
    expect(screen.getByText("--:--:--")).toBeDefined();
  });

  // ===== FIRMWARE FALLBACK TESTS =====

  it("renders 'v?.?' when firmware is missing", () => {
    const statusNoFw: DeviceStatus = {
      ...mockStatus,
      info: { ...mockStatus.info!, firmware: undefined },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={statusNoFw}
        />
      </ToastProvider>
    );
    expect(screen.getByText("v?.?")).toBeDefined();
  });

  // ===== DIMMING BEHAVIOR TESTS =====

  it("applies dimmed styling when device is disabled", () => {
    const disabledDevice = { ...mockDevice, is_enabled: false };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={disabledDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          status={mockStatus}
        />
      </ToastProvider>
    );
    const deviceItem = screen.getByTestId("device-item-dev-1");
    expect(deviceItem.className).toContain("opacity-60");
  });

  it("applies dimmed styling when parent agent is not active", () => {
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onDelete={async () => { }}
          onUpdate={async () => { }}
          parentAgentActive={false}
          status={mockStatus}
        />
      </ToastProvider>
    );
    const deviceItem = screen.getByTestId("device-item-dev-1");
    expect(deviceItem.className).toContain("opacity-60");
  });
});
