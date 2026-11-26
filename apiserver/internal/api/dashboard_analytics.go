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

// DashboardExecutiveRepository serves executive overview and overtime cost breakdowns.
type DashboardExecutiveRepository interface {
	GetOrganizationTimezone(ctx context.Context) (string, error)
	GetExecutiveOverview(ctx context.Context, query NormalizedDashboardMetricsQuery, allowOvertimeCost bool) (DashboardMetricsResponse, error)
	GetOvertimeBreakdown(ctx context.Context, query OvertimeBreakdownQuery, allowOvertimeCost bool) (OvertimeBreakdownResponse, error)
}

// DashboardAnalyticsRepository composes the segregated dashboard sub-domain interfaces.
type DashboardAnalyticsRepository interface {
	DashboardExecutiveRepository
	DashboardAttendanceRepository
	DashboardHRRepository
}

type DashboardAnalyticsHandler struct {
	Repository DashboardAnalyticsRepository
	Authorizer Authorizer
	Audit      AuditEmitter
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
