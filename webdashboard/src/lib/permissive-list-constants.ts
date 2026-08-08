// Permission group icons for the workspace permission management UI.
// Permission IDs and labels are provided by the backend permission catalog.

import type { LucideIcon } from "lucide-react";

import {
  BarChart3,
  Banknote,
  Building2,
  Calendar,
  CalendarDays,
  ClipboardList,
  Clock,
  FileText,
  Monitor,
  ShieldCheck,
  Smartphone,
  UserCheck,
  Users,
  Zap,
} from "lucide-react";

export const permissionGroupIcons: Record<string, LucideIcon> = {
  attendance: Clock,
  departments: Building2,
  documents: FileText,
  employees: UserCheck,
  employment_phases: FileText,
  ess: Smartphone,
  holidays: CalendarDays,
  infra: Monitor,
  leave: Calendar,
  members: Users,
  overtime: Zap,
  payroll: Banknote,
  recruitment: ClipboardList,
  reports: BarChart3,
  security: ShieldCheck,
  shifts: CalendarDays,
  workspace: Building2,
};
