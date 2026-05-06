import { 
  Palmtree, 
  Stethoscope, 
  UserMinus, 
  Heart, 
  Baby, 
  Users, 
  Gift, 
  GraduationCap, 
  Briefcase, 
  Activity, 
  Gavel, 
  Globe,
  Clock,
  Sparkles,
  type LucideIcon
} from "lucide-react";

export interface LeaveTemplate {
  category: "standard" | "wellness" | "family" | "professional" | "special";
  code: string;
  color: string;
  description: string;
  icon: LucideIcon;
  id: string;
  is_paid: boolean;
  name: string;
}

export const LEAVE_TEMPLATES: LeaveTemplate[] = [
  // --- Standard ---
  {
    id: "annual-leave",
    code: "AL",
    name: "Annual Leave",
    is_paid: true,
    category: "standard",
    icon: Palmtree,
    description: "Standard yearly vacation for employees.",
    color: "blue"
  },
  {
    id: "sick-leave",
    code: "SL",
    name: "Sick Leave",
    is_paid: true,
    category: "standard",
    icon: Stethoscope,
    description: "Paid leave for health issues or medical appointments.",
    color: "emerald"
  },
  {
    id: "unpaid-leave",
    code: "UL",
    name: "Unpaid Leave",
    is_paid: false,
    category: "standard",
    icon: UserMinus,
    description: "Leave without pay for personal reasons.",
    color: "slate"
  },
  {
    id: "compensatory-leave",
    code: "CL",
    name: "Compensatory Leave",
    is_paid: true,
    category: "standard",
    icon: Clock,
    description: "Time off in exchange for overtime worked.",
    color: "indigo"
  },

  // --- Wellness ---
  {
    id: "mental-health",
    code: "MHD",
    name: "Mental Health Day",
    is_paid: true,
    category: "wellness",
    icon: Sparkles,
    description: "A day to recharge and focus on mental well-being.",
    color: "violet"
  },
  {
    id: "period-leave",
    code: "PHL",
    name: "Period Leave",
    is_paid: true,
    category: "wellness",
    icon: Activity,
    description: "Support for employees during their menstrual cycle.",
    color: "rose"
  },
  {
    id: "birthday-leave",
    code: "BDL",
    name: "Birthday Leave",
    is_paid: true,
    category: "wellness",
    icon: Gift,
    description: "Celebrate your special day away from work.",
    color: "amber"
  },

  // --- Family ---
  {
    id: "maternity-leave",
    code: "ML",
    name: "Maternity Leave",
    is_paid: true,
    category: "family",
    icon: Baby,
    description: "Extended leave for new mothers.",
    color: "pink"
  },
  {
    id: "paternity-leave",
    code: "PL",
    name: "Paternity Leave",
    is_paid: true,
    category: "family",
    icon: Users,
    description: "Leave for new fathers to support their family.",
    color: "cyan"
  },
  {
    id: "marriage-leave",
    code: "WL",
    name: "Marriage Leave",
    is_paid: true,
    category: "family",
    icon: Heart,
    description: "Celebrate your wedding with some time off.",
    color: "red"
  },

  // --- Professional ---
  {
    id: "study-leave",
    code: "EDL",
    name: "Study Leave",
    is_paid: true,
    category: "professional",
    icon: GraduationCap,
    description: "Time off for exams or professional development.",
    color: "orange"
  },
  {
    id: "sabbatical-leave",
    code: "SAB",
    name: "Sabbatical",
    is_paid: true,
    category: "professional",
    icon: Globe,
    description: "Long-term break for career refreshment (e.g., after 5 years).",
    color: "teal"
  },

  // --- Special ---
  {
    id: "jury-duty",
    code: "JDL",
    name: "Jury Duty",
    is_paid: true,
    category: "special",
    icon: Gavel,
    description: "Leave for performing civic duties.",
    color: "brown"
  },
  {
    id: "bereavement-leave",
    code: "BL",
    name: "Bereavement Leave",
    is_paid: true,
    category: "special",
    icon: Briefcase,
    description: "Support during the loss of a loved one.",
    color: "stone"
  },
  {
    id: "volunteer-leave",
    code: "VTL",
    name: "Volunteer Leave",
    is_paid: true,
    category: "special",
    icon: Heart,
    description: "Time off to give back to the community.",
    color: "lime"
  }
];
