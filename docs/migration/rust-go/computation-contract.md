# Pure Computation Contract: Rust Reference for Go Apiserver

Phase 2 P2-07 contract specification. Source of truth for in-process calculation migration (`pairing`, `matcher`, `attendance_calculator`, `financial_calculator`, `payroll_evaluator`, `payroll_rule_evaluator`, `policy_resolver`, `timezone`). Grounded in `apiserver/src/services/` and `apiserver/src/domain/`.

---

## 1. Architectural Scope & Purity Invariants

Per ADR `0002-go-technology-stack-apiserver.md` (Decision D-01) and `layer-map.md` (L6 Pure Domain Logic), all business arithmetic, scheduling logic, punch pairing, policy resolution, and payroll computations are retained in-process within pure Go packages under `internal/domain/`.

### 1.1 Strict Domain Invariants

| Invariant | Specification | Enforcement Mechanism |
|---|---|---|
| **Zero Database Access** | No imports of `database/sql`, `github.com/jackc/pgx/v5`, Diesel, or ORM drivers. | CI staticcheck package import allowlist; compile failure. |
| **Zero Network / RPC I/O** | No imports of `net/http`, `google.golang.org/grpc`, Redis, S3, or SMTP clients. | CI staticcheck package import allowlist; compile failure. |
| **Zero Wall-Clock Access** | No calls to `time.Now()` inside calculation functions. Wall-clock evaluation time MUST be passed as an explicit snapshot parameter. | Custom AST linter / code review gate. |
| **Zero Random Generation** | No calls to `uuid.New()`, `rand.*`, or non-deterministic ID generators inside domain calculation functions. IDs are provided by callers or factories outside pure domain. | CI staticcheck rule; unit tests verify determinism. |
| **Zero Global State** | No package-level mutable variables, singletons, caches, or global sync primitives. | Go race detector `go test -race ./internal/domain/...`. |
| **Arbitrary Precision Only** | Absolute prohibition of `float32` and `float64` for money, units, rates, multipliers, caps, and balances. All numeric math MUST use `github.com/shopspring/decimal`. | Static analysis ban on `float` in `internal/domain/...`. |
| **Deterministic Ordering** | All collections (logs, shifts, breaks, tiers, segments) MUST have explicit stable sort orders before evaluation. Map iteration ordering is strictly forbidden for calculation inputs. | Parity unit test suites run with inverted input order. |

---

## 2. Planned Go Package Layout & 1-to-1 Function Mapping

### 2.1 Target Package Hierarchy

```
internal/domain/
├── pairing/          # In-process punch pairing and shift-window segmentation
├── matcher/          # Shift assignment, absence detection, and entry-shift alignment
├── payroll/
│   ├── buckets/      # Minute buckets, work units, break auto-deduct/overlap
│   ├── rates/        # Daily rate, hourly rate, monthly OT cap
│   ├── incentives/   # Incentive policy & tier rule evaluation
│   ├── rules/        # Statutory deduction & allowance formula AST engine
│   └── segments/     # Mid-month salary & department change boundaries
├── policy/           # Attendance rules, shift overrides, working condition effects
└── timezone/         # IANA timezone parsing, DST shifts, calendar-day boundaries
```

### 2.2 Complete Function-by-Function 1-to-1 Mapping Table

| Rust Source File & Lines | Rust Declaration | Planned Go Package & Function | Input Snapshot Type | Output Snapshot Type | Invariant & Layer Boundary |
|---|---|---|---|---|---|
| `apiserver/src/services/pairing/strategy.rs:24-31` | `ShiftAwareStrategy::pair(&self, logs: &[LogEvent], config: Option<&PairingShiftConfig>) -> Vec<PairedSession>` | `pairing.Pair(logs []LogEvent, config *ShiftConfig) []PairedSession` | `[]LogEvent`, `*ShiftConfig` | `[]PairedSession` | Pure function. Sorts logs by minute offset; delegates to multi-segment. |
| `apiserver/src/services/pairing/strategy.rs:36-94` | `ShiftAwareStrategy::pair_multi(&self, logs: &[LogEvent], config: Option<&MultiSegmentConfig>) -> Vec<PairedSession>` | `pairing.PairMulti(logs []LogEvent, config *MultiSegmentConfig) []PairedSession` | `[]LogEvent`, `*MultiSegmentConfig` | `[]PairedSession` | Pure function. Tier 1 multi-segment window, Tier 2 single shift window, Tier 3 anchor-first fallback. |
| `apiserver/src/services/pairing/strategy.rs:105-178` | `ShiftAwareStrategy::pair_anchor_first(logs: &[LogEvent], max_gap: Option<i32>) -> Vec<PairedSession>` | `pairing.PairAnchorFirst(logs []LogEvent, maxGapMinutes *int) []PairedSession` | `[]LogEvent`, `*int` (default 960) | `[]PairedSession` | Pure function. Sequential 2-punch pairing; missing-out flagged if gap exceeds limit (960 min = 16h). |
| `apiserver/src/services/pairing/strategy_window.rs:14-109` | `strategy_window::pair_by_shift_window(logs: &[LogEvent], shift: &PairingShiftConfig) -> Vec<PairedSession>` | `pairing.PairByShiftWindow(logs []LogEvent, shift ShiftConfig) []PairedSession` | `[]LogEvent`, `ShiftConfig` | `[]PairedSession` | Pure function. Evaluates buffer (`max(120, grace)`), earliest punch, last out, break splits. |
| `apiserver/src/services/pairing/strategy_window.rs:111-120` | `strategy_window::find_earliest_in_window(logs: &[LogEvent], start_minutes: i32, buffer: i32) -> Option<&LogEvent>` | `pairing.FindEarliestInWindow(logs []LogEvent, startMinutes int, buffer int) *LogEvent` | `[]LogEvent`, `int`, `int` | `*LogEvent` | Pure search helper. Window is `[start - buffer, start + buffer]`. |
| `apiserver/src/services/pairing/strategy_window.rs:122-126` | `strategy_window::find_last_out(logs: &[LogEvent], start_minutes: i32, start_id: Uuid) -> Option<&LogEvent>` | `pairing.FindLastOut(logs []LogEvent, startMinutes int, startID uuid.UUID) *LogEvent` | `[]LogEvent`, `int`, `uuid.UUID` | `*LogEvent` | Pure search helper. Reverse scan for last punch where `id != startID && minutes > startMinutes`. |
| `apiserver/src/services/pairing/strategy_window.rs:129-161` | `strategy_window::build_single_session(...) -> PairedSession` | `pairing.BuildSingleSession(...) PairedSession` | Primitive parameters | `PairedSession` | Pure constructor. Assembles metadata with tier `shift_window`. |
| `apiserver/src/services/pairing/strategy_split.rs:22-86` | `strategy_split::split_by_breaks(input: &SplitByBreaksInput<'_>) -> Vec<PairedSession>` | `pairing.SplitByBreaks(input SplitByBreaksInput) []PairedSession` | `SplitByBreaksInput` | `[]PairedSession` | Pure function. Splits session around active break intervals; interpolates synthetic boundary timestamps. |
| `apiserver/src/services/pairing/strategy.rs:180-192` | `ShiftAwareStrategy::combine_notes(start: Option<PairingNote>, end: Option<PairingNote>) -> Option<PairingNote>` | `pairing.CombineNotes(start *PairingNote, end *PairingNote) *PairingNote` | `*PairingNote`, `*PairingNote` | `*PairingNote` | Pure helper. Merges custom strings with newline `\n`. |
| `apiserver/src/services/pairing/strategy.rs:219-234` | `ShiftAwareStrategy::finalize_note(note: Option<PairingNote>, missing_out: bool) -> Option<PairingNote>` | `pairing.FinalizeNote(note *PairingNote, missingOut bool) *PairingNote` | `*PairingNote`, `bool` | `*PairingNote` | Pure helper. Appends `missing_punch_out` on unmatched session. |
| `apiserver/src/services/pairing/strategy.rs:236-250` | `ShiftAwareStrategy::collect_log_ids(...) -> Vec<Uuid>` | `pairing.CollectLogIDs(...) []uuid.UUID` | `[]LogEvent`, `int`, `*int`, `[]uuid.UUID` | `[]uuid.UUID` | Pure helper. Gathers log IDs contained between start and end minutes. |
| `apiserver/src/services/pairing/context.rs:25-40` | `PairingContext::build_shift_config(...) -> Option<PairingShiftConfig>` | `pairing.BuildShiftConfig(...) *ShiftConfig` | `time.Time`, `time.Time`, optional breaks, grace | `*ShiftConfig` | Pure constructor. Converts wall-clock times to minute offsets from local midnight. |
| `apiserver/src/services/pairing/context.rs:59-85` | `PairingContext::build_multi_segment_config(segments: &[ResolvedSegment]) -> Option<MultiSegmentConfig>` | `pairing.BuildMultiSegmentConfig(segments []ResolvedSegment) *MultiSegmentConfig` | `[]ResolvedSegment` | `*MultiSegmentConfig` | Pure constructor. Maps resolved work segments into paired shift configurations. |
| `apiserver/src/services/pairing/context.rs:87-109` | `PairingContext::calculate_fetch_window(shift_config: Option<&PairingShiftConfig>) -> Result<(DateTime<Utc>, DateTime<Utc>), AppError>` | `timezone.CalculateFetchWindow(date time.Time, loc *time.Location, shift *ShiftConfig) (time.Time, time.Time, error)` | `time.Time`, `*time.Location`, `*ShiftConfig` | `(time.Time, time.Time)` UTC | Pure date math. Default: start-of-day to start-of-next-day. With shift: buffer ±360 minutes (6h). |
| `apiserver/src/services/pairing/context.rs:112-144` | `PairingContext::build_log_events(logs: &[RawAttendanceLog]) -> Result<Vec<LogEvent>, AppError>` | `pairing.BuildLogEvents(date time.Time, loc *time.Location, logs []RawAttendanceLog) ([]LogEvent, error)` | `time.Time`, `*time.Location`, `[]RawAttendanceLog` | `[]LogEvent` | Pure function. Deduplicates logs within 5 seconds; computes signed minutes offset from local 00:00:00. |
| `apiserver/src/services/pairing/context.rs:146-203` | `PairingContext::build_time_entries(...) -> Result<Vec<TimeEntry>, AppError>` | `pairing.BuildTimeEntries(params BuildTimeEntriesParams) ([]TimeEntry, error)` | `BuildTimeEntriesParams` | `[]TimeEntry` | Pure mapping. Discards sessions where local start date != shift date. ID generation & timestamps supplied by caller. |
| `apiserver/src/services/attendance_matcher/scoring.rs:50-62` | `scoring::compute_late_minutes(punch_in: DateTime<Tz>, shift_start: DateTime<Tz>, grace: i32, is_holiday: bool) -> i64` | `matcher.ComputeLateMinutes(punchIn time.Time, shiftStart time.Time, graceMinutes int, isHoliday bool) int64` | `time.Time`, `time.Time`, `int`, `bool` | `int64` | Pure function. Returns 0 if holiday or within grace threshold (`punch_in <= shift_start + grace`). |
| `apiserver/src/services/attendance_matcher/scoring.rs:67-77` | `scoring::compute_early_minutes(punch_out: DateTime<Tz>, shift_end: DateTime<Tz>, is_holiday: bool) -> i64` | `matcher.ComputeEarlyMinutes(punchOut time.Time, shiftEnd time.Time, isHoliday bool) int64` | `time.Time`, `time.Time`, `bool` | `int64` | Pure function. Returns 0 if holiday or `punch_out >= shift_end`. Difference in whole minutes. |
| `apiserver/src/services/attendance_matcher/scoring.rs:83-93` | `scoring::detect_absence(...) -> bool` | `matcher.DetectAbsence(...) bool` | Punches, boundaries, grace, holiday | `bool` | Pure predicate. `compute_late_minutes > 0 \|\| compute_early_minutes > 0`. |
| `apiserver/src/services/attendance_matcher/scoring.rs:124-138` | `scoring::resolve_shift_window(tz: Tz, date: NaiveDate, interval: &RosterInterval) -> Option<(DateTime<Tz>, DateTime<Tz>)>` | `matcher.ResolveShiftWindow(loc *time.Location, date time.Time, interval RosterInterval) (*time.Time, *time.Time, bool)` | `*time.Location`, `time.Time`, `RosterInterval` | `(*time.Time, *time.Time)` | Pure timezone function. Handles overnight shifts via calendar day arithmetic (`date + 1 day`). |
| `apiserver/src/services/attendance_matcher/scoring.rs:148-189` | `AttendanceMatcher::process_matched_entry(...) -> (i32, i32, Decimal, Decimal, bool)` | `matcher.ProcessMatchedEntry(...) MatchedEntryResult` | `ScoringContext`, `TimeEntry`, boundaries | `MatchedEntryResult` | Pure scoring. Returns `is_violation`; std/ot units and holiday premiums returned as zero (sourced from summaries). |
| `apiserver/src/services/attendance_matcher/scoring.rs:196-252` | `AttendanceMatcher::process_daily_entries(...) -> (i32, i32, Decimal, Decimal, i32)` | `matcher.ProcessDailyEntries(...) DailyAttendanceResult` | `ScoringContext`, `date`, `shifts`, `entries` | `DailyAttendanceResult` | Pure scoring loop. Matches work entries to shift intervals; aggregates violations. |
| `apiserver/src/services/attendance_matcher/scoring.rs:255-286` | `AttendanceMatcher::match_entries_to_shift(...) -> Vec<&TimeEntry>` | `matcher.MatchEntriesToShift(...) []*TimeEntry` | Entries, shifts, index, date, end | `[]*TimeEntry` | Pure filter. Filters entries where `punch_in >= prev_end && punch_in < current_end`. |
| `apiserver/src/services/attendance_matcher/pairing.rs:12-58` | `AttendanceMatcher::resolve_day_context(ctx: &AttendanceMatcherContext, date: NaiveDate) -> Option<(Option<Uuid>, Uuid, Uuid)>` | `matcher.ResolveDayContext(ctx *MatcherContext, date time.Time) (*DayContext, bool)` | `*MatcherContext`, `time.Time` | `DayContext (EmployerID, EmploymentID, LocationID)` | Pure resolution over pre-loaded employment and assignment slices. |
| `apiserver/src/services/attendance_matcher/pairing.rs:65-95` | `AttendanceMatcher::collect_sorted_day_shifts(...) -> Vec<(Uuid, &RosterInterval)>` | `matcher.CollectSortedDayShifts(...) []SortedShiftInterval` | In-memory `RosterData`, employee tags/id, date | `[]SortedShiftInterval` | Pure shift resolution. Sorts intervals strictly by `start_time` ascending. |
| `apiserver/src/services/attendance_matcher/mod.rs:55-118` | `AttendanceMatcher::process_segment_attendance(args: ProcessSegmentAttendanceArgs) -> SegmentAttendanceResult` | `matcher.ProcessSegmentAttendance(args ProcessSegmentArgs) SegmentAttendanceResult` | `ProcessSegmentArgs` | `SegmentAttendanceResult` | Pure date-range iterator over pre-fetched `AttendanceMatcherContext`. Zero DB queries. |
| `apiserver/src/services/attendance_calculator.rs:116-120` | `AttendanceCalculator::truncate_to_minute<T: TimeZone>(dt: DateTime<T>) -> DateTime<T>` | `buckets.TruncateToMinute(t time.Time) time.Time` | `time.Time` | `time.Time` | Zeroes seconds and nanoseconds. |
| `apiserver/src/services/attendance_calculator.rs:123-190` | `AttendanceCalculator::calculate_buckets<T: TimeZone>(input: &DailyCalcInput<T>) -> WorkBuckets` | `buckets.CalculateBuckets(input DailyCalcInput) WorkBuckets` | `DailyCalcInput` | `WorkBuckets` | Pure bucket math: standard, early OT, late OT, break OT. Overnight linearization. |
| `apiserver/src/services/attendance_calculator.rs:193-215` | `AttendanceCalculator::effective_ot_start<T: TimeZone>(...) -> DateTime<T>` | `buckets.EffectiveOTStart(punchIn time.Time, shiftStart time.Time, rules PolicyRules) time.Time` | `time.Time`, `time.Time`, `PolicyRules` | `time.Time` | Pure helper. Applies early arrival strategy: `Block`/`Allow` vs `Clamp`. |
| `apiserver/src/services/attendance_calculator.rs:218-297` | `AttendanceCalculator::fill_standard_bucket(...)` | `buckets.FillStandardBucket(...)` | Inputs, boundaries, rules, mut buckets | Mutates standard minutes & break OT | Applies paid early leave authorization, break overlap / auto-deduct. |
| `apiserver/src/services/attendance_calculator.rs:300-325` | `AttendanceCalculator::fill_early_ot(...)` | `buckets.FillEarlyOT(...)` | Inputs, boundaries, mut buckets | Mutates early OT minutes | Computes pre-shift minutes; respects `max_early_ot_minutes`. |
| `apiserver/src/services/attendance_calculator.rs:328-350` | `AttendanceCalculator::fill_late_ot(...)` | `buckets.FillLateOT(...)` | Inputs, boundaries, mut buckets | Mutates late OT minutes | Computes post-shift minutes; caps at `ot_ceiling`. |
| `apiserver/src/services/attendance_calculator.rs:369-393` | `AttendanceCalculator::calculate_break_deduction(...) -> i32` | `buckets.CalculateBreakDeduction(input BreakDeductionInput) int32` | `BreakDeductionInput` | `int32` (minutes) | Fixed mode (frame intersection) vs flexible mode (flat deduction). |
| `apiserver/src/services/attendance_calculator.rs:397-411` | `AttendanceCalculator::calculate_units(minutes: i32, unit_size: i32, policy: RoundingPolicy) -> i32` | `buckets.CalculateUnits(minutes int32, unitSize int32, policy RoundingPolicy) int32` | `int32`, `int32`, `RoundingPolicy` | `int32` | Truncates/rounds minutes into work units (`Floor`, `Ceil`, `Round`, `None`). |
| `apiserver/src/services/attendance_calculator.rs:415-430` | `AttendanceCalculator::calculate_daily_units(...) -> (i32, i32)` | `buckets.CalculateDailyUnits(input DailyCalcInput) (int32, int32)` | `DailyCalcInput` | `(stdUnits, otUnits)` | High-level bucket calculation + unit conversion. |
| `apiserver/src/services/attendance_calculator.rs:440-500` | `AttendanceCalculator::calculate_segment_units(...) -> SegmentCalcResult` | `buckets.CalculateSegmentUnits(inputs []DailyCalcInput) SegmentCalcResult` | `[]DailyCalcInput` | `SegmentCalcResult` | Handles `Aggregate` (daily sum) vs `PerSegment` (independent OT) modes. |
| `apiserver/src/services/financial_calculator.rs:82-88` | `FinancialCalculator::calculate_daily_rate(total_base: Decimal, std_days: Decimal) -> Decimal` | `rates.CalculateDailyRate(totalBase decimal.Decimal, stdDays decimal.Decimal) decimal.Decimal` | `decimal.Decimal`, `decimal.Decimal` | `decimal.Decimal` | Zero-safe division: returns `0` if `stdDays.IsZero()`. |
| `apiserver/src/services/financial_calculator.rs:90-100` | `FinancialCalculator::calculate_hourly_rate(ot_base: Decimal, std_days: Decimal, hours: Decimal) -> Decimal` | `rates.CalculateHourlyRate(otBase decimal.Decimal, stdDays decimal.Decimal, hours decimal.Decimal) decimal.Decimal` | `otBase`, `stdDays`, `hours` | `decimal.Decimal` | Returns `0` if `stdDays.IsZero() \|\| hours.IsZero()`. `(ot_base / std_days) / hours`. |
| `apiserver/src/services/financial_calculator.rs:104-125` | `FinancialCalculator::apply_ot_cap(...) -> (i32, Decimal)` | `rates.ApplyOTCap(totalOTUnits int32, otPay decimal.Decimal, maxMonthlyHours decimal.Decimal, workUnitMinutes int32) (int32, decimal.Decimal)` | `int32`, `decimal.Decimal`, `decimal.Decimal`, `int32` | `(cappedUnits, adjustedPay)` | Proportional reduction: `adjusted_pay = (otPay * (maxUnits / totalUnits)).Round(0)`. |
| `apiserver/src/services/financial_calculator.rs:132-167` | `FinancialCalculator::calculate_incentives(...) -> Result<IncentivesResult, AppError>` | `incentives.CalculateIncentives(policies []IncentivePolicyWithTiers, leave LeaveBreakdown, args CalculateIncentivesArgs) (IncentivesResult, error)` | Policy tiers, leave breakdown, attendance stats | `IncentivesResult` | Pure evaluation of active policies against employee monthly stats. |
| `apiserver/src/services/financial_calculator.rs:174-210` | `FinancialCalculator::calculate_deductions(...) -> Result<DeductionsResult, AppError>` | `rates.CalculateDeductions(...) (DeductionsResult, error)` | Leave breakdown, unpaid holidays, rule assignments, base values | `DeductionsResult` | Unpaid leave deduction (`daily_rate * unpaid_days`) + statutory rule engine. |
| `apiserver/src/domain/policy/overtime.rs:42-129` | `OvertimeEngine::calculate_segments(rules: &[OvertimeRule], date: NaiveDate, is_holiday: bool) -> Vec<OvertimeSegment>` | `overtime.CalculateSegments(rules []OvertimeRule, date time.Time, isHoliday bool) []OvertimeSegment` | `[]OvertimeRule`, `time.Time`, `bool` | `[]OvertimeSegment` | Pure function. Filters active rules by weekday mask & holiday; computes time boundary split-points via BTreeSet; resolves highest priority rule per slice; merges adjacent matching segments. |
| `apiserver/src/domain/policy/overtime.rs:131-161` | `OvertimeEngine::calculate_overtime_pay_from_times(...) -> Decimal` | `overtime.CalculateOvertimePayFromTimes(...) decimal.Decimal` | Rules, date, holiday, intervals, hourlyRate | `decimal.Decimal` | Pure pay calculation: `duration_hours * hourly_rate * segment.multiplier()`. |
| `apiserver/src/domain/policy/overtime.rs:164-211` | `OvertimeEngine::split_interval_by_day(...) -> Vec<(NaiveDate, Vec<(NaiveTime, NaiveTime)>)>` | `overtime.SplitIntervalByDay(...) []DailyOvertimeIntervals` | Start, end, `*time.Location` | `[]DailyOvertimeIntervals` | Pure timezone interval splitter across midnight boundaries; delegates day boundaries to timezone package. |
| `apiserver/src/services/financial_calculator.rs:216-238` | `FinancialCalculator::assemble_payslip(args: AssemblePayslipArgs) -> Payslip` | `payroll.AssemblePayslip(args AssemblePayslipArgs) Payslip` | `AssemblePayslipArgs` (incl. `ID`, `CreatedAt`, `UpdatedAt`) | `Payslip` | Net pay formula: `total_base + ot_pay + incentives + holiday_premium - deductions`. ID and timestamps MUST be passed in. |
| `apiserver/src/services/payroll_evaluator.rs:17-66` | `PayrollEvaluator::evaluate_policy(...) -> Result<IncentiveEvaluationResult, AppError>` | `incentives.EvaluatePolicy(...) (IncentiveEvaluationResult, error)` | `IncentivePolicy`, `[]IncentiveTier`, `IncentiveContext` | `IncentiveEvaluationResult` | Evaluates tiers in priority order; supports percentage (`base * tier_payout`) and fixed. |
| `apiserver/src/services/payroll_evaluator.rs:68-150` | `PayrollEvaluator::matches_tier(tier: &IncentiveTier, ctx: &IncentiveContext) -> bool` | `incentives.MatchesTier(tier IncentiveTier, ctx IncentiveContext) bool` | `IncentiveTier`, `IncentiveContext` | `bool` | Evaluates Option A (legacy flat object) and Option B (dynamic rules array with metrics/operators). |
| `apiserver/src/services/payroll_rule_evaluator.rs:17-104` | `PayrollRuleEvaluator::evaluate_rules(...) -> Result<Vec<RuleEvaluationResult>, AppError>` | `rules.EvaluateRules(assignments []RuleAssignmentWithTiers, baseValues map[string]decimal.Decimal) ([]RuleEvaluationResult, error)` | `[]RuleAssignmentWithTiers`, `map[string]decimal.Decimal` | `[]RuleEvaluationResult` | Percentage (with cap), Fixed, Tiered bracket formula. Zero-amount results filtered out. |
| `apiserver/src/services/payroll_service/segments.rs:15-43` | `payroll_service::segments::build_segments(ctx: &PayrollContext) -> Vec<(NaiveDate, NaiveDate)>` | `segments.BuildSegments(startDate time.Time, endDate time.Time, salaryChanges []time.Time, deptChanges []time.Time) []DateRange` | Period start, end, salary dates, dept dates | `[]DateRange` | Deduplicates change points via sorted set; yields continuous contiguous sub-segments. |
| `apiserver/src/services/payroll_service/helpers.rs:16-29` | `payroll_service::helpers::month_boundaries(year: i32, month: u32) -> Result<(NaiveDate, NaiveDate), AppError>` | `payroll.MonthBoundaries(year int, month int) (time.Time, time.Time, error)` | `int` (year), `int` (month 1-12) | `(startDate, endDate)` | Validates month 1-12; leap-year safe calendar boundaries. |
| `apiserver/src/services/attendance_utils.rs:178-223` | `attendance_utils::determine_daily_status(input: &DailyStatusInput) -> AttendanceStatus` | `matcher.DetermineDailyStatus(input DailyStatusInput) AttendanceStatus` | `DailyStatusInput` | `AttendanceStatus` | SSOT status logic: Present, Late (>5m grace), Absent, Off, Unscheduled, HolidayWork, HolidayPaid, Leave. |
| `apiserver/src/services/attendance_utils.rs:233-243` | `attendance_utils::extract_punch_status(raw_data: Option<&Value>) -> (i32, i32)` | `pairing.ExtractPunchStatus(raw []byte) (int32, int32)` | `[]byte` JSON | `(punch, status)` | Extracts `punch` and `status` from raw log metadata; defaults to -1. |
| `apiserver/src/services/attendance_utils.rs:290-303` | `attendance_utils::group_entries_by_date(entries: &[TimeEntry], tz: Tz) -> HashMap<NaiveDate, Vec<&TimeEntry>>` | `matcher.GroupEntriesByDate(entries []TimeEntry, loc *time.Location) map[string][]TimeEntry` | `[]TimeEntry`, `*time.Location` | `map[string][]TimeEntry` | Groups entries by local calendar date (`YYYY-MM-DD`); sorts each group by `punch_in_time` ascending. |
| `apiserver/src/domain/policy/resolver.rs:44-82` | `PolicyResolver::resolve_standard_month_pure(...) -> StandardMonthConfig` | `policy.ResolveStandardMonth(...) StandardMonthConfig` | Optional Employee, Dept, Workspace configs | `StandardMonthConfig` | 4-tier hierarchy: Individual -> Department -> Workspace -> System default (26 days, 8 hrs). |
| `apiserver/src/domain/policy/resolver.rs:144-167` | `PolicyResolver::resolve_attendance_rules_pure(...) -> PolicyRules` | `policy.ResolveAttendanceRules(...) PolicyRules` | `AttendanceRules`, `[]WorkingCondition`, catalog | `PolicyRules` | Base workspace rules merged with catalog-driven working condition effects. |
| `apiserver/src/domain/policy/resolver.rs:171-196` | `PolicyResolver::build_rules_from_conditions(...) -> PolicyRules` | `policy.BuildRulesFromConditions(...) PolicyRules` | `[]WorkingCondition`, date, strategy, grace, catalog | `PolicyRules` | Date-filtered condition evaluation without DB queries. |
| `apiserver/src/domain/policy/resolver.rs:198-211` | `PolicyResolver::apply_shift_overrides(...) -> PolicyRules` | `policy.ApplyShiftOverrides(rules PolicyRules, overrides ShiftPolicyOverrides) PolicyRules` | `PolicyRules`, `ShiftPolicyOverrides` | `PolicyRules` | Shift interval overrides for auto-deduct, early OT, and break OT. Grace period is decoupled. |
| `apiserver/src/utils/tz.rs:23-32` | `tz::parse_workspace_tz(tz_str: &str) -> Tz` | `timezone.ParseWorkspaceTZ(tzStr string) *time.Location` | `string` | `*time.Location` | Lenient reader: invalid or empty timezone string logs warning and degrades to `time.UTC`. |
| `apiserver/src/utils/tz.rs:47-52` | `tz::validate_tz(tz_str: &str) -> Result<Tz, AppError>` | `timezone.ValidateTZ(tzStr string) (*time.Location, error)` | `string` | `*time.Location` | Strict writer: rejects non-IANA timezone strings with validation error (400 Bad Request). |
| `apiserver/src/utils/tz.rs:69-92` | `tz::local_to_utc(tz: &Tz, date: NaiveDate, h: u32, m: u32, s: u32) -> Result<DateTime<Utc>, String>` | `timezone.LocalToUTC(loc *time.Location, year int, month int, day int, h int, m int, s int) (time.Time, error)` | `*time.Location`, date & time components | `time.Time` (UTC) | Unified DST strategy: Single = exact, Ambiguous = first instant, Gap = shift forward 1 hour. |
| `apiserver/src/utils/tz.rs:104-124` | `tz::local_to_tz(...) -> Result<DateTime<Tz>, String>` | `timezone.LocalToTZ(...) (time.Time, error)` | `*time.Location`, date & time components | `time.Time` (zoned) | Returns timezone-aware `time.Time` with identical DST gap/ambiguity handling. |
| `apiserver/src/utils/tz.rs:157-173` | `tz::naive_time_to_tz_overnight(...) -> Option<DateTime<Tz>>` | `timezone.NaiveTimeToTZOvernight(...) (*time.Time, error)` | `time.Time` (clock), `refTime`, date, `*time.Location` | `*time.Time` | If `time < refTime`, resolves on `date + 1 calendar day` using calendar math. |
| `apiserver/src/utils/tz.rs:200-250` | `tz::resolve_shift_times(...) -> ShiftTimeResolution` | `timezone.ResolveShiftTimes(...) ShiftTimes` | Start, end, break start/end, date, `*time.Location` | `ShiftTimes` | SSOT for shift & break time resolution. Handles overnight shift and overnight break cross-over. |
| `apiserver/src/utils/tz.rs:259-282` | `tz::start_of_day_utc / end_of_day_utc / start_of_next_day_utc` | `timezone.StartOfDayUTC / EndOfDayUTC / StartOfNextDayUTC` | `*time.Location`, date | `time.Time` (UTC) | Calendar day boundaries; handles 23h and 25h DST transition days. |

---

## 3. Snapshot Data Types & Structures

All inputs to pure domain packages MUST be passed as plain Go structs. No references to database rows, Diesel models, or API request context.

### 3.1 Pairing Snapshots (`internal/domain/pairing`)

```go
package pairing

import (
	"time"
	"github.com/google/uuid"
)

type EntryType string
const (
	EntryTypeWork     EntryType = "work"
	EntryTypeBreak    EntryType = "break"
	EntryTypeOvertime EntryType = "overtime"
)

type LogEvent struct {
	ID        uuid.UUID
	Minutes   int32     // Minutes from local midnight [00:00:00]
	PunchTime time.Time // UTC timestamp
	RawData   []byte    // JSON payload
	AllIDs    []uuid.UUID
}

type BreakInterval struct {
	StartMinutes int32
	EndMinutes   int32
}

type ShiftConfig struct {
	StartMinutes         int32
	EndMinutes           int32
	Breaks               []BreakInterval
	CheckInBufferMinutes *int32
	MaxSessionGapMinutes *int32
	SegmentID            *uuid.UUID
}

type MultiSegmentConfig struct {
	Segments []ShiftConfig
}

type PairedSession struct {
	StartMinutes    int32
	EndMinutes      *int32
	StartTime       time.Time
	EndTime         *time.Time
	EntryType       EntryType
	Note            *string
	IsBreakOvertime bool
	Metadata        map[string]any
}
```

### 3.2 Attendance Buckets Snapshots (`internal/domain/payroll/buckets`)

```go
package buckets

import (
	"time"
	"github.com/google/uuid"
)

type OtAggregationMode string
const (
	OtAggregate  OtAggregationMode = "aggregate"
	OtPerSegment OtAggregationMode = "per_segment"
)

type RoundingPolicy string
const (
	RoundingPolicyFloor RoundingPolicy = "floor"
	RoundingPolicyRound RoundingPolicy = "round"
	RoundingPolicyCeil  RoundingPolicy = "ceil"
	RoundingPolicyNone  RoundingPolicy = "none"
)

type WorkBuckets struct {
	StandardMinutes int32
	EarlyOTMinutes  int32
	LateOTMinutes   int32
	BreakOTMinutes  int32
}

type BreakDeductionInput struct {
	ActualBreakMinutes    int32
	ScheduledBreakMinutes int32
	IsBreakAutoDeduct     bool
	IsBreakOvertime       bool
	BreakStart            *time.Time
	BreakEnd              *time.Time
	PunchIn               time.Time
	PunchOut              time.Time
}

type DailyCalcInput struct {
	PunchIn               time.Time
	PunchOut              time.Time
	ShiftStart            *time.Time
	ShiftEnd              *time.Time
	ScheduledBreakMinutes int32
	ActualBreakMinutes    int32
	BreakStart            *time.Time
	BreakEnd              *time.Time
	WorkUnit              int32
	RoundingPolicy        RoundingPolicy
	GracePeriodMinutes    int32
	EarlyArrivalStrategy  string // "block", "allow", "clamp"
	IsEarlyLeavePaid      bool
	AuthorizedEarlyLeave  int32
	AllowEarlyOT          bool
	IsBreakAutoDeduct     bool
	IsBreakOvertime       bool
	OTCeiling             *time.Time
	MaxEarlyOTMinutes     *int32
	SegmentID             *uuid.UUID
	OTAggregationMode     OtAggregationMode
}

type SegmentCalcResult struct {
	TotalStandardUnits int32
	TotalOTUnits       int32
	PerSegmentBuckets  []SegmentBucket
}

type SegmentBucket struct {
	SegmentID *uuid.UUID
	Buckets   WorkBuckets
}
```

### 3.3 Matcher Snapshots (`internal/domain/matcher`)

```go
package matcher

import (
	"time"
	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

type ScoringContext struct {
	Location               *time.Location
	WorkingConditions      []WorkingConditionSnapshot
	HolidaysMap            map[string]HolidaySnapshot // Key: "YYYY-MM-DD"
	ConditionTypeCatalog   map[string]ConditionTypeSnapshot
}

type SegmentAttendanceResult struct {
	OTPay              decimal.Decimal
	HolidayWorkPremium decimal.Decimal
	LateEarlyCount     int32
}

type ProcessSegmentArgs struct {
	Ctx              *ScoringContext
	SegStart         time.Time
	SegEnd           time.Time
	SegmentTotalBase decimal.Decimal
	SegmentOTBase    decimal.Decimal
	StdWorkingDays   decimal.Decimal
	HoursPerDay      decimal.Decimal
	WorkUnitMinutes  int32
	RoundingPolicy   string
	DailyEntries     map[string][]TimeEntrySnapshot
	DayShifts        map[string][]RosterIntervalSnapshot
}
```

### 3.4 Payroll Rate & Formula Snapshots (`internal/domain/payroll`)

```go
package payroll

import (
	"time"
	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

type IncentiveContext struct {
	TotalWorkUnits  int32
	TotalLeaveDays  decimal.Decimal
	UnpaidLeaveDays decimal.Decimal
	SickLeaveDays   decimal.Decimal
	LateEarlyCount  int32
	KPIScore        decimal.Decimal
}

type IncentiveTier struct {
	ID           uuid.UUID
	PolicyID     uuid.UUID
	Priority     int32
	RuleConfig   []byte // JSON raw message
	PayoutAmount decimal.Decimal
}

type IncentiveEvaluationResult struct {
	PolicyName     string
	Amount         decimal.Decimal
	StandardAmount decimal.Decimal
	TierID         *uuid.UUID
}

type RuleAssignmentWithTiers struct {
	Code              string
	CalculationMethod string // "percentage", "fixed", "tiered"
	DefaultConfig     []byte
	OverrideConfig    []byte
	Tiers             []RuleTierSnapshot
}

type RuleTierSnapshot struct {
	MinAmount   decimal.Decimal
	MaxAmount   *decimal.Decimal
	Rate        decimal.Decimal
	FixedAmount decimal.Decimal
	TierOrder   int32
}

type RuleEvaluationResult struct {
	RuleCode string
	Amount   decimal.Decimal
}

type OvertimeSegment struct {
	Start      time.Time // Local wall-clock; maps to NaiveTime in Rust
	End        time.Time // Local wall-clock; maps to NaiveTime in Rust
	Multiplier decimal.Decimal
}

type OvertimeRule struct {
	ID         uuid.UUID
	PolicyID   uuid.UUID
	Priority   int32
	Multiplier decimal.Decimal
	TimeStart  *time.Time // Local clock; maps to Option<NaiveTime> in Rust
	TimeEnd    *time.Time // Local clock; maps to Option<NaiveTime> in Rust
	DaysOfWeek int32      // Bitmask 1 << weekday (Monday = 1)
	IsHoliday  bool
}

type AssemblePayslipArgs struct {
	ID                       uuid.UUID
	WorkspaceID              uuid.UUID
	EmploymentID             uuid.UUID
	StartDate                time.Time
	EndDate                  time.Time
	TotalBasePay             decimal.Decimal
	TotalOvertimePay         decimal.Decimal
	TotalHolidayWorkPremium  decimal.Decimal
	TotalIncentiveAmount     decimal.Decimal
	DeductionsAmount         decimal.Decimal
	TotalStdUnits            int32
	TotalOTUnits             int32
	CreatedAt                time.Time
	UpdatedAt                time.Time
}
```

---

## 4. Deterministic Ordering Invariants

To guarantee exact parity with Rust and eliminate non-deterministic output drift between differential test runs:

1. **Log Events Ordering**:
   - In `pairing.PairMulti` and `pairing.BuildLogEvents`, logs MUST be sorted by `LogEvent.Minutes` ascending (`apiserver/src/services/pairing/strategy.rs:46`).
   - Tie-breaking: identical minute punches preserve order of arrival; punches within 5 seconds (`abs(t1 - t2) < 5s`) collapse into a single `LogEvent` with aggregated `all_ids` (`apiserver/src/services/pairing/context.rs:129`).
2. **Break Intervals Ordering**:
   - Breaks inside `ShiftConfig` MUST be sorted by `BreakInterval.StartMinutes` ascending before splitting (`apiserver/src/services/pairing/strategy_window.rs:78`).
3. **Roster Shift Intervals Ordering**:
   - Daily shift intervals resolved from `RosterData` MUST be sorted strictly by `RosterInterval.StartTime` ascending (`apiserver/src/services/attendance_matcher/pairing.rs:93`).
4. **Incentive Tiers Ordering**:
   - Tiers inside `PayrollEvaluator::evaluate_policy` MUST be sorted by `IncentiveTier.Priority` ascending (`apiserver/src/services/payroll_evaluator.rs:24`). First matching tier terminates evaluation.
5. **Statutory Deduction Tiers Ordering**:
   - Tiers inside `PayrollRuleEvaluator::evaluate_rules` MUST be sorted by `PayrollRuleTier.TierOrder` ascending (`apiserver/src/models/payroll_rules.rs`).
6. **Segment Date Change Points**:
   - Dates for salary/department change boundaries MUST be inserted into an ordered, deduplicated set (`BTreeSet` equivalent in Go: slice sorted with `sort.Slice` and compacted with `slices.Compact`) before building contiguous segments (`apiserver/src/services/payroll_service/segments.rs:16-30`).
7. **Daily Entries Map Iteration**:
   - Never iterate a Go `map[time.Time][]TimeEntry` directly during calculations. When iterating dates, generate an explicit sequential date loop from `StartDate` to `EndDate` via `curr = curr.AddDate(0, 0, 1)`.

---

## 5. Timezone & DST Calculation Rules

All timezone handling in Go MUST be centralized under `internal/domain/timezone` (or `pkg/timezone`), adhering to the reference rules in `apiserver/src/utils/tz.rs`.

### 5.1 Conversion Rules & Edge Cases

| Scenario | Rust Reference Rule (`apiserver/src/utils/tz.rs`) | Go Implementation Requirement |
|---|---|---|
| **Lenient TZ Parse** | `parse_workspace_tz`: empty or invalid string falls back to `chrono_tz::UTC` (`:23-32`). | `time.LoadLocation(name)`: if error or empty, log `slog.Warn` and return `time.UTC`. Used for read/batch pipelines. |
| **Strict TZ Parse** | `validate_tz`: invalid string returns `AppError::Validation(I18nKey::InvalidTimezone)` (`:47-52`). | Rejects invalid IANA strings with typed validation error. Used on workspace/device write paths. |
| **DST Gap (Clocks Jump Forward)** | `local_to_utc`: nonexistent wall-clock time shifts forward by 1 hour and retries (`:83-90`). | Check `t.In(loc)` wall-clock match; if gap detected, shift `t = t.Add(1 * time.Hour)` to resolve target UTC instant. |
| **DST Ambiguous (Clocks Fall Back)** | `local_to_utc`: duplicate wall-clock time picks the *first* (pre-transition / daylight) instant (`:79-82`). | Resolve ambiguous instant to earlier UTC timestamp (standard Go `time.Date` resolves pre-transition by default). |
| **Overnight Shifts** | `resolve_shift_times`: `end < start` resolves `end` on `date + 1 calendar day` (`:209-216`). | `nextDate = date.AddDate(0, 0, 1)`; resolve wall-clock end time in `nextDate`. Preserves intended clock hour across DST. |
| **Start / End of Day** | `start_of_day_utc`: local 00:00:00. `start_of_next_day_utc`: local midnight of `date + 1 calendar day` (`:259-282`). | Use `time.Date(year, month, day + 1, 0, 0, 0, 0, loc).UTC()`. NEVER use `start.Add(24 * time.Hour)` (incorrect on 23h/25h DST days). |
| **Punch Minute Offset** | `build_log_events`: `minutes = (punch_local - start_of_day_local).num_minutes()` (`context.rs:122`). | Calculate duration between punch and local midnight in minutes: `int32(punchInLoc.Sub(midnightLoc).Minutes())`. |

---

## 6. Rounding, Numeric Scales & Decimal Math

Per ADR `0002-go-technology-stack-apiserver.md` (Decision D-04) and `type-mapping.md`, all financial and attendance calculations use `shopspring/decimal`.

### 6.1 Rounding Policy Definitions

Used by `buckets.CalculateUnits` (`apiserver/src/services/attendance_calculator.rs:397-411`):

```go
func CalculateUnits(minutes int32, unitSize int32, policy RoundingPolicy) int32 {
	if unitSize <= 1 {
		unitSize = 1
	}
	minsDec := decimal.NewFromInt32(minutes)
	sizeDec := decimal.NewFromInt32(unitSize)
	ratio := minsDec.Div(sizeDec)

	switch policy {
	case RoundingPolicyFloor:
		return int32(ratio.Floor().IntPart())
	case RoundingPolicyCeil:
		return int32(ratio.Ceil().IntPart())
	case RoundingPolicyRound:
		// Half away from zero round matching standard business math
		return int32(ratio.Round(0).IntPart())
	case RoundingPolicyNone:
		// Truncate towards zero
		return int32(ratio.Truncate(0).IntPart())
	default:
		return int32(ratio.Floor().IntPart())
	}
}
```

### 6.2 Monetary Formula Scale and Precision

1. **Daily Rate** (`rates.CalculateDailyRate`):
   - Formula: $\text{daily\_rate} = \frac{\text{total\_base}}{\text{std\_working\_days}}$ (`financial_calculator.rs:82-88`).
   - If $\text{std\_working\_days} = 0$, rate is $0$.
   - Scale: Keep full decimal precision (no premature intermediate rounding).
2. **Hourly Rate** (`rates.CalculateHourlyRate`):
   - Formula: $\text{hourly\_rate} = \frac{\text{ot\_base}}{\text{std\_working\_days} \times \text{hours\_per\_day}}$ (`financial_calculator.rs:90-100`).
   - If $\text{std\_working\_days} = 0$ or $\text{hours\_per\_day} = 0$, rate is $0$.
3. **Monthly Overtime Cap** (`rates.ApplyOTCap`):
   - Maximum allowed minutes: $\text{max\_minutes} = \text{max\_monthly\_hours} \times 60$.
   - Maximum units: $\text{max\_units} = \lfloor\frac{\text{max\_minutes}}{\text{work\_unit\_minutes}}\rfloor$.
   - If $\text{total\_ot\_units} > \text{max\_units}$:
     $$\text{adjusted\_pay} = \left(\text{total\_overtime\_pay} \times \frac{\text{max\_units}}{\text{total\_ot\_units}}\right).\text{Round}(0)$$
     $$\text{capped\_units} = \text{max\_units}$$
   - Rounding: `.Round(0)` (scale 0 round-half-up for integer currency units, e.g. VND).
4. **Statutory Deduction / Rule Evaluator Formulas** (`rules.EvaluateRules`):
   - Percentage: $\text{amount} = \text{base\_value} \times \frac{\text{rate}}{100}$. If $\text{cap\_amount} > 0$, $\text{amount} = \min(\text{amount}, \text{cap\_amount})$.
   - Fixed: $\text{amount} = \text{fixed\_config\_amount}$.
   - Tiered Marginal Brackets:
     For each tier where $\text{current\_base} > \text{tier.min\_amount}$:
     $$\text{taxable} = \min(\text{current\_base}, \text{tier.max\_amount}) - \text{tier.min\_amount}$$
     $$\text{tier\_pay} = \left(\text{taxable} \times \frac{\text{tier.rate}}{100}\right) + \text{tier.fixed\_amount}$$
     $$\text{amount} = \text{amount} + \text{tier\_pay}$$
   - Zero-amount suppression: If $\text{amount} \le 0$, the rule evaluation result is NOT added to the result slice (`payroll_rule_evaluator.rs:94-99`).

---

## 7. Error Classification & Handling

Pure domain calculations do not return HTTP status codes or database error types.

### 7.1 Domain Error Rules

| Domain Error Category | Rust Representation | Go Domain Return | Handling at Service Layer |
|---|---|---|---|
| **Numeric Overflow / Out-of-Range** | `AppError::Internal("Log minute offset overflow...")` | `ErrMinuteOverflow = errors.New("minute offset overflow")` | Maps to `AppError.Internal` with structured log. |
| **Invalid Calculation Boundaries** | `AppError::Validation(I18nKey::InvalidYearMonth)` | `ErrInvalidPeriod = errors.New("invalid year or month")` | Maps to `AppError.Validation` (400 Bad Request). |
| **Invalid Timezone String** | `AppError::Validation(I18nKey::InvalidTimezone)` | `ErrInvalidTimezone = errors.New("invalid IANA timezone")` | Maps to `AppError.Validation` (400 Bad Request). |
| **Invalid Rule Configuration** | `AppError::Internal("Invalid monetary amount in incentive tier")` | `ErrInvalidRuleConfig = errors.New("invalid rule config")` | Maps to `AppError.Internal` (500 Internal Server Error). |

Domain packages define sentinel errors (`var Err... = errors.New(...)`). The calling Service layer wraps domain errors into the application's unified `AppError`.

---

## 8. Prohibited I/O & Wall-Clock Invariants

The following operations are strictly forbidden inside `internal/domain/...`:

| Prohibited Operation | Why Forbidden | Required Go Pattern |
|---|---|---|
| `time.Now()`, `Utc::now()` | Produces non-deterministic output across test runs; breaks snapshot replay. | Pass `calcTime time.Time` or `date time.Time` as an explicit parameter in the input snapshot. |
| `uuid.New()`, `Uuid::new_v4()` | Randomness prevents deterministic output comparison. | Caller generates UUIDs or passes a deterministic test ID generator. |
| Database queries (`pgx`, `sql.DB`, Diesel) | Couples business logic to schema and I/O latency; prevents pure unit testing. | Pre-load all required data into memory snapshots before calling domain packages. |
| Redis access (`go-redis`) | Hidden latency, network failure modes, cache invalidation races. | Read cache in service orchestration layer; pass raw snapshots to domain. |
| HTTP / S3 / Network calls | Destroys idempotency and domain testability. | Adapter calls in `internal/service/` or `internal/adapter/`; domain receives bytes. |
| Goroutines (`go func()`) | Spawns unmanaged background work inside pure algorithms. | All pure calculation functions must execute synchronously on the caller's goroutine. |

---

## 9. Parity Test Fixture Inventory

The following table inventories all existing Rust calculation tests and establishes exact Go test file locations for parity verification. Counts equal `#[test]`/`#[tokio::test]` annotations measured inside the cited source ranges; "Planned" verification methods are contract targets, not completed work.

| Rust Test Suite & File | Test Count | Key Test Scenarios Covered | Target Go Parity Test File | Verification Method |
|---|---:|---|---|---|
| `apiserver/src/services/pairing/tests.rs:1-2910` | 63 | Standard shift window, long overtime (23:00 punch), logs outside window fallback, anchor-first sequential pairing, 16h gap cutoff, overnight shift with break, rapid punch deduplication, DST spring-forward (23h) and fall-back (25h) fetch window, 4-session bug reproduction scenario, negative minutes, unsorted logs. | `internal/domain/pairing/pairing_test.go` | Planned: table-driven pure unit tests replay the same punch fixtures and compare session metadata. |
| `apiserver/tests/integration/attendance/pairing.rs:1-353` | 20 | Tier 1 vs Tier 2 integration, early arrival (90m early within buffer), late departure, mid-day punch outside break, missing check-out with custom notes. | `internal/domain/pairing/integration_test.go` | Planned: integration tests compare Go and Rust time-entry outputs for the same fixture. |
| `apiserver/src/services/attendance_matcher/tests_mod.rs:1-1397` | 30 | Segment attendance without shifts, shift matching with overtime, paid holiday work compensation, paid holiday auto-credit on zero work, break-is-overtime flag, late arrival beyond grace, early departure, DST overnight shift window in America/New_York (spring-forward 7h vs fall-back 9h duration). | `internal/domain/matcher/matcher_test.go` | Planned: golden-vector tests compare `SegmentAttendanceResult` with captured Rust outputs. |
| `apiserver/src/services/attendance_calculator_tests.rs:1-1261` | 83 | Minute truncation, standard work bucket, early OT with clamping/blocking, late OT with ceiling, frame intersection break deduction vs flat deduction, work unit rounding (Floor/Ceil/Round/None), segment unit aggregation mode (`Aggregate` vs `PerSegment`). Module included by `apiserver/src/services/mod.rs:100`. | `internal/domain/payroll/buckets/calculator_test.go` | Planned: boundary assertion suite covering every annotated source case. |
| `apiserver/src/services/financial_calculator.rs:241-527` | 21 | Zero-safe daily rate, fractional hourly rate, monthly OT cap exact boundary / proportional reduction, payslip net pay assembly with incentives/deductions, zero-everything boundary. | `internal/domain/payroll/rates/calculator_test.go` | Planned: decimal arithmetic tests assert exact `decimal.Decimal` equivalence. |
| `apiserver/src/services/payroll_evaluator.rs:153-659` | 9 | Incentive policy payout calculation, percentage vs fixed, dynamic rules array operators (`>`, `<`, `==`, `!=`, `>=`), sick leave & unpaid leave thresholds, tier priority ordering. | `internal/domain/payroll/incentives/evaluator_test.go` | Planned: table tests assert tier payouts from fixed monthly-stat snapshots. |
| `apiserver/src/services/payroll_rule_evaluator.rs:106-246` | 6 | Percentage statutory rules with cap, fixed rules, multi-tier marginal tax bracket rules, zero-amount suppression, empty assignments. | `internal/domain/payroll/rules/evaluator_test.go` | Planned: rule-execution tests use equivalent rule definitions and base-value snapshots. |
| `apiserver/src/services/payroll_service/segments.rs:45-261` | 7 | Salary change mid-month, department change mid-month, multiple changes on same day deduplication, change on day 1 (no extra segment), change after month end ignored. | `internal/domain/payroll/segments/segments_test.go` | Planned: date-range comparison verifies identical sub-period boundaries. |
| `apiserver/src/services/payroll_service/helpers.rs:50-96` | 6 | Month boundaries for January, February leap year (29 days), December, invalid months (0, 13), daily entries grouping by local date. | `internal/domain/payroll/helpers_test.go` | Planned: calendar-boundary tests verify leap-year and invalid-period parity. |
| `apiserver/src/utils/tz.rs:302-1877` | 84 | Strict IANA validation, lenient fallback to UTC, local-to-UTC across Asia/Ho_Chi_Minh, Europe/London, America/New_York, DST gap shift forward (+1h), DST ambiguous resolution (first instant), start/end of day UTC, overnight shift calendar-day arithmetic. | `internal/domain/timezone/timezone_test.go` | Planned: differential timezone tests assert identical UTC timestamps per edge case. |
| `apiserver/src/domain/policy/resolver.rs:290-771` | 19 | 4-tier standard month resolution (individual, department, workspace, system default), condition effects (early leave allowance, grace period, early arrival strategy), shift overrides decoupling. | `internal/domain/policy/resolver_test.go` | Planned: precedence table tests verify inheritance resolution order. |
| `apiserver/src/domain/policy/overtime.rs:214-621` | 16 | Weekday/holiday rule filtering, priority resolution, boundary splitting/merging, overtime pay by interval, zero-rate/no-rule behavior, timezone-aware day splitting. | `internal/domain/payroll/overtime_test.go` | Planned: interval comparison verifies merged segments and total overtime pay. |
| `apiserver/src/services/attendance_utils.rs:309-1182` | 45 | Attendance status enum/string behavior, present/late/absent/off/unscheduled and holiday/leave statuses, raw punch/status defaults, local date range conversion, entry grouping and ordering. | `internal/domain/matcher/status_test.go` | Planned: status state-machine table tests verify daily status determination. |
| `apiserver/src/services/rule_evaluator.rs:116-538` | 21 | Assignment rule matching by department, employee ID whitelist, tags intersection, catch-all fallback, rule resolution priority ordering. | `internal/domain/matcher/rule_evaluator_test.go` | Planned: priority-filter tests verify matched policy IDs for fixed employee contexts. |
| `apiserver/src/services/payroll_service/tests_mod.rs:1-366` | 12 | Daily rate and OT cap helpers, snapshot/context construction, empty/missing employee data and shared batch data. | `internal/domain/payroll/payroll_test.go` | Planned: snapshot-builder tests assert valid batch-orchestrator inputs. |

---

## 10. Exact Unknowns & Contract Decisions

The following discrepancies, ambiguities, and contract decisions were identified during source code analysis:

### 10.1 Resolved Decisions

1. **Deterministic Assembler Parameters (Decision P2-07-C1)**:
   - *Observation*: `FinancialCalculator::assemble_payslip` in Rust generates `Uuid::new_v4()` and calls `Utc::now()` internally (`apiserver/src/services/financial_calculator.rs:218,235,236`).
   - *Contract*: In Go, `AssemblePayslipArgs` MUST include explicit `ID uuid.UUID`, `CreatedAt time.Time`, and `UpdatedAt time.Time` generated in the orchestration service layer. Pure domain functions MUST NOT generate random IDs or read wall clocks.
2. **Float Representation in Payroll Rules and Incentives (Decision D-12 & D-04)**:
   - *Observation*: Rust uses `f64` for leave days in `IncentiveContext` (`total_leave_days`, `unpaid_leave_days`, `sick_leave_days`), KPI score, and `payout_amount` (`apiserver/src/services/payroll_evaluator.rs:8-13`).
   - *Contract*: Go replaces all `f64` fields in `IncentiveContext` and `IncentiveTier` with `decimal.Decimal`. Wire JSON floats are parsed directly into `decimal.Decimal` via string or byte decoding to prevent binary floating-point drift.
3. **Legacy `OtAggregationMode` Value Collapsing**:
   - *Observation*: The database contains three string values: `"aggregate"`, `"daily"`, and `"per_segment"`. Rust collapses both `"aggregate"` and `"daily"` to `OtAggregationMode::Aggregate` (`apiserver/src/services/attendance_calculator.rs:40`).
   - *Contract*: Go preserves this exact behavior. Any string other than `"per_segment"` collapses to `OtAggregate`.
4. **Overnight Break Splitting Guard**:
   - *Observation*: `pair_by_shift_window` intentionally suppresses splitting when a break's `end_minutes >= 1440` (`apiserver/src/services/pairing/strategy_window.rs:73`).
   - *Contract*: Go MUST retain this check. Splitting a break across midnight produces a session on D+1 that would be dropped by date filtering, dropping the employee's check-out punch.
5. **Deduplication Threshold**:
   - *Observation*: `PairingContext::build_log_events` merges punches within 5 seconds (`num_seconds().abs() < 5`) into a single `LogEvent` (`apiserver/src/services/pairing/context.rs:129`).
   - *Contract*: Go implementation MUST use strictly `math.Abs(t1.Sub(t2).Seconds()) < 5`.

### 10.2 Open Unknowns & Decisions for Review

| Unknown ID | Subject | Location in Rust Source | Description | Proposed Go Contract |
|---|---|---|---|---|
| **UNK-COMP-01** | Shift interval OT pay calculation redundancy | `apiserver/src/services/attendance_matcher/scoring.rs:185,248` | In Phase 7 of Rust development, standard units, OT units, and holiday premiums were decoupled from the matcher and sourced exclusively from `daily_summaries`. However, `day_ot_pay` is still calculated and summed in `att.ot_pay`. In `payslip_calc.rs:211`, `att.ot_pay` is added to `total_overtime_pay`. In existing tests, `att.ot_pay` evaluates to `0` because overtime rules are resolved in the summary service. | Retain `ot_pay` calculation in matcher for compatibility with the formula engine, but confirm whether Go summary recalculation is the sole source of truth for daily overtime pay. |
| **UNK-COMP-02** | Seniority month calculation approximation | `apiserver/src/services/leave_accrual_service/calculator.rs:102` | `calculate_total_months_seniority` calculates total service days across contracts and divides by 30: `total_days / 30`. This is an approximation rather than calendar-month calculation. | Preserve `total_days / 30` integer division in Go for 100% parity with legacy leave accruals, pending product team review. |
| **UNK-COMP-03** | Dynamic rule array operator precision | `apiserver/src/services/payroll_evaluator.rs:138-139` | In dynamic rules array, float equality uses `f64::EPSILON`: `(ctx_val - value).abs() < f64::EPSILON`. | With migration to `decimal.Decimal`, equality becomes exact: `ctxVal.Equal(ruleVal)`. Parity tests must verify whether existing dynamic rules rely on epsilon tolerance. |
