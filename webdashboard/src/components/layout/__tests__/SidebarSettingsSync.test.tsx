import React from "react";
import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";
import Sidebar from "../Sidebar";
import SettingsLayout from "../../../pages/settings/SettingsLayout";

// Mock i18next
vi.mock("react-i18next", () => ({
    useTranslation: () => ({
        t: (key: string) => key,
    }),
}));

describe("Sidebar and SettingsLayout Synchronization", () => {
    it("should have the exact same settings routes in both components", () => {
        // 1. Render Sidebar with the 'system' section open
        const { unmount: unmountSidebar } = render(
            <MemoryRouter>
                <Sidebar
                    isOpen={true}
                    isCollapsed={false}
                    onClose={() => { }}
                    onShowSwitcher={() => { }}
                    openSections={{ system: true }}
                    toggleSection={() => { }}
                />
            </MemoryRouter>
        );

        // Get all links in the sidebar
        const sidebarLinks = screen.getAllByRole("link");

        // Filter for settings routes
        const sidebarSettingsHrefs = sidebarLinks
            .map(link => link.getAttribute("href"))
            .filter(href => href?.startsWith("/workspace/settings/"))
            .sort();

        unmountSidebar();

        // 2. Render SettingsLayout
        const { unmount: unmountSettings } = render(
            <MemoryRouter>
                <SettingsLayout />
            </MemoryRouter>
        );

        // Get all links in the settings layout
        const settingsLinks = screen.getAllByRole("link");

        // Filter for settings routes
        const settingsLayoutHrefs = settingsLinks
            .map(link => link.getAttribute("href"))
            .filter(href => href?.startsWith("/workspace/settings/"))
            .sort();

        unmountSettings();

        // 3. Assert they are exactly the same
        expect(sidebarSettingsHrefs.length).toBeGreaterThan(0);
        expect(settingsLayoutHrefs.length).toBeGreaterThan(0);
        expect(sidebarSettingsHrefs).toEqual(settingsLayoutHrefs);
    });
});
