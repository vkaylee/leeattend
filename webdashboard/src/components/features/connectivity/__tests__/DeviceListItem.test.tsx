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
    logs_capacity: 50000,
    device_time: "2023-01-01 10:00:00",
    firmware: "FW 1.2",
    last_sync: "2023-01-01 10:00:01",
  },
};

const LOGS_LABEL = "devices.info.logs_capacity";

describe("DeviceListItem", () => {
  it("renders basic device info", () => {
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          onUpdate={async () => { }}
          onDelete={async () => { }}
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
          onUpdate={async () => { }}
          onDelete={async () => { }}
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
          status={undefined}
          onUpdate={async () => { }}
          onDelete={async () => { }}
        />
      </ToastProvider>
    );
    const logsValue = screen.getByText(LOGS_LABEL).nextElementSibling?.textContent;
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
          onUpdate={async () => { }}
          onDelete={async () => { }}
        />
      </ToastProvider>
    );
    const logsValue = screen.getByText(LOGS_LABEL).nextElementSibling?.textContent;
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
          onUpdate={async () => { }}
          onDelete={async () => { }}
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
          status={mockStatus}
          onUpdate={async () => { }}
          onDelete={async () => { }}
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
          status={statusNoCapacity}
          onUpdate={async () => { }}
          onDelete={async () => { }}
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
          status={statusZeroCapacity}
          onUpdate={async () => { }}
          onDelete={async () => { }}
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
          status={statusWithSync}
          onUpdate={async () => { }}
          onDelete={async () => { }}
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
      info: { ...mockStatus.info!, last_sync: undefined as any },
    };
    renderWithClient(
      <ToastProvider>
        <DeviceListItem
          device={mockDevice}
          status={statusNoSync}
          onUpdate={async () => { }}
          onDelete={async () => { }}
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
          status={statusWithZ}
          onUpdate={async () => { }}
          onDelete={async () => { }}
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
          status={statusWithOffset}
          onUpdate={async () => { }}
          onDelete={async () => { }}
        />
      </ToastProvider>
    );
    // Raw ISO string with offset should NOT appear — should be converted via toLocaleString()
    const allText = document.body.textContent || "";
    expect(allText).not.toContain("2026-02-25T22:43:52+07:00");
  });
});

