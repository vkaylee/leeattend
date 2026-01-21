import { Routes, Route } from "react-router-dom";
import DashboardPage from "./pages/DashboardPage";
import AgentsPage from "./pages/AgentsPage";
import DevicesPage from "./pages/DevicesPage";
import MembersPage from "./pages/MembersPage";
import ProfilePage from "./pages/ProfilePage";
import LoginPage from "./pages/LoginPage";
import RegisterPage from "./pages/RegisterPage";
import OnboardingPage from "./pages/OnboardingPage";
import EmployeesPage from "./pages/EmployeesPage"; 
import DepartmentsPage from "./pages/DepartmentsPage";
import LeaveRequestsPage from "./pages/LeaveRequestsPage";
import LeaveBalancesPage from "./pages/LeaveBalancesPage";
import ReportsPage from "./pages/ReportsPage";
import PayrollPage from "./pages/PayrollPage";
import ShiftsPage from "./pages/ShiftsPage";
import WorkingPatternsPage from "./pages/WorkingPatternsPage";
import AssignmentRulesPage from "./pages/AssignmentRulesPage";
import SettingsLayout from "./pages/settings/SettingsLayout";
import GeneralSettings from "./pages/settings/GeneralSettings";
import PeopleSettings from "./pages/settings/PeopleSettings";
import TimeSettings from "./pages/settings/TimeSettings";
import PayrollSettings from "./pages/settings/PayrollSettings";
import ConnectivitySettings from "./pages/settings/ConnectivitySettings";
import AuthGuard from "./components/AuthGuard";
import WorkspaceGuard from "./components/WorkspaceGuard";
import AttendanceAdjustmentsPage from "./pages/AttendanceAdjustmentsPage";
import AttendanceLogsPage from "./pages/AttendanceLogsPage";
import MyProfilePage from "./pages/ess/MyProfilePage";
import MyAttendancePage from "./pages/ess/MyAttendancePage";
import MyClockPage from "./pages/ess/MyClockPage";
import MainLayout from "./components/layout/MainLayout";

export function AppRoutes() {
  return (
    <Routes>
      {/* Public routes */}
      <Route path="/login" element={<LoginPage />} />
      <Route path="/register" element={<RegisterPage />} />

      {/* Protected routes */}
      <Route element={<AuthGuard />}>
        <Route path="/onboarding" element={<OnboardingPage />} />
        <Route element={<WorkspaceGuard />}>
          <Route path="/" element={<MainLayout />}>
            <Route index element={<DashboardPage />} />
            <Route path="employees" element={<EmployeesPage />} />
            <Route path="departments" element={<DepartmentsPage />} />
            <Route path="members" element={<MembersPage />} />
            <Route path="agents" element={<AgentsPage />} />
            <Route path="devices" element={<DevicesPage />} />
            <Route path="workspace/reports" element={<ReportsPage />} />
            <Route path="workspace/payroll" element={<PayrollPage />} />
            <Route path="attendance-logs" element={<AttendanceLogsPage />} />
            <Route path="leave-requests" element={<LeaveRequestsPage />} />
            <Route path="leave-balances" element={<LeaveBalancesPage />} />
            <Route path="workspace/leave/balances" element={<LeaveBalancesPage />} />
            <Route path="attendance-adjustments" element={<AttendanceAdjustmentsPage />} />
            <Route path="reports" element={<ReportsPage />} />
            <Route path="shifts" element={<ShiftsPage />} />
            <Route path="working-patterns" element={<WorkingPatternsPage />} />
            <Route path="assignment-rules" element={<AssignmentRulesPage />} />
            <Route path="payroll" element={<PayrollPage />} />
            
            {/* Settings Hub */}
            <Route path="workspace/settings" element={<SettingsLayout />}>
              <Route index element={<GeneralSettings />} />
              <Route path="general" element={<GeneralSettings />} />
              <Route path="people" element={<PeopleSettings />} />
              <Route path="time" element={<TimeSettings />} />
              <Route path="payroll" element={<PayrollSettings />} />
              <Route path="connectivity" element={<ConnectivitySettings />} />
            </Route>

            <Route path="profile" element={<ProfilePage />} />
            {/* ESS Routes */}
            <Route path="/ess/clock" element={<MyClockPage />} />
            <Route path="/ess/profile" element={<MyProfilePage />} />
            <Route path="/ess/attendance" element={<MyAttendancePage />} />
          </Route>
        </Route>
      </Route>
    </Routes>
  );
}
