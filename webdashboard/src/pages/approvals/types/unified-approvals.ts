export type ApprovalType = "leave" | "overtime" | "adjustment" | "shift_swap";

export type ApprovalItemStatus = "pending" | "approved" | "rejected" | "cancelled";

export interface UnifiedApprovalRequester {
  avatarUrl?: string;
  departmentId?: string;
  departmentName?: string;
  id: string;
  idNumber?: string;
  jobTitle?: string;
  name: string;
}

export interface UnifiedApprovalDetails {
  dateStr: string;
  durationLabel?: string;
  notes?: string;
  rawPayload?: Record<string, unknown>;
  reason?: string;
  shiftInfo?: string;
}

export interface TeamImpactInfo {
  colleaguesAwayCount: number;
  colleaguesAwayNames: string[];
  departmentName?: string;
  isHighImpact: boolean; // e.g. > 25% of team absent
  totalTeamMembers?: number;
}

export interface UnifiedApprovalItem {
  details: UnifiedApprovalDetails;
  id: string;
  impact?: TeamImpactInfo;
  requester: UnifiedApprovalRequester;
  status: ApprovalItemStatus;
  submittedAt: string;
  subtitle: string;
  targetDate: string;
  title: string;
  type: ApprovalType;
}

export interface UnifiedInboxFilter {
  departmentId?: string;
  searchQuery?: string;
  status: ApprovalItemStatus | "all";
  type: ApprovalType | "all";
}

export interface BatchApprovalResult {
  failed: { id: string; error: string }[];
  succeeded: string[];
}
