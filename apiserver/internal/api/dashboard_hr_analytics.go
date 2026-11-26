package api

import (
	"context"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

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

// DashboardHRRepository serves turnover and recruitment effectiveness analytics.
type DashboardHRRepository interface {
	GetTurnoverAnalytics(ctx context.Context, query TurnoverAnalyticsQuery) (TurnoverAnalytics, error)
	GetRecruitmentEffectiveness(ctx context.Context, query RecruitmentEffectivenessQuery) (RecruitmentEffectivenessAnalytics, error)
}

func parseHRDateOrDefault(raw string, fallback time.Time, loc *time.Location) (time.Time, error) {
	if raw == "" {
		return fallback, nil
	}
	parsed, err := time.ParseInLocation("2006-01-02", raw, loc)
	if err != nil {
		return time.Time{}, apperror.Validation(apperror.KeyInvalidDateFormat)
	}
	return parsed, nil
}

func parseHRDateAndComparison(r *http.Request, loc *time.Location) (time.Time, time.Time, string, error) {
	now := time.Now().In(loc)
	today := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, loc)

	startDate, err := parseHRDateOrDefault(r.URL.Query().Get("start"), today.AddDate(0, 0, -180), loc)
	if err != nil {
		return time.Time{}, time.Time{}, "", err
	}
	endDate, err := parseHRDateOrDefault(r.URL.Query().Get("end"), today, loc)
	if err != nil {
		return time.Time{}, time.Time{}, "", err
	}
	if err := validateCustomRange(startDate, endDate, today); err != nil {
		return time.Time{}, time.Time{}, "", err
	}
	comp := r.URL.Query().Get("comparison")
	if comp == "" {
		comp = "none"
	}
	if comp != "none" && comp != "previous_period" && comp != "same_period_last_year" {
		return time.Time{}, time.Time{}, "", apperror.Validation(apperror.KeyValidation)
	}
	return startDate, endDate, comp, nil
}

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
	startDate, endDate, comp, err := parseHRDateAndComparison(r, loc)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}

	target := decimal.NewFromInt(15)
	if rawTarget := r.URL.Query().Get("target"); rawTarget != "" {
		t, err := decimal.NewFromString(rawTarget)
		if err != nil {
			writeValidation(w, r)
			return
		}
		target = t
	}

	analytics, err := h.Repository.GetTurnoverAnalytics(r.Context(), TurnoverAnalyticsQuery{
		StartDate:       startDate,
		EndDate:         endDate,
		ComparisonBasis: comp,
		Target:          target,
	})
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
	startDate, endDate, comp, err := parseHRDateAndComparison(r, loc)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}

	reqDeptIDs := parseUUIDList(r.URL.Query()["department_ids"])
	scopedDeptIDs, err := h.resolveDepartmentScope(r.Context(), userID, reqDeptIDs)
	if err != nil {
		apperror.WriteError(w, r, err)
		return
	}

	analytics, err := h.Repository.GetRecruitmentEffectiveness(r.Context(), RecruitmentEffectivenessQuery{
		StartDate:       startDate,
		EndDate:         endDate,
		ComparisonBasis: comp,
		DepartmentIDs:   scopedDeptIDs,
		LocationIDs:     parseUUIDList(r.URL.Query()["location_ids"]),
		JobRoles:        parseStringList(r.URL.Query()["job_roles"]),
		SourceTypes:     parseStringList(r.URL.Query()["source_types"]),
		SourceIDs:       parseUUIDList(r.URL.Query()["source_ids"]),
	})
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, analytics)
}

// parsing helpers
