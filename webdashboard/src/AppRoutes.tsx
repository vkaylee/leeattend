import React, { Suspense } from "react";
import { Routes, Route } from "react-router-dom";

// Layout & guard components — kept eager (always needed)
import AuthGuard from "./components/common/AuthGuard";
import WorkspaceGuard from "./components/common/WorkspaceGuard";
import MainLayout from "./components/layout/MainLayout";
import InfraLayout from "./components/layout/InfraLayout";

// Lazy-loaded pages — each becomes its own chunk
const LoginPage = React.lazy(() => import("./pages/LoginPage"));
const RegisterPage = React.lazy(() => import("./pages/RegisterPage"));
const OnboardingPage = React.lazy(() => import("./pages/OnboardingPage"));
const DashboardPage = React.lazy(() => import("./pages/DashboardPage"));
const EmployeesPage = React.lazy(() => import("./pages/EmployeesPage"));
const DepartmentsPage = React.lazy(() => import("./pages/DepartmentsPage"));
const MembersPage = React.lazy(() => import("./pages/MembersPage"));
const AgentsPage = React.lazy(() => import("./pages/AgentsPage"));
const DevicesPage = React.lazy(() => import("./pages/DevicesPage"));
const AttendanceLogsPage = React.lazy(() => import("./pages/AttendanceLogsPage"));
const LeaveRequestsPage = React.lazy(() => import("./pages/LeaveRequestsPage"));
const LeaveBalancesPage = React.lazy(() => import("./pages/LeaveBalancesPage"));
const AttendanceAdjustmentsPage = React.lazy(() => import("./pages/AttendanceAdjustmentsPage"));
const ReportsPage = React.lazy(() => import("./pages/ReportsPage"));
const ShiftsPage = React.lazy(() => import("./pages/ShiftsPage"));
const ShiftSwapsPage = React.lazy(() => import("./pages/ShiftSwapsPage"));
const PayrollPage = React.lazy(() => import("./pages/PayrollPage"));
const ProfilePage = React.lazy(() => import("./pages/ProfilePage"));
const WorkerMonitorPage = React.lazy(() => import("./pages/WorkerMonitorPage"));
const WorkingPatternsPage = React.lazy(() => import("./pages/WorkingPatternsPage"));
const AssignmentRulesPage = React.lazy(() => import("./pages/AssignmentRulesPage"));
const OtApprovalsPage = React.lazy(() => import("./pages/approvals/OtApprovalsPage"));
const MyProfilePage = React.lazy(() => import("./pages/ess/MyProfilePage"));
const MyAttendancePage = React.lazy(() => import("./pages/ess/MyAttendancePage"));
const MyClockPage = React.lazy(() => import("./pages/ess/MyClockPage"));

// Settings sub-pages
const SettingsLayout = React.lazy(() => import("./pages/settings/SettingsLayout"));
const GeneralSettings = React.lazy(() => import("./pages/settings/GeneralSettings"));
const PeopleSettings = React.lazy(() => import("./pages/settings/PeopleSettings"));
const TimeSettings = React.lazy(() => import("./pages/settings/TimeSettings"));
const PayrollSettings = React.lazy(() => import("./pages/settings/PayrollSettings"));
const ConnectivitySettings = React.lazy(() => import("./pages/settings/ConnectivitySettings"));
const LeaveSettings = React.lazy(() => import("./pages/settings/LeaveSettings"));
const HolidaysPage = React.lazy(() => import("./pages/settings/HolidaysPage"));

function PageLoader() {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100vh", opacity: 0.5 }}>
      <div style={{ width: 32, height: 32, border: "3px solid currentColor", borderTopColor: "transparent", borderRadius: "50%", animation: "spin 0.8s linear infinite" }} />
      <style>{`@keyframes spin { to { transform: rotate(360deg) } }`}</style>
    </div>
  );
}

export function AppRoutes() {
  return (
    <Suspense fallback={<PageLoader />}>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />

        <Route element={<AuthGuard />}>
          <Route path="/onboarding" element={<OnboardingPage />} />

          {/* Global infrastructure pages (no workspace required) */}
          <Route path="/infra" element={<InfraLayout />}>
            <Route path="monitor" element={<WorkerMonitorPage />} />
          </Route>

          <Route element={<WorkspaceGuard />}>
            <Route path="/" element={<MainLayout />}>
              <Route index element={<DashboardPage />} />
              <Route path="employees" element={<EmployeesPage />} />
              <Route path="departments" element={<DepartmentsPage />} />
              <Route path="members" element={<MembersPage />} />
              <Route path="agents" element={<AgentsPage />} />
              <Route path="devices" element={<DevicesPage />} />
              <Route path="attendance-logs" element={<AttendanceLogsPage />} />
              <Route path="leave-requests" element={<LeaveRequestsPage />} />
              <Route path="leave-balances" element={<LeaveBalancesPage />} />
              <Route path="attendance-adjustments" element={<AttendanceAdjustmentsPage />} />
              <Route path="approvals/ot" element={<OtApprovalsPage />} />
              <Route path="reports" element={<ReportsPage />} />
              <Route path="shifts" element={<ShiftsPage />} />
              <Route path="shift-swaps" element={<ShiftSwapsPage />} />
              <Route path="payroll" element={<PayrollPage />} />

              <Route path="workspace/settings" element={<SettingsLayout />}>
                <Route index element={<GeneralSettings />} />
                <Route path="general" element={<GeneralSettings />} />
                <Route path="people" element={<PeopleSettings />} />
                <Route path="time" element={<TimeSettings />} />
                <Route path="payroll" element={<PayrollSettings />} />
                <Route path="leave" element={<LeaveSettings />} />
                <Route path="holidays" element={<HolidaysPage />} />
                <Route path="connectivity" element={<ConnectivitySettings />} />
              </Route>

              <Route path="profile" element={<ProfilePage />} />
              <Route path="ess">
                <Route path="profile" element={<MyProfilePage />} />
                <Route path="attendance" element={<MyAttendancePage />} />
                <Route path="clock" element={<MyClockPage />} />
              </Route>
            </Route>
          </Route>
        </Route>
      </Routes>
    </Suspense>
  );
}
