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
  category: WidgetCategory;
  colSpan: WidgetColSpan;
  component: React.ComponentType;
  defaultEnabled: boolean;
  defaultOrder: number;
  descriptionFallback: string;
  descriptionKey?: string;
  icon: React.ElementType;
  id: string;
  permissions?: string[];
  titleFallback: string;
  titleKey?: string;
}

export interface DesktopLayoutState {
  enabledIds: string[];
  order: string[];
}
