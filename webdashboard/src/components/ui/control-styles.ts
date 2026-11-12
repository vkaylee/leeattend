// Shared by single-line controls. Textareas keep content-driven height.
export type ControlSize = "sm" | "md" | "lg";

export const CONTROL_SIZE_STYLES = {
  sm: "h-[var(--ui-height-sm)] px-4 text-sm rounded-xl gap-2",
  md: "h-[var(--ui-height-md)] px-6 text-sm rounded-2xl gap-3",
  lg: "h-[var(--ui-height-lg)] px-8 text-base rounded-2xl gap-4",
} as const satisfies Record<ControlSize, string>;

export const CONTROL_TEXT_STYLE = "font-medium normal-case tracking-normal";
