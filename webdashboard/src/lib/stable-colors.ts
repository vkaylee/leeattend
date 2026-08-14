// Stable identity-keyed colors for chart series.
//
// Requirement: a given entity (department, source, manager, ...) must keep the
// same color across periods, reorders, and data refreshes. Array-index coloring
// fails that contract — when the ranking shifts, every bar changes hue. Instead
// we hash the entity's stable key (UUID or name) into a fixed palette.

export const ENTITY_PALETTE = [
  "#6366f1", // indigo
  "#22c55e", // green
  "#f59e0b", // amber
  "#06b6d4", // cyan
  "#ec4899", // pink
  "#10b981", // emerald
  "#3b82f6", // blue
  "#f97316", // orange
  "#8b5cf6", // violet
  "#14b8a6", // teal
];

/** Deterministic FNV-1a string hash. */
export function hashEntityKey(key: string): number {
  let hash = 0x811c9dc5;
  for (let i = 0; i < key.length; i += 1) {
    hash ^= key.charCodeAt(i);
    hash = Math.imul(hash, 0x01000193);
  }
  return hash >>> 0;
}

/** Map an entity identity to a stable palette color. */
export function colorByEntity(entityKey: string): string {
  return ENTITY_PALETTE[hashEntityKey(entityKey) % ENTITY_PALETTE.length];
}

/** Fall back to a known-type color map before hashing (mirrors recruitment-source-chart). */
export function colorByEntityWithKnown(
  entityKey: string,
  known: Record<string, string>,
): string {
  return known[entityKey] ?? colorByEntity(entityKey);
}
