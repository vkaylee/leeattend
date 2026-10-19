package api

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockDashboardAnalyticsRepo struct {
	tz                 string
	executiveOverview  DashboardMetricsResponse
	overtimeBreakdown  OvertimeBreakdownResponse
	workspaceStats     WorkspaceStats
	attendanceTrend    []AttendanceTrendPoint
	deptAttendance     []DepartmentAttendancePoint
	unexcusedAbsences  []UnexcusedAbsencesPoint
	overtimeTrend      []OvertimeTrendPoint
	peakTimes          []PeakTimeSlot
	leaveTypes         []LeaveTypeBreakdown
	latenessByDay      []LatenessByDay
	turnoverAnalytics  TurnoverAnalytics
	recruitmentEffect  RecruitmentEffectivenessAnalytics
	lastAllowCost      bool
	lastExecutiveQuery NormalizedDashboardMetricsQuery
	lastBreakdownQuery OvertimeBreakdownQuery
	lastTurnoverQuery  TurnoverAnalyticsQuery
	lastRecruitQuery   RecruitmentEffectivenessQuery
}

func (m *mockDashboardAnalyticsRepo) GetWorkspaceTimezone(ctx context.Context, wsID uuid.UUID) (string, error) {
	if m.tz != "" {
		return m.tz, nil
	}
	return "UTC", nil
}

func (m *mockDashboardAnalyticsRepo) GetExecutiveOverview(ctx context.Context, wsID uuid.UUID, q NormalizedDashboardMetricsQuery, allowOvertimeCost bool) (DashboardMetricsResponse, error) {
	m.lastExecutiveQuery = q
	m.lastAllowCost = allowOvertimeCost
	return m.executiveOverview, nil
}

func (m *mockDashboardAnalyticsRepo) GetOvertimeBreakdown(ctx context.Context, wsID uuid.UUID, q OvertimeBreakdownQuery, allowOvertimeCost bool) (OvertimeBreakdownResponse, error) {
	m.lastBreakdownQuery = q
	m.lastAllowCost = allowOvertimeCost
	return m.overtimeBreakdown, nil
}

func (m *mockDashboardAnalyticsRepo) GetWorkspaceStats(ctx context.Context, wsID uuid.UUID, tzName string, deptIDs []uuid.UUID) (WorkspaceStats, error) {
	return m.workspaceStats, nil
}

func (m *mockDashboardAnalyticsRepo) GetAttendanceTrend(ctx context.Context, wsID uuid.UUID, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID) ([]AttendanceTrendPoint, error) {
	return m.attendanceTrend, nil
}

func (m *mockDashboardAnalyticsRepo) GetDepartmentAttendance(ctx context.Context, wsID uuid.UUID, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID, topN *int) ([]DepartmentAttendancePoint, error) {
	return m.deptAttendance, nil
}

func (m *mockDashboardAnalyticsRepo) GetUnexcusedAbsences(ctx context.Context, wsID uuid.UUID, startDate, endDate time.Time, tzName string, groupBy string, deptIDs []uuid.UUID) ([]UnexcusedAbsencesPoint, error) {
	return m.unexcusedAbsences, nil
}

func (m *mockDashboardAnalyticsRepo) GetOvertimeTrend(ctx context.Context, wsID uuid.UUID, today time.Time) ([]OvertimeTrendPoint, error) {
	return m.overtimeTrend, nil
}

func (m *mockDashboardAnalyticsRepo) GetPeakTimes(ctx context.Context, wsID uuid.UUID, today time.Time, tzName string) ([]PeakTimeSlot, error) {
	return m.peakTimes, nil
}

func (m *mockDashboardAnalyticsRepo) GetLeaveTypesBreakdown(ctx context.Context, wsID uuid.UUID, today time.Time) ([]LeaveTypeBreakdown, error) {
	return m.leaveTypes, nil
}

func (m *mockDashboardAnalyticsRepo) GetLatenessByDay(ctx context.Context, wsID uuid.UUID, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID) ([]LatenessByDay, error) {
	return m.latenessByDay, nil
}

func (m *mockDashboardAnalyticsRepo) GetTurnoverAnalytics(ctx context.Context, wsID uuid.UUID, q TurnoverAnalyticsQuery) (TurnoverAnalytics, error) {
	m.lastTurnoverQuery = q
	return m.turnoverAnalytics, nil
}

func (m *mockDashboardAnalyticsRepo) GetRecruitmentEffectiveness(ctx context.Context, wsID uuid.UUID, q RecruitmentEffectivenessQuery) (RecruitmentEffectivenessAnalytics, error) {
	m.lastRecruitQuery = q
	return m.recruitmentEffect, nil
}

type mockDashboardAuthz struct {
	permissions  map[string]bool
	isGlobal     bool
	managedDepts []uuid.UUID
}

func (a mockDashboardAuthz) HasPermission(ctx context.Context, userID, workspaceID uuid.UUID, permission string) bool {
	return a.permissions[permission]
}

func (a mockDashboardAuthz) GetManagedDepartmentIDs(ctx context.Context, userID, workspaceID uuid.UUID) ([]uuid.UUID, bool, error) {
	return a.managedDepts, a.isGlobal, nil
}

func testContext(userID, wsID uuid.UUID) context.Context {
	ctx := context.Background()
	ctx = httpinfra.WithUserIDForTest(ctx, userID)
	ctx = httpinfra.WithWorkspaceIDForTest(ctx, wsID)
	return ctx
}

func TestDashboardAnalytics_ForbiddenWithoutBasePermission(t *testing.T) {
	userID := uuid.New()
	wsID := uuid.New()
	ctx := testContext(userID, wsID)

	h := DashboardAnalyticsHandler{
		Repository: &mockDashboardAnalyticsRepo{},
		Authorizer: mockDashboardAuthz{permissions: map[string]bool{}},
	}

	endpoints := []struct {
		name    string
		path    string
		handler func(http.ResponseWriter, *http.Request)
	}{
		{"ExecutiveOverview", "/api/v1/dashboard/executive-overview", h.ExecutiveOverview},
		{"OvertimeBreakdown", "/api/v1/dashboard/overtime-breakdown", h.OvertimeBreakdown},
		{"WorkspaceStats", "/api/v1/dashboard/workspace-stats", h.WorkspaceStats},
		{"AttendanceTrend", "/api/v1/dashboard/attendance-trend", h.AttendanceTrend},
		{"DepartmentAttendance", "/api/v1/dashboard/department-attendance", h.DepartmentAttendance},
		{"UnexcusedAbsences", "/api/v1/dashboard/unexcused-absences", h.UnexcusedAbsences},
		{"OvertimeTrend", "/api/v1/dashboard/overtime-trend", h.OvertimeTrend},
		{"PeakTimes", "/api/v1/dashboard/peak-times", h.PeakTimes},
		{"LeaveTypes", "/api/v1/dashboard/leave-types", h.LeaveTypes},
		{"LatenessByDay", "/api/v1/dashboard/lateness-by-day", h.LatenessByDay},
		{"TurnoverAnalytics", "/api/v1/dashboard/turnover-analytics", h.TurnoverAnalytics},
		{"RecruitmentEffectiveness", "/api/v1/dashboard/recruitment-effectiveness", h.RecruitmentEffectiveness},
	}

	for _, ep := range endpoints {
		t.Run(ep.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, ep.path, nil).WithContext(ctx)
			rec := httptest.NewRecorder()
			ep.handler(rec, req)
			if rec.Code != http.StatusForbidden {
				t.Fatalf("expected 403 Forbidden for %s, got %d", ep.name, rec.Code)
			}
		})
	}
}

func TestDashboardAnalytics_UnexcusedAbsencesPermissions(t *testing.T) {
	userID := uuid.New()
	wsID := uuid.New()
	ctx := testContext(userID, wsID)

	repo := &mockDashboardAnalyticsRepo{
		unexcusedAbsences: []UnexcusedAbsencesPoint{
			{DepartmentID: uuid.New(), DepartmentName: "Logistics", UnexcusedCount: 5},
		},
	}

	// 1. group_by=department with only dashboard:read -> 200 OK
	h := DashboardAnalyticsHandler{
		Repository: repo,
		Authorizer: mockDashboardAuthz{
			permissions: map[string]bool{string(auth.PermissionDashboardRead): true},
			isGlobal:    true,
		},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/unexcused-absences?group_by=department", nil).WithContext(ctx)
	rec := httptest.NewRecorder()
	h.UnexcusedAbsences(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for group_by=department, got %d", rec.Code)
	}

	// 2. group_by=employee WITHOUT dashboard:employee_absence:read -> 403 Forbidden
	req = httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/unexcused-absences?group_by=employee", nil).WithContext(ctx)
	rec = httptest.NewRecorder()
	h.UnexcusedAbsences(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden for employee absence without permission, got %d", rec.Code)
	}

	// 3. group_by=employee WITH dashboard:employee_absence:read -> 200 OK
	h.Authorizer = mockDashboardAuthz{
		permissions: map[string]bool{
			string(auth.PermissionDashboardRead):                true,
			string(auth.PermissionDashboardEmployeeAbsenceRead): true,
		},
		isGlobal: true,
	}
	req = httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/unexcused-absences?group_by=employee", nil).WithContext(ctx)
	rec = httptest.NewRecorder()
	h.UnexcusedAbsences(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for employee absence with permission, got %d", rec.Code)
	}

	// 4. invalid group_by -> 400 Bad Request
	req = httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/unexcused-absences?group_by=invalid", nil).WithContext(ctx)
	rec = httptest.NewRecorder()
	h.UnexcusedAbsences(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 Validation for invalid group_by, got %d", rec.Code)
	}
}

func TestDashboardAnalytics_ExecutiveOverview(t *testing.T) {
	userID := uuid.New()
	wsID := uuid.New()
	ctx := testContext(userID, wsID)

	repo := &mockDashboardAnalyticsRepo{
		executiveOverview: DashboardMetricsResponse{
			ContractVersion: "executive-dashboard.v1",
			Workspace:       DashboardWorkspaceContext{Timezone: "UTC", Currency: "USD"},
			Context: DashboardMetricContext{
				Period:  DashboardReportingPeriod{Key: "today"},
				Filters: map[string]string{},
				Scope:   DashboardMetricScope{Kind: "workspace"},
			},
			Metrics:     []DashboardMetric{},
			Insights:    []DashboardInsight{},
			Actions:     DashboardActionsSummary{Total: 2, ByType: []DashboardActionTypeCount{}},
			DataQuality: DashboardDataQuality{Status: "complete", Sources: []DashboardQualitySource{}},
		},
	}

	h := DashboardAnalyticsHandler{
		Repository: repo,
		Authorizer: mockDashboardAuthz{
			permissions: map[string]bool{string(auth.PermissionDashboardRead): true},
			isGlobal:    true,
		},
	}

	// 1. Basic query without payroll:sensitive:read
	req := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/executive-overview?period=today", nil).WithContext(ctx)
	rec := httptest.NewRecorder()
	h.ExecutiveOverview(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.lastAllowCost {
		t.Fatalf("expected allowOvertimeCost to be false")
	}

	// 2. With payroll:sensitive:read
	h.Authorizer = mockDashboardAuthz{
		permissions: map[string]bool{
			string(auth.PermissionDashboardRead):        true,
			string(auth.PermissionPayrollSensitiveRead): true,
		},
		isGlobal: true,
	}
	rec = httptest.NewRecorder()
	h.ExecutiveOverview(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	if !repo.lastAllowCost {
		t.Fatalf("expected allowOvertimeCost to be true")
	}

	// 3. Invalid period
	badReq := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/executive-overview?period=invalid", nil).WithContext(ctx)
	rec = httptest.NewRecorder()
	h.ExecutiveOverview(rec, badReq)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid period, got %d", rec.Code)
	}

	// 4. Custom period missing dates
	customMissingReq := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/executive-overview?period=custom", nil).WithContext(ctx)
	rec = httptest.NewRecorder()
	h.ExecutiveOverview(rec, customMissingReq)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for custom period missing dates, got %d", rec.Code)
	}

	// 5. Non-custom period with explicit dates rejected
	forbiddenDatesReq := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/executive-overview?period=today&start_date=2026-01-01", nil).WithContext(ctx)
	rec = httptest.NewRecorder()
	h.ExecutiveOverview(rec, forbiddenDatesReq)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for non-custom period with start_date, got %d", rec.Code)
	}
}

func TestDashboardAnalytics_OvertimeBreakdown(t *testing.T) {
	userID := uuid.New()
	wsID := uuid.New()
	ctx := testContext(userID, wsID)

	repo := &mockDashboardAnalyticsRepo{
		overtimeBreakdown: OvertimeBreakdownResponse{
			ContractVersion: "overtime-breakdown.v1",
			Workspace:       OvertimeBreakdownWorkspace{Timezone: "UTC", Currency: "USD"},
			Rows: []OvertimeBreakdownRow{
				{DepartmentID: uuid.New(), DepartmentName: "Eng", SystemOTHours: 10, ApprovedOTHours: 8, CostAvailable: false},
			},
		},
	}

	h := DashboardAnalyticsHandler{
		Repository: repo,
		Authorizer: mockDashboardAuthz{
			permissions: map[string]bool{string(auth.PermissionDashboardRead): true},
			isGlobal:    true,
		},
	}

	// 1. Default date range
	req := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/overtime-breakdown", nil).WithContext(ctx)
	rec := httptest.NewRecorder()
	h.OvertimeBreakdown(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}

	// 2. Only start_date provided -> 400 Validation
	badReq := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/overtime-breakdown?start_date=2026-01-01", nil).WithContext(ctx)
	rec = httptest.NewRecorder()
	h.OvertimeBreakdown(rec, badReq)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for partial date range, got %d", rec.Code)
	}
}

func TestDashboardAnalytics_DepartmentScoping(t *testing.T) {
	userID := uuid.New()
	wsID := uuid.New()
	ctx := testContext(userID, wsID)

	managedDept := uuid.New()
	foreignDept := uuid.New()

	repo := &mockDashboardAnalyticsRepo{}
	h := DashboardAnalyticsHandler{
		Repository: repo,
		Authorizer: mockDashboardAuthz{
			permissions:  map[string]bool{string(auth.PermissionDashboardRead): true},
			isGlobal:     false,
			managedDepts: []uuid.UUID{managedDept},
		},
	}

	// 1. Requesting foreign department returns 403 Forbidden
	req := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/executive-overview?period=today&department_ids="+foreignDept.String(), nil).WithContext(ctx)
	rec := httptest.NewRecorder()
	h.ExecutiveOverview(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden for foreign department, got %d", rec.Code)
	}

	// 2. Requesting managed department returns 200 OK
	req = httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/executive-overview?period=today&department_ids="+managedDept.String(), nil).WithContext(ctx)
	rec = httptest.NewRecorder()
	h.ExecutiveOverview(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for managed department, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestDashboardAnalytics_AllRemainingEndpoints(t *testing.T) {
	userID := uuid.New()
	wsID := uuid.New()
	ctx := testContext(userID, wsID)

	repo := &mockDashboardAnalyticsRepo{
		workspaceStats: WorkspaceStats{
			TotalEmployees: 50,
			AttendanceRate: NewRateQuantity(45, 50, "employees"),
		},
		attendanceTrend: []AttendanceTrendPoint{
			{Date: "2026-10-01", Present: 40, AttendanceRate: NewRateQuantity(40, 50, "employee_days")},
		},
		deptAttendance: []DepartmentAttendancePoint{
			{DepartmentID: uuid.New(), DepartmentName: "Sales", TotalEmployees: 10, AttendanceRate: NewRateQuantity(10, 10, "employees")},
		},
		overtimeTrend: []OvertimeTrendPoint{
			{Date: "2026-10-01", SystemOTHours: 5.0, ApprovedOTHours: 4.0},
		},
		peakTimes: []PeakTimeSlot{
			{TimeSlot: "08:00-08:30", CheckIn: 10, CheckOut: 0},
		},
		leaveTypes: []LeaveTypeBreakdown{
			{LeaveType: "Sick", Count: 3},
		},
		latenessByDay: []LatenessByDay{
			{Day: "Mon", LateCount: 2, AvgLateMinutes: 10.5},
		},
		turnoverAnalytics: TurnoverAnalytics{
			PeriodStart:  "2026-04-20",
			PeriodEnd:    "2026-10-20",
			TurnoverRate: NewTurnoverRate(2, 50),
			Target:       TurnoverTarget{Value: decimal.NewFromInt(15), Operator: "lte"},
		},
		recruitmentEffect: RecruitmentEffectivenessAnalytics{
			PeriodStart: "2026-04-20",
			PeriodEnd:   "2026-10-20",
			TotalHires:  5,
			Funnel:      DefaultRecruitmentFunnel(5, 4),
			DataGaps:    DefaultRecruitmentDataGaps(),
		},
	}

	h := DashboardAnalyticsHandler{
		Repository: repo,
		Authorizer: mockDashboardAuthz{
			permissions: map[string]bool{string(auth.PermissionDashboardRead): true},
			isGlobal:    true,
		},
	}

	tests := []struct {
		name    string
		path    string
		handler func(http.ResponseWriter, *http.Request)
	}{
		{"WorkspaceStats", "/api/v1/dashboard/workspace-stats", h.WorkspaceStats},
		{"AttendanceTrend", "/api/v1/dashboard/attendance-trend", h.AttendanceTrend},
		{"DepartmentAttendance", "/api/v1/dashboard/department-attendance?range=today", h.DepartmentAttendance},
		{"OvertimeTrend", "/api/v1/dashboard/overtime-trend", h.OvertimeTrend},
		{"PeakTimes", "/api/v1/dashboard/peak-times", h.PeakTimes},
		{"LeaveTypes", "/api/v1/dashboard/leave-types", h.LeaveTypes},
		{"LatenessByDay", "/api/v1/dashboard/lateness-by-day", h.LatenessByDay},
		{"TurnoverAnalytics", "/api/v1/dashboard/turnover-analytics", h.TurnoverAnalytics},
		{"RecruitmentEffectiveness", "/api/v1/dashboard/recruitment-effectiveness", h.RecruitmentEffectiveness},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, tt.path, nil).WithContext(ctx)
			rec := httptest.NewRecorder()
			tt.handler(rec, req)
			if rec.Code != http.StatusOK {
				t.Fatalf("expected 200 OK for %s, got %d: %s", tt.name, rec.Code, rec.Body.String())
			}
		})
	}

	// Test turnover invalid comparison
	badTurnover := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/turnover-analytics?comparison=invalid", nil).WithContext(ctx)
	rec := httptest.NewRecorder()
	h.TurnoverAnalytics(rec, badTurnover)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid turnover comparison, got %d", rec.Code)
	}

	// Test recruitment invalid comparison
	badRecruit := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/recruitment-effectiveness?comparison=invalid", nil).WithContext(ctx)
	rec = httptest.NewRecorder()
	h.RecruitmentEffectiveness(rec, badRecruit)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid recruitment comparison, got %d", rec.Code)
	}
}

func TestRateQuantity_ZeroDenominator(t *testing.T) {
	rate := NewRateQuantity(10, 0, "employees")
	if rate.Value != nil {
		t.Fatalf("expected nil value for zero denominator, got %v", rate.Value)
	}
	if rate.Numerator != 10 || rate.Denominator != 0 {
		t.Fatalf("unexpected quantities: %+v", rate)
	}

	data, err := json.Marshal(rate)
	if err != nil {
		t.Fatalf("marshal error: %v", err)
	}
	var decoded map[string]any
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}
	if decoded["value"] != nil {
		t.Fatalf("expected value to be null in JSON, got %v", decoded["value"])
	}
}
