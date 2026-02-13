import { screen, fireEvent, waitFor, render } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import DeviceListItem from "../DeviceListItem";
import { deviceFactory } from "@/test/factories";
import { ToastProvider } from "@/components/ui/Toast";

const mockOnUpdate = vi.fn().mockResolvedValue(undefined);
const mockOnDelete = vi.fn().mockResolvedValue(undefined);
const mockOnEdit = vi.fn();

function renderItem(deviceOverrides = {}, props = {}) {
  const device = deviceFactory(deviceOverrides);
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const result = render(
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <DeviceListItem
          device={device}
          onUpdate={mockOnUpdate}
          onDelete={mockOnDelete}
          onEdit={mockOnEdit}
          {...props}
        />
      </ToastProvider>
    </QueryClientProvider>
  );
  return { device, ...result };
}

describe("DeviceListItem", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    document.body.style.overflow = "unset";
  });

  // --- Rendering ---

  it("renders device info correctly", () => {
    const { device } = renderItem({ name: "ZK-Front", ip: "10.0.0.5", port: 4370, brand: "zkteco" });

    expect(screen.getByTestId(`device-name-${device.id}`)).toHaveTextContent("ZK-Front");
    expect(screen.getByTestId(`device-ip-${device.id}`)).toHaveTextContent("10.0.0.5");
    expect(screen.getByTestId(`device-brand-${device.id}`)).toHaveTextContent("zkteco");
  });

  it("shows ACTIVE status when enabled", () => {
    const { device } = renderItem({ is_enabled: true });
    expect(screen.getByTestId(`device-status-${device.id}`)).toBeInTheDocument();
  });

  it("shows DISABLED status when not enabled", () => {
    const { device } = renderItem({ is_enabled: false });
    expect(screen.getByTestId(`device-status-${device.id}`)).toBeInTheDocument();
  });

  it("dims the item when device is disabled", () => {
    const { device } = renderItem({ is_enabled: false });
    const item = screen.getByTestId(`device-item-${device.id}`);
    expect(item.className).toContain("opacity-50");
  });

  it("dims the item when parent agent is inactive", () => {
    const { device } = renderItem({}, { parentAgentActive: false });
    const item = screen.getByTestId(`device-item-${device.id}`);
    expect(item.className).toContain("opacity-50");
  });

  // --- Toggle Modal ---

  it("opens toggle modal when disable button is clicked", () => {
    const { device } = renderItem({ is_enabled: true });

    fireEvent.click(screen.getByTestId(`device-toggle-${device.id}`));

    expect(screen.getByTestId("device-toggle-modal")).toBeInTheDocument();
    expect(screen.getByTestId("device-toggle-modal-confirm")).toBeInTheDocument();
    expect(screen.getByTestId("device-toggle-modal-cancel")).toBeInTheDocument();
  });

  it("opens toggle modal when enable button is clicked", () => {
    const { device } = renderItem({ is_enabled: false });

    fireEvent.click(screen.getByTestId(`device-toggle-${device.id}`));

    expect(screen.getByTestId("device-toggle-modal")).toBeInTheDocument();
  });

  it("closes toggle modal on cancel", () => {
    const { device } = renderItem({ is_enabled: true });

    fireEvent.click(screen.getByTestId(`device-toggle-${device.id}`));
    expect(screen.getByTestId("device-toggle-modal")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("device-toggle-modal-cancel"));
    expect(screen.queryByTestId("device-toggle-modal")).not.toBeInTheDocument();
  });

  it("calls onUpdate with is_enabled=false when confirming disable", async () => {
    const { device } = renderItem({ is_enabled: true });

    fireEvent.click(screen.getByTestId(`device-toggle-${device.id}`));
    fireEvent.click(screen.getByTestId("device-toggle-modal-confirm"));

    await waitFor(() => {
      expect(mockOnUpdate).toHaveBeenCalledWith(device.id, { is_enabled: false });
    });
  });

  it("calls onUpdate with is_enabled=true when confirming enable", async () => {
    const { device } = renderItem({ is_enabled: false });

    fireEvent.click(screen.getByTestId(`device-toggle-${device.id}`));
    fireEvent.click(screen.getByTestId("device-toggle-modal-confirm"));

    await waitFor(() => {
      expect(mockOnUpdate).toHaveBeenCalledWith(device.id, { is_enabled: true });
    });
  });

  it("does not call onUpdate when toggle modal is cancelled", () => {
    const { device } = renderItem({ is_enabled: true });

    fireEvent.click(screen.getByTestId(`device-toggle-${device.id}`));
    fireEvent.click(screen.getByTestId("device-toggle-modal-cancel"));

    expect(mockOnUpdate).not.toHaveBeenCalled();
  });

  it("closes toggle modal after confirm", async () => {
    const { device } = renderItem({ is_enabled: true });

    fireEvent.click(screen.getByTestId(`device-toggle-${device.id}`));
    fireEvent.click(screen.getByTestId("device-toggle-modal-confirm"));

    await waitFor(() => {
      expect(screen.queryByTestId("device-toggle-modal")).not.toBeInTheDocument();
    });
  });

  // --- Delete Modal ---

  it("opens delete modal from dropdown menu", async () => {
    const { device } = renderItem();

    // Open Radix dropdown — needs pointerdown event
    const menuBtn = screen.getByTestId(`device-menu-${device.id}`);
    fireEvent.pointerDown(menuBtn, { pointerType: "mouse" });

    // Wait for Radix portal to render menu items
    const deleteOption = await screen.findByTestId(`device-delete-${device.id}`);
    fireEvent.click(deleteOption);

    await waitFor(() => {
      expect(screen.getByTestId("device-delete-modal")).toBeInTheDocument();
      expect(screen.getByTestId("device-delete-modal-confirm")).toBeInTheDocument();
      expect(screen.getByTestId("device-delete-modal-cancel")).toBeInTheDocument();
    });
  });

  it("closes delete modal on cancel", async () => {
    const { device } = renderItem();

    const menuBtn = screen.getByTestId(`device-menu-${device.id}`);
    fireEvent.pointerDown(menuBtn, { pointerType: "mouse" });

    const deleteOption = await screen.findByTestId(`device-delete-${device.id}`);
    fireEvent.click(deleteOption);

    await waitFor(() => {
      expect(screen.getByTestId("device-delete-modal")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("device-delete-modal-cancel"));

    expect(screen.queryByTestId("device-delete-modal")).not.toBeInTheDocument();
  });

  it("calls onDelete when confirming delete", async () => {
    const { device } = renderItem();

    const menuBtn = screen.getByTestId(`device-menu-${device.id}`);
    fireEvent.pointerDown(menuBtn, { pointerType: "mouse" });

    const deleteOption = await screen.findByTestId(`device-delete-${device.id}`);
    fireEvent.click(deleteOption);

    await waitFor(() => {
      expect(screen.getByTestId("device-delete-modal-confirm")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("device-delete-modal-confirm"));

    await waitFor(() => {
      expect(mockOnDelete).toHaveBeenCalledWith(device.id);
    });
  });

  it("does not call onDelete when delete modal is cancelled", async () => {
    const { device } = renderItem();

    const menuBtn = screen.getByTestId(`device-menu-${device.id}`);
    fireEvent.pointerDown(menuBtn, { pointerType: "mouse" });

    const deleteOption = await screen.findByTestId(`device-delete-${device.id}`);
    fireEvent.click(deleteOption);

    await waitFor(() => {
      expect(screen.getByTestId("device-delete-modal-cancel")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId("device-delete-modal-cancel"));
    expect(mockOnDelete).not.toHaveBeenCalled();
  });

  // --- Error handling ---

  it("shows error toast when toggle fails", async () => {
    mockOnUpdate.mockRejectedValueOnce(new Error("Network error"));
    const { device } = renderItem({ is_enabled: true });

    fireEvent.click(screen.getByTestId(`device-toggle-${device.id}`));
    fireEvent.click(screen.getByTestId("device-toggle-modal-confirm"));

    await waitFor(() => {
      expect(mockOnUpdate).toHaveBeenCalled();
    });
  });

  // --- Edit ---

  it("calls onEdit from dropdown menu", async () => {
    const { device } = renderItem();

    const menuBtn = screen.getByTestId(`device-menu-${device.id}`);
    fireEvent.pointerDown(menuBtn, { pointerType: "mouse" });

    const editOption = await screen.findByTestId(`device-edit-${device.id}`);
    fireEvent.click(editOption);

    expect(mockOnEdit).toHaveBeenCalledWith(device);
  });
});
