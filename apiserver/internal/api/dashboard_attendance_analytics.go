package api

import (
	"context"
	"net/http"
	"strconv"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
)

type AttendanceTrendPoint struct {
	Date           string       `json:"date"`
	Present        int64        `json:"present"`
	Late           int64        `json:"late"`
	Absent         int64        `json:"absent"`
	OnLeave        int64        `json:"on_leave"`
	Eligible       int64        `json:"eligible"`
	AttendanceRate RateQuantity `json:"attendance_rate"`
	AbsenceRate    RateQuantity `json:"absence_rate"`
	LateRate       RateQuantity `json:"late_rate"`
}

// DepartmentAttendancePoint represents department-scoped attendance rate stats.
type DepartmentAttendancePoint struct {
	DepartmentID   uuid.UUID    `json:"department_id"`
	DepartmentName string       `json:"department_name"`
	OnTime         int64        `json:"on_time"`
	Late           int64        `json:"late"`
	Absent         int64        `json:"absent"`
	OnLeave        int64        `json:"on_leave"`
	TotalEmployees int64        `json:"total_employees"`
	AttendanceRate RateQuantity `json:"attendance_rate"`
	AbsenceRate    RateQuantity `json:"absence_rate"`
	LateRate       RateQuantity `json:"late_rate"`
}

// UnexcusedAbsencesPoint represents absence stats grouped by department or employee.
type UnexcusedAbsencesPoint struct {
	DepartmentID     uuid.UUID  `json:"department_id"`
	DepartmentName   string     `json:"department_name"`
	EmployeeID       *uuid.UUID `json:"employee_id,omitempty"`
	EmployeeName     *string    `json:"employee_name,omitempty"`
	UnexcusedCount   int64      `json:"unexcused_count"`
	ConsecutiveCount int64      `json:"consecutive_count"`
	TotalAbsentCount int64      `json:"total_absent_count"`
	TotalLeaveCount  int64      `json:"total_leave_count"`
}

// OvertimeTrendPoint represents daily overtime hours in a trend chart.
type OvertimeTrendPoint struct {
	Date            string  `json:"date"`
	SystemOTHours   float64 `json:"system_ot_hours"`
	ApprovedOTHours float64 `json:"approved_ot_hours"`
}

// DashboardStats represents organization-wide key metrics and status summary.
type DashboardStats struct {
	TotalEmployees          int64           `json:"total_employees"`
	PresentToday            int64           `json:"present_today"`
	PendingLeaveRequests    int64           `json:"pending_leave_requests"`
	EstimatedMonthlyPayroll decimal.Decimal `json:"estimated_monthly_payroll"`
	LateToday               int64           `json:"late_today"`
	AbsentToday             int64           `json:"absent_today"`
	OnLeaveToday            int64           `json:"on_leave_today"`
	EarlyDepartureToday     int64           `json:"early_departure_today"`
	AvgWorkHoursToday       float64         `json:"avg_work_hours_today"`
	AttendanceRate          RateQuantity    `json:"attendance_rate"`
	AbsenceRate             RateQuantity    `json:"absence_rate"`
	LateRate                RateQuantity    `json:"late_rate"`
}

// PeakTimeSlot represents check-in and check-out distribution by 30-min slot.
type PeakTimeSlot struct {
	TimeSlot string `json:"time_slot"`
	CheckIn  int64  `json:"check_in"`
	CheckOut int64  `json:"check_out"`
}

// LeaveTypeBreakdown represents count of leave requests by type.
type LeaveTypeBreakdown struct {
	LeaveType string `json:"leave_type"`
	Count     int64  `json:"count"`
}

// LatenessByDay represents late count and average late minutes by day of week.
type LatenessByDay struct {
	Day            string  `json:"day"`
	LateCount      int64   `json:"late_count"`
	AvgLateMinutes float64 `json:"avg_late_minutes"`
}

// OvertimeBreakdownRow represents department-level overtime hours and optional cost.

// DashboardAttendanceRepository serves attendance, lateness, overtime trend, and leave breakdown charts.
type DashboardAttendanceRepository interface {
	GetDashboardStats(ctx context.Context, tzName string, deptIDs []uuid.UUID) (DashboardStats, error)
	GetAttendanceTrend(ctx context.Context, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID) ([]AttendanceTrendPoint, error)
	GetDepartmentAttendance(ctx context.Context, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID, topN *int) ([]DepartmentAttendancePoint, error)
	GetUnexcusedAbsences(ctx context.Context, startDate, endDate time.Time, tzName string, groupBy string, deptIDs []uuid.UUID) ([]UnexcusedAbsencesPoint, error)
	GetOvertimeTrend(ctx context.Context, today time.Time) ([]OvertimeTrendPoint, error)
	GetPeakTimes(ctx context.Context, today time.Time, tzName string) ([]PeakTimeSlot, error)
	GetLeaveTypesBreakdown(ctx context.Context, today time.Time) ([]LeaveTypeBreakdown, error)
	GetLatenessByDay(ctx context.Context, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID) ([]LatenessByDay, error)
}

func (h *DashboardAnalyticsHandler) OrganizationStats(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.resolveContext(w, r)
	if !ok {
		return
	}
	_, tzName, err := h.getTimezone(r.Context(), r.URL.Query().Get("timezone"))
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	scopedDeptIDs, err := h.resolveDepartmentScope(r.Context(), userID, nil)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	stats, err := h.Repository.GetDashboardStats(r.Context(), tzName, scopedDeptIDs)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, stats)
}

// 4. GET /api/v1/dashboard/attendance-trend
func (h *DashboardAnalyticsHandler) AttendanceTrend(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.resolveContext(w, r)
	if !ok {
		return
	}
	loc, tzName, err := h.getTimezone(r.Context(), r.URL.Query().Get("timezone"))
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	now := time.Now().In(loc)
	today := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, loc)

	startDate, endDate, err := parseDateRange(r.URL.Query().Get("start_date"), r.URL.Query().Get("end_date"), today, loc)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	scopedDeptIDs, err := h.resolveDepartmentScope(r.Context(), userID, nil)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	points, err := h.Repository.GetAttendanceTrend(r.Context(), startDate, endDate, tzName, scopedDeptIDs)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if points == nil {
		points = []AttendanceTrendPoint{}
	}
	apperror.WriteJSON(w, http.StatusOK, points)
}

// 5. GET /api/v1/dashboard/department-attendance
func (h *DashboardAnalyticsHandler) DepartmentAttendance(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.resolveContext(w, r)
	if !ok {
		return
	}
	loc, tzName, err := h.getTimezone(r.Context(), r.URL.Query().Get("timezone"))
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	now := time.Now().In(loc)
	today := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, loc)

	var startDate, endDate time.Time
	if rng := r.URL.Query().Get("range"); rng != "" {
		switch rng {
		case "today":
			startDate, endDate = today, today
		case "week":
			startDate, endDate = today.AddDate(0, 0, -6), today
		case "month":
			startDate, endDate = today.AddDate(0, 0, -29), today
		default:
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	} else {
		s, e, err := parseDateRange(r.URL.Query().Get("start_date"), r.URL.Query().Get("end_date"), today, loc)
		if err != nil {
			apperror.WriteError(w, r, err)
			return
		}
		startDate, endDate = s, e
	}

	var topN *int
	if rawTop := r.URL.Query().Get("top_n"); rawTop != "" {
		n, err := strconv.Atoi(rawTop)
		if err != nil || n < 1 {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		topN = &n
	}

	scopedDeptIDs, err := h.resolveDepartmentScope(r.Context(), userID, nil)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}

	points, err := h.Repository.GetDepartmentAttendance(r.Context(), startDate, endDate, tzName, scopedDeptIDs, topN)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if points == nil {
		points = []DepartmentAttendancePoint{}
	}
	if topN != nil && len(points) > *topN {
		points = points[:*topN]
	}
	apperror.WriteJSON(w, http.StatusOK, points)
}

// 6. GET /api/v1/dashboard/unexcused-absences
func (h *DashboardAnalyticsHandler) UnexcusedAbsences(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.resolveContext(w, r)
	if !ok {
		return
	}
	groupBy := r.URL.Query().Get("group_by")
	if groupBy == "" {
		groupBy = "department"
	}
	if groupBy != "department" && groupBy != "employee" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if groupBy == "employee" && !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionDashboardEmployeeAbsenceRead)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}

	loc, tzName, err := h.getTimezone(r.Context(), r.URL.Query().Get("timezone"))
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	now := time.Now().In(loc)
	today := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, loc)

	startDate, endDate, err := parseDateRange(r.URL.Query().Get("start_date"), r.URL.Query().Get("end_date"), today, loc)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}

	scopedDeptIDs, err := h.resolveDepartmentScope(r.Context(), userID, nil)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}

	points, err := h.Repository.GetUnexcusedAbsences(r.Context(), startDate, endDate, tzName, groupBy, scopedDeptIDs)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if points == nil {
		points = []UnexcusedAbsencesPoint{}
	}
	apperror.WriteJSON(w, http.StatusOK, points)
}

// 7. GET /api/v1/dashboard/overtime-trend
func (h *DashboardAnalyticsHandler) OvertimeTrend(w http.ResponseWriter, r *http.Request) {
	_, ok := h.resolveContext(w, r)
	if !ok {
		return
	}
	loc, _, err := h.getTimezone(r.Context(), "")
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	now := time.Now().In(loc)
	today := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, loc)

	points, err := h.Repository.GetOvertimeTrend(r.Context(), today)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if points == nil {
		points = []OvertimeTrendPoint{}
	}
	apperror.WriteJSON(w, http.StatusOK, points)
}

// 8. GET /api/v1/dashboard/peak-times
func (h *DashboardAnalyticsHandler) PeakTimes(w http.ResponseWriter, r *http.Request) {
	_, ok := h.resolveContext(w, r)
	if !ok {
		return
	}
	loc, tzName, err := h.getTimezone(r.Context(), "")
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	now := time.Now().In(loc)
	today := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, loc)

	slots, err := h.Repository.GetPeakTimes(r.Context(), today, tzName)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if slots == nil {
		slots = []PeakTimeSlot{}
	}
	apperror.WriteJSON(w, http.StatusOK, slots)
}

// 9. GET /api/v1/dashboard/leave-types
func (h *DashboardAnalyticsHandler) LeaveTypes(w http.ResponseWriter, r *http.Request) {
	_, ok := h.resolveContext(w, r)
	if !ok {
		return
	}
	loc, _, err := h.getTimezone(r.Context(), "")
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	now := time.Now().In(loc)
	today := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, loc)

	items, err := h.Repository.GetLeaveTypesBreakdown(r.Context(), today)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if items == nil {
		items = []LeaveTypeBreakdown{}
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

// 10. GET /api/v1/dashboard/lateness-by-day
func (h *DashboardAnalyticsHandler) LatenessByDay(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.resolveContext(w, r)
	if !ok {
		return
	}
	loc, tzName, err := h.getTimezone(r.Context(), r.URL.Query().Get("timezone"))
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	now := time.Now().In(loc)
	today := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, loc)

	startDate, endDate, err := parseDateRange(r.URL.Query().Get("start_date"), r.URL.Query().Get("end_date"), today, loc)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	scopedDeptIDs, err := h.resolveDepartmentScope(r.Context(), userID, nil)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}
	items, err := h.Repository.GetLatenessByDay(r.Context(), startDate, endDate, tzName, scopedDeptIDs)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if items == nil {
		items = []LatenessByDay{}
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

// 11. GET /api/v1/dashboard/turnover-analytics
