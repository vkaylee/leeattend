package api

import (
	"context"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/timezone"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

// RateQuantity represents a percentage rate along with the raw quantities it was derived from.
// Value is nil when denominator <= 0 (never a fabricated "0.0").
type RateQuantity struct {
	Value       *decimal.Decimal `json:"value"`
	Numerator   int64            `json:"numerator"`
	Denominator int64            `json:"denominator"`
	Unit        string           `json:"unit"`
}

func NewRateQuantity(numerator, denominator int64, unit string) RateQuantity {
	if denominator <= 0 {
		return RateQuantity{
			Value:       nil,
			Numerator:   max64(0, numerator),
			Denominator: denominator,
			Unit:        unit,
		}
	}
	val := decimal.NewFromInt(max64(0, numerator)).Div(decimal.NewFromInt(denominator)).Mul(decimal.NewFromInt(100))
	return RateQuantity{
		Value:       &val,
		Numerator:   max64(0, numerator),
		Denominator: denominator,
		Unit:        unit,
	}
}

// AttendanceTrendPoint represents daily attendance stats in a trend chart.
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
type OvertimeBreakdownRow struct {
	DepartmentID      uuid.UUID        `json:"department_id"`
	DepartmentName    string           `json:"department_name"`
	SystemOTHours     float64          `json:"system_ot_hours"`
	ApprovedOTHours   float64          `json:"approved_ot_hours"`
	UnapprovedOTHours float64          `json:"unapproved_ot_hours"`
	OvertimeCost      *decimal.Decimal `json:"overtime_cost,omitempty"`
	CostAvailable     bool             `json:"cost_available"`
}

type DashboardOrganizationContext struct {
	Timezone string `json:"timezone"`
	Currency string `json:"currency"`
}

type OvertimeBreakdownResponse struct {
	ContractVersion string                       `json:"contract_version"`
	Organization    DashboardOrganizationContext `json:"organization"`
	Rows            []OvertimeBreakdownRow       `json:"rows"`
	GeneratedAt     string                       `json:"generated_at"`
}

// Executive overview response models

type DashboardReportingPeriod struct {
	Key       string    `json:"key"`
	StartDate string    `json:"start_date"`
	EndDate   string    `json:"end_date"`
	Timezone  string    `json:"timezone"`
	AsOf      time.Time `json:"as_of"`
}

type DashboardMetricScope struct {
	Kind              string      `json:"kind"`
	DepartmentIDs     []uuid.UUID `json:"department_ids"`
	LocationIDs       []uuid.UUID `json:"location_ids"`
	OmittedDimensions []string    `json:"omitted_dimensions"`
}

type DashboardMetricContext struct {
	Period     DashboardReportingPeriod `json:"period"`
	Comparison string                   `json:"comparison"`
	Filters    map[string]string        `json:"filters"`
	Scope      DashboardMetricScope     `json:"scope"`
}

type DashboardMetricQuantity struct {
	Value decimal.Decimal `json:"value"`
	Unit  string          `json:"unit"`
}

type DashboardMetricFreshness struct {
	AsOf       time.Time `json:"as_of"`
	LagSeconds *uint64   `json:"lag_seconds"`
}

type DashboardMetricComparison struct {
	Basis                 string           `json:"basis"`
	PeriodStart           string           `json:"period_start"`
	PeriodEnd             string           `json:"period_end"`
	Value                 *decimal.Decimal `json:"value"`
	Delta                 *decimal.Decimal `json:"delta"`
	DeltaPercent          *decimal.Decimal `json:"delta_percent"`
	DeltaPercentagePoints *decimal.Decimal `json:"delta_percentage_points"`
	Status                string           `json:"status"`
}

type DashboardMetricTarget struct {
	Value    decimal.Decimal `json:"value"`
	Operator string          `json:"operator"`
}

type DashboardMetric struct {
	Key               string                     `json:"key"`
	Value             *decimal.Decimal           `json:"value"`
	Unit              string                     `json:"unit"`
	Numerator         *DashboardMetricQuantity   `json:"numerator"`
	Denominator       *DashboardMetricQuantity   `json:"denominator"`
	Status            string                     `json:"status"`
	Health            string                     `json:"health"`
	NullReason        *string                    `json:"null_reason"`
	DefinitionVersion string                     `json:"definition_version"`
	Direction         string                     `json:"direction"`
	Freshness         DashboardMetricFreshness   `json:"freshness"`
	QualityFlags      []string                   `json:"quality_flags"`
	Comparison        *DashboardMetricComparison `json:"comparison"`
	Target            *DashboardMetricTarget     `json:"target"`
}

type DashboardInsight struct {
	ID                 string            `json:"id"`
	Type               string            `json:"type"`
	Severity           string            `json:"severity"`
	MetricKey          string            `json:"metric_key"`
	TitleKey           string            `json:"title_key"`
	SummaryKey         string            `json:"summary_key"`
	Parameters         map[string]string `json:"parameters"`
	RequiresPermission *string           `json:"requires_permission"`
	Scope              string            `json:"scope"`
	Dismissible        bool              `json:"dismissible"`
}

type DashboardActionTypeCount struct {
	Type  string `json:"type"`
	Count uint64 `json:"count"`
}

type DashboardActionsSummary struct {
	Total   uint64                     `json:"total"`
	Overdue uint64                     `json:"overdue"`
	ByType  []DashboardActionTypeCount `json:"by_type"`
}

type DashboardQualitySource struct {
	Key           string                   `json:"key"`
	Status        string                   `json:"status"`
	CoverageRatio *decimal.Decimal         `json:"coverage_ratio"`
	Freshness     DashboardMetricFreshness `json:"freshness"`
	Warnings      []string                 `json:"warnings"`
}

type DashboardDataQuality struct {
	Status  string                   `json:"status"`
	Sources []DashboardQualitySource `json:"sources"`
}

type DashboardMetricsResponse struct {
	ContractVersion string                       `json:"contract_version"`
	Organization    DashboardOrganizationContext `json:"organization"`
	Context         DashboardMetricContext       `json:"context"`
	Metrics         []DashboardMetric            `json:"metrics"`
	Insights        []DashboardInsight           `json:"insights"`
	Actions         DashboardActionsSummary      `json:"actions"`
	DataQuality     DashboardDataQuality         `json:"data_quality"`
	GeneratedAt     time.Time                    `json:"generated_at"`
}

// Turnover analytics response models
type TurnoverTenureBucket struct {
	Bucket string `json:"bucket"`
	Count  int64  `json:"count"`
}

type TurnoverBySource struct {
	SourceType string `json:"source_type"`
	SourceName string `json:"source_name"`
	Count      int64  `json:"count"`
}

type TurnoverByDepartment struct {
	DepartmentName string `json:"department_name"`
	Count          int64  `json:"count"`
}

type TurnoverRate struct {
	Value       *decimal.Decimal `json:"value"`
	Numerator   int64            `json:"numerator"`
	Denominator int64            `json:"denominator"`
	Unit        string           `json:"unit"`
}

func NewTurnoverRate(numerator, denominator int64) TurnoverRate {
	if denominator <= 0 {
		return TurnoverRate{
			Value:       nil,
			Numerator:   max64(0, numerator),
			Denominator: denominator,
			Unit:        "employees",
		}
	}
	val := decimal.NewFromInt(max64(0, numerator)).Div(decimal.NewFromInt(denominator)).Mul(decimal.NewFromInt(100))
	return TurnoverRate{
		Value:       &val,
		Numerator:   max64(0, numerator),
		Denominator: denominator,
		Unit:        "employees",
	}
}

type TurnoverTarget struct {
	Value    decimal.Decimal `json:"value"`
	Operator string          `json:"operator"`
	Source   string          `json:"source"`
}

type TurnoverTrendPoint struct {
	Month        string        `json:"month"`
	Headcount    int64         `json:"headcount"`
	Leavers      int64         `json:"leavers"`
	TurnoverRate *TurnoverRate `json:"turnover_rate"`
}

type CohortRetentionRow struct {
	CohortMonth     string           `json:"cohort_month"`
	MonthsSinceHire int64            `json:"months_since_hire"`
	CohortSize      int64            `json:"cohort_size"`
	ActiveCount     int64            `json:"active_count"`
	RetentionRate   *decimal.Decimal `json:"retention_rate"`
}

type TurnoverReasonRow struct {
	Reason string `json:"reason"`
	Count  int64  `json:"count"`
}

type TurnoverClassification struct {
	Classification string `json:"classification"`
	Count          int64  `json:"count"`
}

type TurnoverByEmploymentType struct {
	EmploymentType string `json:"employment_type"`
	Count          int64  `json:"count"`
}

type TurnoverByLocation struct {
	LocationName string `json:"location_name"`
	Count        int64  `json:"count"`
}

type TurnoverByManager struct {
	ManagerName string `json:"manager_name"`
	Count       int64  `json:"count"`
}

type TurnoverWarning struct {
	Code    string `json:"code"`
	Message string `json:"message"`
}

type TurnoverComparison struct {
	Basis                 string           `json:"basis"`
	PeriodStart           string           `json:"period_start"`
	PeriodEnd             string           `json:"period_end"`
	Rate                  *TurnoverRate    `json:"rate"`
	DeltaPercentagePoints *decimal.Decimal `json:"delta_percentage_points"`
}

type TurnoverAnalytics struct {
	TenureDistribution         []TurnoverTenureBucket     `json:"tenure_distribution"`
	BySource                   []TurnoverBySource         `json:"by_source"`
	ByDepartment               []TurnoverByDepartment     `json:"by_department"`
	TotalTerminated            int64                      `json:"total_terminated"`
	PeriodStart                string                     `json:"period_start"`
	PeriodEnd                  string                     `json:"period_end"`
	TurnoverRate               TurnoverRate               `json:"turnover_rate"`
	AverageHeadcount           int64                      `json:"average_headcount"`
	VoluntaryCount             int64                      `json:"voluntary_count"`
	InvoluntaryCount           int64                      `json:"involuntary_count"`
	UnknownClassificationCount int64                      `json:"unknown_classification_count"`
	RetentionRate              *decimal.Decimal           `json:"retention_rate"`
	EarlyAttrition             TurnoverRate               `json:"early_attrition"`
	RegrettableAttrition       *decimal.Decimal           `json:"regrettable_attrition"`
	Comparison                 *TurnoverComparison        `json:"comparison"`
	Target                     TurnoverTarget             `json:"target"`
	MonthlyTrend               []TurnoverTrendPoint       `json:"monthly_trend"`
	CohortRetention            []CohortRetentionRow       `json:"cohort_retention"`
	TerminationReasons         []TurnoverReasonRow        `json:"termination_reasons"`
	Classification             []TurnoverClassification   `json:"classification"`
	ByEmploymentType           []TurnoverByEmploymentType `json:"by_employment_type"`
	ByLocation                 []TurnoverByLocation       `json:"by_location"`
	ByManager                  []TurnoverByManager        `json:"by_manager"`
	Warnings                   []TurnoverWarning          `json:"warnings"`
}

// Recruitment effectiveness response models
type RecruitmentEffectivenessOverview struct {
	Hires                int64            `json:"hires"`
	AverageCostPerHire   *decimal.Decimal `json:"average_cost_per_hire"`
	CostAvailable        bool             `json:"cost_available"`
	Retained90d          int64            `json:"retained_90d"`
	Retention90d         *decimal.Decimal `json:"retention_90d"`
	TurnoverInRange      int64            `json:"turnover_in_range"`
	ComparisonHires      *int64           `json:"comparison_hires"`
	ComparisonHiresDelta *int64           `json:"comparison_hires_delta"`
}

type RecruitmentFunnelStage struct {
	Stage          string           `json:"stage"`
	Count          *int64           `json:"count"`
	DataSource     *string          `json:"data_source"`
	ConversionRate *decimal.Decimal `json:"conversion_rate"`
}

type RecruitmentEffectivenessSource struct {
	SourceID             *uuid.UUID       `json:"source_id"`
	SourceType           string           `json:"source_type"`
	SourceName           string           `json:"source_name"`
	Hires                int64            `json:"hires"`
	HiresRatio           *decimal.Decimal `json:"hires_ratio"`
	Retained90d          int64            `json:"retained_90d"`
	Retention90d         *decimal.Decimal `json:"retention_90d"`
	CostPerHire          *decimal.Decimal `json:"cost_per_hire"`
	CostPerHireAvailable bool             `json:"cost_per_hire_available"`
	TurnoverInRange      int64            `json:"turnover_in_range"`
}

type RecruitmentTrendPoint struct {
	Month string `json:"month"`
	Hires int64  `json:"hires"`
}

type RecruitmentDataGap struct {
	Metric string `json:"metric"`
	Reason string `json:"reason"`
}

type RecruitmentEffectivenessAnalytics struct {
	PeriodStart string                           `json:"period_start"`
	PeriodEnd   string                           `json:"period_end"`
	TotalHires  int64                            `json:"total_hires"`
	Overview    RecruitmentEffectivenessOverview `json:"overview"`
	Funnel      []RecruitmentFunnelStage         `json:"funnel"`
	BySource    []RecruitmentEffectivenessSource `json:"by_source"`
	HiresTrend  []RecruitmentTrendPoint          `json:"hires_trend"`
	DataGaps    []RecruitmentDataGap             `json:"data_gaps"`
	Warnings    []TurnoverWarning                `json:"warnings"`
}

func DefaultRecruitmentFunnel(hires, retained90d int64) []RecruitmentFunnelStage {
	var retConv *decimal.Decimal
	if hires > 0 {
		v := decimal.NewFromInt(retained90d).Div(decimal.NewFromInt(hires)).Mul(decimal.NewFromInt(100))
		retConv = &v
	}
	return []RecruitmentFunnelStage{
		{Stage: "applicants", Count: nil, DataSource: nil, ConversionRate: nil},
		{Stage: "screening", Count: nil, DataSource: nil, ConversionRate: nil},
		{Stage: "interview", Count: nil, DataSource: nil, ConversionRate: nil},
		{Stage: "offer", Count: nil, DataSource: nil, ConversionRate: nil},
		{Stage: "hired", Count: new(hires), DataSource: new("hires"), ConversionRate: nil},
		{Stage: "retained_90d", Count: new(retained90d), DataSource: new("hires"), ConversionRate: retConv},
	}
}

func DefaultRecruitmentDataGaps() []RecruitmentDataGap {
	return []RecruitmentDataGap{
		{Metric: "applicants", Reason: "dashboard.hr.effective_gap_no_applicant_data"},
		{Metric: "screening", Reason: "dashboard.hr.effective_gap_no_screening_data"},
		{Metric: "interview", Reason: "dashboard.hr.effective_gap_no_interview_data"},
		{Metric: "offer", Reason: "dashboard.hr.effective_gap_no_offer_data"},
		{Metric: "offer_acceptance", Reason: "dashboard.hr.effective_gap_no_offer_acceptance"},
		{Metric: "time_to_hire", Reason: "dashboard.hr.effective_gap_no_time_to_hire"},
		{Metric: "time_to_fill", Reason: "dashboard.hr.effective_gap_no_time_to_fill"},
		{Metric: "quality_of_hire", Reason: "dashboard.hr.effective_gap_no_performance_data"},
	}
}

// Request / normalization parameters
type NormalizedDashboardMetricsQuery struct {
	Period           string
	Comparison       string
	StartDate        time.Time
	EndDate          time.Time
	DepartmentIDs    []uuid.UUID
	LocationIDs      []uuid.UUID
	BusinessUnitIDs  []uuid.UUID
	ManagerIDs       []uuid.UUID
	EmploymentTypes  []string
	EmployeeStatuses []string
	ShiftIDs         []uuid.UUID
	TopN             int
	RiskThreshold    string
	Timezone         string
}

type OvertimeBreakdownQuery struct {
	StartDate *time.Time
	EndDate   *time.Time
	Timezone  *string
	TopN      *int
	DeptIDs   []uuid.UUID
}

type TurnoverAnalyticsQuery struct {
	StartDate       time.Time
	EndDate         time.Time
	ComparisonBasis string
	Target          decimal.Decimal
}

type RecruitmentEffectivenessQuery struct {
	StartDate       time.Time
	EndDate         time.Time
	ComparisonBasis string
	DepartmentIDs   []uuid.UUID
	LocationIDs     []uuid.UUID
	JobRoles        []string
	SourceTypes     []string
	SourceIDs       []uuid.UUID
}

// Repository and authorizer interfaces
type DashboardAnalyticsRepository interface {
	GetOrganizationTimezone(ctx context.Context) (string, error)
	GetExecutiveOverview(ctx context.Context, query NormalizedDashboardMetricsQuery, allowOvertimeCost bool) (DashboardMetricsResponse, error)
	GetOvertimeBreakdown(ctx context.Context, query OvertimeBreakdownQuery, allowOvertimeCost bool) (OvertimeBreakdownResponse, error)
	GetDashboardStats(ctx context.Context, tzName string, deptIDs []uuid.UUID) (DashboardStats, error)
	GetAttendanceTrend(ctx context.Context, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID) ([]AttendanceTrendPoint, error)
	GetDepartmentAttendance(ctx context.Context, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID, topN *int) ([]DepartmentAttendancePoint, error)
	GetUnexcusedAbsences(ctx context.Context, startDate, endDate time.Time, tzName string, groupBy string, deptIDs []uuid.UUID) ([]UnexcusedAbsencesPoint, error)
	GetOvertimeTrend(ctx context.Context, today time.Time) ([]OvertimeTrendPoint, error)
	GetPeakTimes(ctx context.Context, today time.Time, tzName string) ([]PeakTimeSlot, error)
	GetLeaveTypesBreakdown(ctx context.Context, today time.Time) ([]LeaveTypeBreakdown, error)
	GetLatenessByDay(ctx context.Context, startDate, endDate time.Time, tzName string, deptIDs []uuid.UUID) ([]LatenessByDay, error)
	GetTurnoverAnalytics(ctx context.Context, query TurnoverAnalyticsQuery) (TurnoverAnalytics, error)
	GetRecruitmentEffectiveness(ctx context.Context, query RecruitmentEffectivenessQuery) (RecruitmentEffectivenessAnalytics, error)
}

type DashboardAnalyticsAudit interface {
	Emit(context.Context, audit.Event) error
}
type DashboardAnalyticsHandler struct {
	Repository DashboardAnalyticsRepository
	Authorizer Authorizer
	Audit      DashboardAnalyticsAudit
}

func (h *DashboardAnalyticsHandler) resolveContext(w http.ResponseWriter, r *http.Request) (uuid.UUID, bool) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return uuid.Nil, false
	}
	if !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionDashboardRead)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return uuid.Nil, false
	}
	return userID, true
}

func (h *DashboardAnalyticsHandler) resolveDepartmentScope(ctx context.Context, userID uuid.UUID, requested []uuid.UUID) ([]uuid.UUID, error) {
	if h.Authorizer == nil {
		return nil, apperror.Forbidden(apperror.KeyForbidden)
	}

	var managed []uuid.UUID
	var isGlobal bool
	var err error

	isGlobal = h.Authorizer.IsGlobalManager(ctx, userID)
	if !isGlobal {
		managed, err = h.Authorizer.GetManagedDepartmentIDs(ctx, userID)
	}
	if err != nil {
		return nil, err
	}
	if isGlobal {
		return requested, nil
	}
	managedSet := make(map[uuid.UUID]bool, len(managed))
	for _, id := range managed {
		managedSet[id] = true
	}
	if len(requested) > 0 {
		for _, reqID := range requested {
			if !managedSet[reqID] {
				return nil, apperror.Forbidden(apperror.KeyForbidden)
			}
		}
		return requested, nil
	}
	return managed, nil
}

func (h *DashboardAnalyticsHandler) getTimezone(ctx context.Context, requested string) (*time.Location, string, error) {
	if requested != "" {
		loc, err := timezone.ValidateTZ(requested)
		if err != nil {
			return nil, "", apperror.Validation(apperror.KeyInvalidTimezone)
		}
		return loc, requested, nil
	}
	orgTZ, err := h.Repository.GetOrganizationTimezone(ctx)
	if err != nil || orgTZ == "" {
		orgTZ = "UTC"
	}
	loc := timezone.ParseTimezone(orgTZ)
	return loc, orgTZ, nil
}

func validateCustomRange(start, end, today time.Time) error {
	if end.Before(start) {
		return apperror.Validation(apperror.KeyInvalidDateRange)
	}
	if end.After(today) {
		return apperror.Validation(apperror.KeyInvalidDateRange)
	}
	diffDays := int(end.Sub(start).Hours()/24) + 1
	if diffDays > 366 {
		return apperror.Validation(apperror.KeyInvalidDateRange)
	}
	return nil
}

// 1. GET /api/v1/dashboard/executive-overview
func (h *DashboardAnalyticsHandler) ExecutiveOverview(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.resolveContext(w, r)
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

	period := r.URL.Query().Get("period")
	if period == "" {
		period = "today"
	}
	validPeriods := map[string]bool{"today": true, "this_week": true, "this_month": true, "this_quarter": true, "year_to_date": true, "custom": true}
	if !validPeriods[period] {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	rawStart := r.URL.Query().Get("start_date")
	rawEnd := r.URL.Query().Get("end_date")

	var startDate, endDate time.Time
	if period == "custom" {
		if rawStart == "" || rawEnd == "" {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateRange))
			return
		}
		s, err := time.ParseInLocation("2006-01-02", rawStart, loc)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
			return
		}
		e, err := time.ParseInLocation("2006-01-02", rawEnd, loc)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
			return
		}
		if err := validateCustomRange(s, e, today); err != nil {
			apperror.WriteError(w, r, err)
			return
		}
		startDate, endDate = s, e
	} else {
		if rawStart != "" || rawEnd != "" {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateRange))
			return
		}
		endDate = today
		switch period {
		case "this_week":
			weekday := int(today.Weekday())
			if weekday == 0 {
				weekday = 7
			}
			startDate = today.AddDate(0, 0, -(weekday - 1))
		case "this_month":
			startDate = time.Date(today.Year(), today.Month(), 1, 0, 0, 0, 0, loc)
		case "this_quarter":
			qMonth := ((int(today.Month())-1)/3)*3 + 1
			startDate = time.Date(today.Year(), time.Month(qMonth), 1, 0, 0, 0, 0, loc)
		case "year_to_date":
			startDate = time.Date(today.Year(), 1, 1, 0, 0, 0, 0, loc)
		default: // "today"
			startDate = today
		}
	}

	comparison := r.URL.Query().Get("comparison")
	if comparison == "" {
		comparison = "none"
	}
	validComparisons := map[string]bool{"none": true, "previous_period": true, "same_period_last_year": true, "target": true, "budget": true, "benchmark": true}
	if !validComparisons[comparison] {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	riskThreshold := r.URL.Query().Get("risk_threshold")
	if riskThreshold == "" {
		riskThreshold = "watch"
	}
	validRisk := map[string]bool{"info": true, "watch": true, "high": true, "critical": true}
	if !validRisk[riskThreshold] {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	topN := 10
	if rawTop := r.URL.Query().Get("top_n"); rawTop != "" {
		n, err := strconv.Atoi(rawTop)
		if err != nil || n < 1 || n > 100 {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		topN = n
	}

	reqDeptIDs := parseUUIDList(r.URL.Query()["department_ids"])
	scopedDeptIDs, err := h.resolveDepartmentScope(r.Context(), userID, reqDeptIDs)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}

	query := NormalizedDashboardMetricsQuery{
		Period:           period,
		Comparison:       comparison,
		StartDate:        startDate,
		EndDate:          endDate,
		DepartmentIDs:    scopedDeptIDs,
		LocationIDs:      parseUUIDList(r.URL.Query()["location_ids"]),
		BusinessUnitIDs:  parseUUIDList(r.URL.Query()["business_unit_ids"]),
		ManagerIDs:       parseUUIDList(r.URL.Query()["manager_ids"]),
		EmploymentTypes:  parseStringList(r.URL.Query()["employment_types"]),
		EmployeeStatuses: parseStringList(r.URL.Query()["employee_statuses"]),
		ShiftIDs:         parseUUIDList(r.URL.Query()["shift_ids"]),
		TopN:             topN,
		RiskThreshold:    riskThreshold,
		Timezone:         tzName,
	}

	allowOvertimeCost := h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionPayrollSensitiveRead))
	resp, err := h.Repository.GetExecutiveOverview(r.Context(), query, allowOvertimeCost)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	if allowOvertimeCost && h.Audit != nil {
		for _, m := range resp.Metrics {
			if m.Key == "overtime_cost" && m.Value != nil {
				_ = h.Audit.Emit(r.Context(), audit.Event{
					Name:     "dashboard.sensitive.read",
					ActorID:  userID.String(),
					Resource: "dashboard.executive-overview",
					Details: map[string]string{
						"endpoint":     "executive-overview",
						"period":       period,
						"start_date":   startDate.Format("2006-01-02"),
						"end_date":     endDate.Format("2006-01-02"),
						"cost_metrics": "true",
					},
					At: time.Now().UTC(),
				})
				break
			}
		}
	}

	apperror.WriteJSON(w, http.StatusOK, resp)
}

// 2. GET /api/v1/dashboard/overtime-breakdown
func (h *DashboardAnalyticsHandler) OvertimeBreakdown(w http.ResponseWriter, r *http.Request) {
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

	rawStart := r.URL.Query().Get("start_date")
	rawEnd := r.URL.Query().Get("end_date")

	var startDate, endDate time.Time
	if rawStart == "" && rawEnd == "" {
		endDate = today
		startDate = today.AddDate(0, 0, -29)
	} else if rawStart != "" && rawEnd != "" {
		s, err := time.ParseInLocation("2006-01-02", rawStart, loc)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
			return
		}
		e, err := time.ParseInLocation("2006-01-02", rawEnd, loc)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
			return
		}
		if err := validateCustomRange(s, e, today); err != nil {
			apperror.WriteError(w, r, err)
			return
		}
		startDate, endDate = s, e
	} else {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateRange))
		return
	}

	var topN *int
	if rawTop := r.URL.Query().Get("top_n"); rawTop != "" {
		n, err := strconv.Atoi(rawTop)
		if err != nil || n < 1 {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		if n > 100 {
			n = 100
		}
		topN = &n
	}

	scopedDeptIDs, err := h.resolveDepartmentScope(r.Context(), userID, nil)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}

	allowOvertimeCost := h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionPayrollSensitiveRead))
	query := OvertimeBreakdownQuery{
		StartDate: &startDate,
		EndDate:   &endDate,
		Timezone:  &tzName,
		TopN:      topN,
		DeptIDs:   scopedDeptIDs,
	}

	resp, err := h.Repository.GetOvertimeBreakdown(r.Context(), query, allowOvertimeCost)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	if topN != nil && len(resp.Rows) > *topN {
		resp.Rows = resp.Rows[:*topN]
	}

	if allowOvertimeCost && h.Audit != nil {
		for _, row := range resp.Rows {
			if row.CostAvailable {
				_ = h.Audit.Emit(r.Context(), audit.Event{
					Name:     "dashboard.sensitive.read",
					ActorID:  userID.String(),
					Resource: "dashboard.overtime-breakdown",
					Outcome:  "success",
					Details: map[string]string{
						"endpoint":     "overtime-breakdown",
						"start_date":   startDate.Format("2006-01-02"),
						"end_date":     endDate.Format("2006-01-02"),
						"cost_metrics": "true",
					},
					At: time.Now().UTC(),
				})
				break
			}
		}
	}

	apperror.WriteJSON(w, http.StatusOK, resp)
}

// 3. GET /api/v1/dashboard/organization-stats
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
func (h *DashboardAnalyticsHandler) TurnoverAnalytics(w http.ResponseWriter, r *http.Request) {
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

	startStr := r.URL.Query().Get("start")
	endStr := r.URL.Query().Get("end")

	var startDate, endDate time.Time
	if startStr != "" {
		s, err := time.ParseInLocation("2006-01-02", startStr, loc)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
			return
		}
		startDate = s
	} else {
		startDate = today.AddDate(0, 0, -180)
	}

	if endStr != "" {
		e, err := time.ParseInLocation("2006-01-02", endStr, loc)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
			return
		}
		endDate = e
	} else {
		endDate = today
	}

	if err := validateCustomRange(startDate, endDate, today); err != nil {
		apperror.WriteError(w, r, err)
		return
	}

	comp := r.URL.Query().Get("comparison")
	if comp == "" {
		comp = "none"
	}
	if comp != "none" && comp != "previous_period" && comp != "same_period_last_year" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	target := decimal.NewFromInt(15)
	if rawTarget := r.URL.Query().Get("target"); rawTarget != "" {
		t, err := decimal.NewFromString(rawTarget)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		target = t
	}

	query := TurnoverAnalyticsQuery{
		StartDate:       startDate,
		EndDate:         endDate,
		ComparisonBasis: comp,
		Target:          target,
	}

	analytics, err := h.Repository.GetTurnoverAnalytics(r.Context(), query)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, analytics)
}

// 12. GET /api/v1/dashboard/recruitment-effectiveness
func (h *DashboardAnalyticsHandler) RecruitmentEffectiveness(w http.ResponseWriter, r *http.Request) {
	userID, ok := h.resolveContext(w, r)
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

	startStr := r.URL.Query().Get("start")
	endStr := r.URL.Query().Get("end")

	var startDate, endDate time.Time
	if startStr != "" {
		s, err := time.ParseInLocation("2006-01-02", startStr, loc)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
			return
		}
		startDate = s
	} else {
		startDate = today.AddDate(0, 0, -180)
	}

	if endStr != "" {
		e, err := time.ParseInLocation("2006-01-02", endStr, loc)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
			return
		}
		endDate = e
	} else {
		endDate = today
	}

	if err := validateCustomRange(startDate, endDate, today); err != nil {
		apperror.WriteError(w, r, err)
		return
	}

	comp := r.URL.Query().Get("comparison")
	if comp == "" {
		comp = "none"
	}
	if comp != "none" && comp != "previous_period" && comp != "same_period_last_year" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	reqDeptIDs := parseUUIDList(r.URL.Query()["department_ids"])
	scopedDeptIDs, err := h.resolveDepartmentScope(r.Context(), userID, reqDeptIDs)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}

	query := RecruitmentEffectivenessQuery{
		StartDate:       startDate,
		EndDate:         endDate,
		ComparisonBasis: comp,
		DepartmentIDs:   scopedDeptIDs,
		LocationIDs:     parseUUIDList(r.URL.Query()["location_ids"]),
		JobRoles:        parseStringList(r.URL.Query()["job_roles"]),
		SourceTypes:     parseStringList(r.URL.Query()["source_types"]),
		SourceIDs:       parseUUIDList(r.URL.Query()["source_ids"]),
	}

	analytics, err := h.Repository.GetRecruitmentEffectiveness(r.Context(), query)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, analytics)
}

// parsing helpers
func parseDateRange(rawStart, rawEnd string, today time.Time, loc *time.Location) (time.Time, time.Time, error) {
	if rawStart == "" && rawEnd == "" {
		return today.AddDate(0, 0, -29), today, nil
	}
	if rawStart == "" || rawEnd == "" {
		return time.Time{}, time.Time{}, apperror.Validation(apperror.KeyInvalidDateRange)
	}
	s, err := time.ParseInLocation("2006-01-02", rawStart, loc)
	if err != nil {
		return time.Time{}, time.Time{}, apperror.Validation(apperror.KeyInvalidDateFormat)
	}
	e, err := time.ParseInLocation("2006-01-02", rawEnd, loc)
	if err != nil {
		return time.Time{}, time.Time{}, apperror.Validation(apperror.KeyInvalidDateFormat)
	}
	if err := validateCustomRange(s, e, today); err != nil {
		return time.Time{}, time.Time{}, err
	}
	return s, e, nil
}

func parseUUIDList(values []string) []uuid.UUID {
	var out []uuid.UUID
	for _, val := range values {
		for _, part := range strings.Split(val, ",") {
			part = strings.TrimSpace(part)
			if part == "" {
				continue
			}
			if id, err := uuid.Parse(part); err == nil {
				out = append(out, id)
			}
		}
	}
	return out
}

func parseStringList(values []string) []string {
	var out []string
	for _, val := range values {
		for _, part := range strings.Split(val, ",") {
			part = strings.TrimSpace(part)
			if part != "" {
				out = append(out, part)
			}
		}
	}
	return out
}

func max64(a, b int64) int64 {
	if a > b {
		return a
	}
	return b
}
