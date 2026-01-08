export function isAgentOnline(lastSeenAt: string | null | undefined): boolean {
  if (!lastSeenAt) return false;
  const lastSeen = new Date(lastSeenAt).getTime();
  const now = new Date().getTime();
  // Threshold is 90 seconds
  return now - lastSeen < 90000;
}
