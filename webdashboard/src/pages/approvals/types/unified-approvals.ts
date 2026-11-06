export type ApprovalType = "leave" | "overtime" | "adjustment" | "shift_swap";

export type ApprovalItemStatus = "pending" | "approved" | "rejected" | "cancelled";

export interface UnifiedApprovalRequester {
  id: string;
  name: string;
  idNumber?: string;
  avatarUrl?: string;
  departmentId?: string;
  departmentName?: string;
  jobTitle?: string;
}

export interface UnifiedApprovalDetails {
  reason?: string;
  dateStr: string;
  durationLabel?: string;
  shiftInfo?: string;
  notes?: string;
  rawPayload?: Record<string, unknown>;
}

export interface TeamImpactInfo {
  departmentName?: string;
  totalTeamMembers?: number;
  colleaguesAwayCount: number;
  colleaguesAwayNames: string[];
  isHighImpact: boolean; // e.g. > 25% of team absent
}

export interface UnifiedApprovalItem {
  id: string;
  type: ApprovalType;
  status: ApprovalItemStatus;
  requester: UnifiedApprovalRequester;
  submittedAt: string;
  targetDate: string;
  title: string;
  subtitle: string;
  details: UnifiedApprovalDetails;
  impact?: TeamImpactInfo;
}

export interface UnifiedInboxFilter {
  type: ApprovalType | "all";
  status: ApprovalItemStatus | "all";
  departmentId?: string;
  searchQuery?: string;
}

export interface BatchApprovalResult {
  succeeded: string[];
  failed: Array<{ id: string; error: string }>;
}
