import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, expect, it, vi } from "vitest";

import { PunchGeofenceStatus } from "../punch-geofence-status";
import { Shift24hTimeline } from "../shift-24h-timeline";
import { TodayHoursSummary } from "../today-hours-summary";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, defaultString?: string) => defaultString ?? key,
  }),
}));

describe("Shift24hTimeline Component", () => {
  it("renders the 24h timeline container with scheduled shift and break boxes", () => {
    render(
      <Shift24hTimeline
        scheduledShift={{
          startHour: 8.5,
          endHour: 17.5,
          breakStartHour: 12.0,
          breakEndHour: 13.0,
          label: "Ca hành chính",
        }}
      />
    );

    expect(screen.getByTestId("shift-24h-timeline")).toBeInTheDocument();
    expect(screen.getByTestId("scheduled-shift-box")).toBeInTheDocument();
    expect(screen.getByTestId("break-window-box")).toBeInTheDocument();
    expect(screen.getByTestId("now-scrubber-needle")).toBeInTheDocument();

    // Verify 24h hour marks
    expect(screen.getByText("00:00")).toBeInTheDocument();
    expect(screen.getByTestId("timeline-hour-axis")).toHaveTextContent("12:00");
    expect(screen.getByText("24:00")).toBeInTheDocument();
  });

  it("renders punch in dot, punch out dot, and worked bar for actual punches", () => {
    const today = new Date().toISOString().split("T")[0];
    const punches = [
      {
        punchInTime: `${today}T08:30:00.000Z`,
        punchOutTime: `${today}T17:30:00.000Z`,
      },
    ];

    render(<Shift24hTimeline actualPunches={punches} />);

    expect(screen.getByTestId("worked-bar-0")).toBeInTheDocument();
    expect(screen.getByTestId("punch-in-dot-0")).toBeInTheDocument();
    expect(screen.getByTestId("punch-out-dot-0")).toBeInTheDocument();
  });
});

describe("PunchGeofenceStatus Component", () => {
  it("renders valid Wi-Fi and GPS geofence badges", () => {
    render(
      <PunchGeofenceStatus
        gpsRadiusMeters={25}
        isGpsValid={true}
        isWifiValid={true}
        locationName="Văn phòng Landmark 81"
        wifiSsid="LeeCorp-5G"
      />
    );

    expect(screen.getByTestId("punch-geofence-status")).toBeInTheDocument();
    expect(screen.getByTestId("wifi-status-badge")).toHaveTextContent("LeeCorp-5G");
    expect(screen.getByTestId("gps-status-badge")).toHaveTextContent("Văn phòng Landmark 81 (~25m)");
  });

  it("handles out of network wifi status gracefully", () => {
    render(
      <PunchGeofenceStatus
        isGpsValid={false}
        isWifiValid={false}
        wifiSsid="Coffee-Shop-Free"
      />
    );

    const wifiBadge = screen.getByTestId("wifi-status-badge");
    expect(wifiBadge).toHaveClass("bg-rose-50");
  });
});

describe("TodayHoursSummary Component", () => {
  it("renders correct progress bar and metric cards when below 8 hours", () => {
    render(
      <TodayHoursSummary
        actualMinutes={240} // 4h
        breakMinutes={60}
        otMinutes={0}
        standardMinutes={480} // 8h
      />
    );

    expect(screen.getByTestId("today-hours-summary")).toBeInTheDocument();
    expect(screen.getByTestId("today-progress-percent")).toHaveTextContent("50%");
    expect(screen.getByText("4h")).toBeInTheDocument(); // Actual
    expect(screen.getByText("8h")).toBeInTheDocument(); // Standard
    expect(screen.getByText("1h")).toBeInTheDocument(); // Break
  });

  it("renders 100% progress and congratulations when target reached", () => {
    render(
      <TodayHoursSummary
        actualMinutes={500}
        breakMinutes={60}
        otMinutes={20}
        standardMinutes={480}
      />
    );

    expect(screen.getByTestId("today-progress-percent")).toHaveTextContent("100%");
    expect(screen.getByText(/Chúc mừng! Bạn đã hoàn thành/i)).toBeInTheDocument();
  });
});
