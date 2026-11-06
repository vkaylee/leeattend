import React from "react";

export type WidgetCategory =
  | "attendance"
  | "overview"
  | "system"
  | "tasks"
  | "analytics";

export type WidgetColSpan =
  | "col-span-1"
  | "col-span-1 md:col-span-2"
  | "col-span-1 md:col-span-2 lg:col-span-3"
  | "col-span-full";

export interface DesktopWidgetDefinition {
  id: string;
  titleKey?: string;
  titleFallback: string;
  descriptionKey?: string;
  descriptionFallback: string;
  category: WidgetCategory;
  defaultEnabled: boolean;
  defaultOrder: number;
  colSpan: WidgetColSpan;
  icon: React.ElementType;
  component: React.ComponentType;
  permissions?: string[];
}

export interface DesktopLayoutState {
  enabledIds: string[];
  order: string[];
}
