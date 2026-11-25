package repository

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type leaveTestRows struct {
	data [][]any
	idx  int
}

func (r *leaveTestRows) Close()                                       {}
func (r *leaveTestRows) Err() error                                   { return nil }
func (r *leaveTestRows) CommandTag() pgconn.CommandTag                { return pgconn.CommandTag{} }
func (r *leaveTestRows) FieldDescriptions() []pgconn.FieldDescription { return nil }
func (r *leaveTestRows) Next() bool {
	if r.idx < len(r.data) {
		r.idx++
		return true
	}
	return false
}
func (r *leaveTestRows) Scan(dest ...any) error {
	row := r.data[r.idx-1]
	for i, v := range row {
		switch d := dest[i].(type) {
		case *uuid.UUID:
			*d = v.(uuid.UUID)
		case **uuid.UUID:
			if v == nil {
				*d = nil
			} else if ptr, ok := v.(*uuid.UUID); ok {
				*d = ptr
			} else {
				val := v.(uuid.UUID)
				*d = &val
			}
		case *string:
			if s, ok := v.(string); ok {
				*d = s
			} else if sp, ok := v.(*string); ok && sp != nil {
				*d = *sp
			}
		case **string:
			if v == nil {
				*d = nil
			} else if ptr, ok := v.(*string); ok {
				*d = ptr
			} else {
				val := v.(string)
				*d = &val
			}
		case *int:
			*d = v.(int)
		case *int64:
			*d = v.(int64)
		case *bool:
			*d = v.(bool)
		case *time.Time:
			*d = v.(time.Time)
		case *decimal.Decimal:
			*d = v.(decimal.Decimal)
		case *[]byte:
			*d = v.([]byte)
		case *[]uuid.UUID:
			*d = v.([]uuid.UUID)
		}
	}
	return nil
}
func (r *leaveTestRows) Values() ([]any, error) { return r.data[r.idx-1], nil }
func (r *leaveTestRows) RawValues() [][]byte    { return nil }
func (r *leaveTestRows) Conn() *pgx.Conn        { return nil }

func TestLeaveRepository_CreateLeaveType_NoWorkspace(t *testing.T) {
	expectedID := uuid.New()
	now := time.Now().UTC()
	isPaid := true

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("expected query to not contain workspace_id: %s", sql)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = expectedID
				*(dest[1].(*string)) = "annual"
				*(dest[2].(*string)) = "Annual Leave"
				*(dest[3].(*bool)) = true
				*(dest[4].(*[]byte)) = []byte(`{}`)
				*(dest[5].(*time.Time)) = now
				return nil
			}}
		},
	}

	repo := LeaveRepository{DB: db}
	rec, err := repo.CreateLeaveType(context.Background(), api.CreateLeaveTypeRequest{
		Code:   "annual",
		Name:   "Annual Leave",
		IsPaid: &isPaid,
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if rec.ID != expectedID || rec.Code != "annual" || rec.Name != "Annual Leave" {
		t.Fatalf("unexpected record: %+v", rec)
	}
}

func TestLeaveRepository_CreateLeaveType_Conflict(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(_ ...any) error {
				return &pgconn.PgError{Code: "23505"}
			}}
		},
	}

	repo := LeaveRepository{DB: db}
	_, err := repo.CreateLeaveType(context.Background(), api.CreateLeaveTypeRequest{
		Code: "annual",
		Name: "Annual Leave",
	})
	if !errors.Is(err, api.ErrLeaveConflict) {
		t.Fatalf("expected ErrLeaveConflict, got %v", err)
	}
}

func TestLeaveRepository_ListLeaveTypes_ScopeFree(t *testing.T) {
	typeID := uuid.New()
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, args ...any) (pgx.Rows, error) {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("expected query to be scope-free without workspace_id: %s", sql)
			}
			if len(args) != 0 {
				t.Fatalf("expected 0 args, got %d", len(args))
			}
			return &leaveTestRows{
				data: [][]any{
					{typeID, "annual", "Annual Leave", true, []byte(`{}`), now},
				},
			}, nil
		},
	}

	repo := LeaveRepository{DB: db}
	types, err := repo.ListLeaveTypes(context.Background())
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(types) != 1 || types[0].ID != typeID {
		t.Fatalf("unexpected leave types: %+v", types)
	}
}

func TestLeaveRepository_UpdateLeaveTypeConfig(t *testing.T) {
	typeID := uuid.New()
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("expected query to not contain workspace_id: %s", sql)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = typeID
				*(dest[1].(*string)) = "annual"
				*(dest[2].(*string)) = "Annual Leave"
				*(dest[3].(*bool)) = true
				*(dest[4].(*[]byte)) = []byte(`{"carry_over_limit": 5}`)
				*(dest[5].(*time.Time)) = now
				return nil
			}}
		},
	}

	repo := LeaveRepository{DB: db}
	rec, err := repo.UpdateLeaveTypeConfig(context.Background(), typeID, json.RawMessage(`{"carry_over_limit": 5}`))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if rec.ID != typeID {
		t.Fatalf("unexpected record: %+v", rec)
	}
}

func TestLeaveRepository_DeleteLeaveType_ConflictAndSuccess(t *testing.T) {
	typeID := uuid.New()

	// Conflict when in use
	dbConflict := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = true
				return nil
			}}
		},
	}
	repo := LeaveRepository{DB: dbConflict}
	err := repo.DeleteLeaveType(context.Background(), typeID)
	if !errors.Is(err, api.ErrLeaveConflict) {
		t.Fatalf("expected ErrLeaveConflict, got %v", err)
	}

	// Success soft-delete
	dbSuccess := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = false
				return nil
			}}
		},
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id: %s", sql)
			}
			return pgconn.NewCommandTag("UPDATE 1"), nil
		},
	}
	repoSuccess := LeaveRepository{DB: dbSuccess}
	if err := repoSuccess.DeleteLeaveType(context.Background(), typeID); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestLeaveRepository_RestoreLeaveType(t *testing.T) {
	typeID := uuid.New()
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id in restore: %s", sql)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = typeID
				*(dest[1].(*string)) = "sick"
				*(dest[2].(*string)) = "Sick Leave"
				*(dest[3].(*bool)) = true
				*(dest[4].(*[]byte)) = []byte(`{}`)
				*(dest[5].(*time.Time)) = now
				return nil
			}}
		},
	}

	repo := LeaveRepository{DB: db}
	rec, err := repo.RestoreLeaveType(context.Background(), typeID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if rec.ID != typeID || rec.Code != "sick" {
		t.Fatalf("unexpected restored record: %+v", rec)
	}
}

func TestLeaveRepository_SubmitLeaveRequest(t *testing.T) {
	requesterID := uuid.New()
	empID := uuid.New()
	typeID := uuid.New()
	reqID := uuid.New()
	now := time.Now().UTC()

	tx := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id in submit leave: %s", sql)
			}
			if strings.Contains(sql, "FROM employments") || strings.Contains(sql, "FROM leave_types") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*bool)) = true
					return nil
				}}
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = reqID
				*(dest[1].(*uuid.UUID)) = empID
				*(dest[2].(*uuid.UUID)) = typeID
				*(dest[3].(*string)) = "2026-06-01"
				*(dest[4].(*string)) = "2026-06-02"
				reason := "Vacation"
				*(dest[5].(**string)) = &reason
				*(dest[6].(*string)) = "pre_approved"
				*(dest[7].(*string)) = "full_day"
				*(dest[8].(**string)) = nil
				*(dest[9].(**string)) = nil
				*(dest[10].(**uuid.UUID)) = &requesterID
				*(dest[11].(**uuid.UUID)) = &requesterID
				*(dest[12].(*time.Time)) = now
				*(dest[13].(*time.Time)) = now
				*(dest[14].(*[]uuid.UUID)) = nil
				return nil
			}}
		},
	}}

	repo := LeaveRepository{
		BeginTx: func(_ context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	rec, err := repo.SubmitLeaveRequest(context.Background(), requesterID, api.SubmitLeaveRequest{
		EmploymentID: empID,
		LeaveTypeID:  typeID,
		StartDate:    "2026-06-01",
		EndDate:      "2026-06-02",
	}, true)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if rec.ID != reqID || rec.Status != "pre_approved" {
		t.Fatalf("unexpected request record: %+v", rec)
	}
}

func TestLeaveRepository_ListOrganizationLeaveRequests(t *testing.T) {
	reqID := uuid.New()
	empID := uuid.New()
	typeID := uuid.New()
	userID := uuid.New()
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id: %s", sql)
			}
			reason := "Doctor visit"
			return &leaveTestRows{
				data: [][]any{
					{reqID, empID, typeID, "2026-07-01", "2026-07-01", &reason, "pending", "full_day", (*string)(nil), (*string)(nil), &userID, (*uuid.UUID)(nil), now, now, ([]uuid.UUID)(nil)},
				},
			}, nil
		},
	}

	repo := LeaveRepository{DB: db}
	status := "pending"
	list, err := repo.ListOrganizationLeaveRequests(context.Background(), &status)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(list) != 1 || list[0].ID != reqID {
		t.Fatalf("unexpected list: %+v", list)
	}
}

func TestLeaveRepository_ApproveLeaveRequest(t *testing.T) {
	reqID := uuid.New()
	approverID := uuid.New()

	tx := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*string)) = "pending"
				return nil
			}}
		},
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id: %s", sql)
			}
			return pgconn.NewCommandTag("UPDATE 1"), nil
		},
	}}

	repo := LeaveRepository{
		BeginTx: func(_ context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	err := repo.ApproveLeaveRequest(context.Background(), approverID, reqID, "approved")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestLeaveRepository_GetEmployeeLeaveBalances(t *testing.T) {
	empID := uuid.New()
	typeID := uuid.New()

	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, args ...any) (pgx.Rows, error) {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id: %s", sql)
			}
			if len(args) != 2 || args[0] != empID || args[1] != 2026 {
				t.Fatalf("expected args [empID, 2026], got %v", args)
			}
			return &leaveTestRows{
				data: [][]any{
					{typeID, "annual", "Annual Leave", true, 12, "2.00", 0, int64(3)},
				},
			}, nil
		},
	}

	repo := LeaveRepository{DB: db}
	balances, err := repo.GetEmployeeLeaveBalances(context.Background(), empID, 2026)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(balances) != 1 {
		t.Fatalf("expected 1 balance, got %d", len(balances))
	}
	b := balances[0]
	if b.Entitlement != 12 || b.Used != 3 {
		t.Fatalf("unexpected balance: %+v", b)
	}
	// Remaining: 12 + 0 + 2.00 - 3 = 11.00
	expectedRemaining := decimal.NewFromInt(11)
	if !b.Remaining.Equal(expectedRemaining) {
		t.Fatalf("expected remaining 11, got %v", b.Remaining)
	}
}

func TestLeaveRepository_ResolveEmployeeForUser(t *testing.T) {
	userID := uuid.New()
	expectedEmpID := uuid.New()

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id: %s", sql)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = expectedEmpID
				return nil
			}}
		},
	}

	repo := LeaveRepository{DB: db}
	empID, err := repo.ResolveEmployeeForUser(context.Background(), userID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if empID != expectedEmpID {
		t.Fatalf("expected empID %v, got %v", expectedEmpID, empID)
	}
}

func TestLeaveRepository_GetOrganizationLeaveBalances(t *testing.T) {
	empID := uuid.New()
	typeID := uuid.New()

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id in countSQL: %s", sql)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 1
				return nil
			}}
		},
		queryFn: func(_ context.Context, sql string, args ...any) (pgx.Rows, error) {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id in dataSQL: %s", sql)
			}
			policyName := "Standard Policy"
			assignType := "organization"
			return &leaveTestRows{
				data: [][]any{
					{empID, "Alice Smith", typeID, "annual", "Annual Leave", true, 12, "0.00", 0, 2, &policyName, &assignType},
				},
			}, nil
		},
	}

	repo := LeaveRepository{DB: db}
	balances, total, err := repo.GetOrganizationLeaveBalances(context.Background(), 2026, 1, 20, "")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if total != 1 || len(balances) != 1 {
		t.Fatalf("expected 1 item, total 1; got total=%d len=%d", total, len(balances))
	}
	if balances[0].EmployeeName != "Alice Smith" || balances[0].Used != 2 {
		t.Fatalf("unexpected balance item: %+v", balances[0])
	}
}

func TestLeaveRepository_GetEmployeeLeaveHistory(t *testing.T) {
	empID := uuid.New()
	typeID := uuid.New()
	logID := uuid.New()
	reqID := uuid.New()
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			if strings.Contains(sql, "FROM leave_accrual_logs") {
				notes := "Monthly accrual"
				assignID := uuid.New()
				return &leaveTestRows{
					data: [][]any{
						{logID, "2026-05-01", "1.00", &notes, &assignID, now},
					},
				}, nil
			}
			if strings.Contains(sql, "FROM leave_requests") {
				reason := "Personal"
				return &leaveTestRows{
					data: [][]any{
						{reqID, empID, "2026-05-10", "2026-05-11", "full_day", (*string)(nil), (*string)(nil), &reason, now},
					},
				}, nil
			}
			return &leaveTestRows{}, nil
		},
	}

	repo := LeaveRepository{DB: db}
	items, err := repo.GetEmployeeLeaveHistory(context.Background(), empID, 2026, &typeID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(items) != 2 {
		t.Fatalf("expected 2 history items, got %d", len(items))
	}
}

func TestCalculateLeaveDaysForRequest_OfficialScheduleBreaksAndHolidays(t *testing.T) {
	tUTC := func(raw string) time.Time {
		parsed, _ := time.Parse(time.RFC3339, raw)
		return parsed
	}
	tPtr := func(raw string) *time.Time { v := tUTC(raw); return &v }
	sPtr := func(raw string) *string { return &raw }

	schedule := map[string]leaveScheduleDay{
		"2026-11-22": {State: "published", Timezone: "UTC", Segments: []leaveScheduleSegment{{
			StartAt: tUTC("2026-11-22T08:00:00Z"), EndAt: tUTC("2026-11-22T17:00:00Z"),
			Breaks: []leaveScheduleBreak{
				{StartAt: tPtr("2026-11-22T12:00:00Z"), EndAt: tPtr("2026-11-22T12:45:00Z")},
				{StartAt: tPtr("2026-11-22T12:30:00Z"), EndAt: tPtr("2026-11-22T13:00:00Z")},
			},
		}}},
		"2026-11-23": {State: "explicit_off", Timezone: "UTC"},
		"2026-11-24": {State: "published", Timezone: "UTC", Segments: []leaveScheduleSegment{{
			StartAt: tUTC("2026-11-24T22:00:00Z"), EndAt: tUTC("2026-11-25T06:00:00Z"),
			Breaks: []leaveScheduleBreak{{StartAt: tPtr("2026-11-25T02:00:00Z"), EndAt: tPtr("2026-11-25T03:00:00Z")}},
		}}},
		"2026-11-25": {State: "published", Timezone: "UTC", Segments: []leaveScheduleSegment{{
			StartAt: tUTC("2026-11-25T08:00:00Z"), EndAt: tUTC("2026-11-25T17:00:00Z"),
		}}},
	}
	holidays := map[string]leaveHoliday{"2026-11-25": {HolidayType: "national", IsPaid: true}}

	cases := []struct {
		name string
		req  leaveUsageRequest
		want string
	}{
		{"sun work + mon off", leaveUsageRequest{StartDate: "2026-11-22", EndDate: "2026-11-23", DurationType: "full_day"}, "1"},
		{"mon explicit_off", leaveUsageRequest{StartDate: "2026-11-23", EndDate: "2026-11-23", DurationType: "full_day"}, "0"},
		{"national holiday", leaveUsageRequest{StartDate: "2026-11-25", EndDate: "2026-11-25", DurationType: "full_day"}, "0"},
		{"sunday half_day", leaveUsageRequest{StartDate: "2026-11-22", EndDate: "2026-11-22", DurationType: "first_half"}, "0.5"},
		{"custom across overlapping breaks", leaveUsageRequest{StartDate: "2026-11-22", EndDate: "2026-11-22", DurationType: "custom_hours", StartTime: sPtr("11:00"), EndTime: sPtr("14:00")}, "0.25"},
		{"overnight custom hours", leaveUsageRequest{StartDate: "2026-11-24", EndDate: "2026-11-24", DurationType: "custom_hours", StartTime: sPtr("01:00"), EndTime: sPtr("04:00")}, "0.2857"},
	}
	for _, tc := range cases {
		got := calculateLeaveDaysForRequest(tc.req, schedule, holidays)
		if !got.Equal(decimal.RequireFromString(tc.want)) {
			t.Fatalf("%s: got %s, want %s", tc.name, got.String(), tc.want)
		}
	}
}

func TestLeaveRepository_AdjustLeaveBalance(t *testing.T) {
	empID := uuid.New()
	typeID := uuid.New()
	employmentID := uuid.New()

	tx := &fakeTx{fakeDBTX: fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employmentID
				return nil
			}}
		},
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id: %s", sql)
			}
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}}

	repo := LeaveRepository{
		BeginTx: func(_ context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	amt, _ := decimal.NewFromString("2.00")
	err := repo.AdjustLeaveBalance(context.Background(), empID, api.AdjustBalanceRequest{
		LeaveTypeID:     typeID,
		Amount:          amt,
		Reason:          "Correction",
		TransactionDate: "2026-05-01",
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestLeaveRepository_LeavePolicies_CRUD(t *testing.T) {
	policyID := uuid.New()
	typeID := uuid.New()
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id: %s", sql)
			}
			desc := "Standard definition"
			return &leaveTestRows{
				data: [][]any{
					{policyID, "Standard Policy", &desc, typeID, []byte(`{}`), now, now},
				},
			}, nil
		},
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id: %s", sql)
			}
			desc := "Standard definition"
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = policyID
				*(dest[1].(*string)) = "Standard Policy"
				*(dest[2].(**string)) = &desc
				*(dest[3].(*uuid.UUID)) = typeID
				*(dest[4].(*[]byte)) = []byte(`{}`)
				*(dest[5].(*time.Time)) = now
				*(dest[6].(*time.Time)) = now
				return nil
			}}
		},
	}

	repo := LeaveRepository{DB: db}
	policies, err := repo.ListLeavePolicies(context.Background())
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(policies) != 1 || policies[0].ID != policyID {
		t.Fatalf("unexpected policies: %+v", policies)
	}

	created, err := repo.CreateLeavePolicy(context.Background(), api.CreateLeavePolicyRequest{
		Name:        "Standard Policy",
		LeaveTypeID: typeID,
	})
	if err != nil {
		t.Fatalf("unexpected create error: %v", err)
	}
	if created.ID != policyID {
		t.Fatalf("unexpected created policy: %+v", created)
	}
}

func TestLeaveRepository_LeavePolicyAssignments(t *testing.T) {
	assignID := uuid.New()
	policyID := uuid.New()
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id: %s", sql)
			}
			from := "2026-01-01"
			return &leaveTestRows{
				data: [][]any{
					{assignID, "organization", (*uuid.UUID)(nil), policyID, &from, (*string)(nil), now, "Standard Policy", "Annual Leave"},
				},
			}, nil
		},
	}

	repo := LeaveRepository{DB: db}
	list, err := repo.ListLeavePolicyAssignments(context.Background())
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(list) != 1 || list[0].ID != assignID {
		t.Fatalf("unexpected assignments: %+v", list)
	}
}

func TestLeaveRepository_Jobs_AccrualAndImport(t *testing.T) {
	userID := uuid.New()
	jobID := uuid.New()
	now := time.Now().UTC()

	db := &fakeDBTX{
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			if strings.Contains(strings.ToLower(sql), "workspace_id") {
				t.Fatalf("unexpected workspace_id: %s", sql)
			}
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "FROM outbox_jobs") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*string)) = "completed"
					*(dest[1].(**string)) = nil
					*(dest[2].(*time.Time)) = now
					return nil
				}}
			}
			if strings.Contains(sql, "INSERT INTO attendance_import_jobs") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = jobID
					*(dest[1].(*uuid.UUID)) = userID
					*(dest[2].(*string)) = "processing"
					*(dest[3].(*int)) = 0
					*(dest[4].(*int)) = 0
					filePath := "/tmp/test.xlsx"
					*(dest[5].(**string)) = &filePath
					*(dest[6].(**string)) = nil
					*(dest[7].(*time.Time)) = now
					*(dest[8].(*time.Time)) = now
					return nil
				}}
			}
			if strings.Contains(sql, "FROM attendance_import_jobs") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = jobID
					*(dest[1].(*uuid.UUID)) = userID
					*(dest[2].(*string)) = "completed"
					*(dest[3].(*int)) = 10
					*(dest[4].(*int)) = 10
					filePath := "/tmp/test.xlsx"
					*(dest[5].(**string)) = &filePath
					*(dest[6].(**string)) = nil
					*(dest[7].(*time.Time)) = now
					*(dest[8].(*time.Time)) = now
					return nil
				}}
			}
			return fakeRow{}
		},
	}

	repo := LeaveRepository{DB: db}

	// Accrual job
	createdJobID, err := repo.CreateAccrualJob(context.Background(), userID, "2026-05-01")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if createdJobID == uuid.Nil {
		t.Fatal("expected non-nil job id")
	}

	status, err := repo.GetJobStatus(context.Background(), createdJobID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if status.Status != "completed" {
		t.Fatalf("expected completed status, got %s", status.Status)
	}

	// Import job
	importJob, err := repo.CreateLeaveImportJob(context.Background(), userID, "/tmp/test.xlsx")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if importJob.ID != jobID || importJob.UserID != userID {
		t.Fatalf("unexpected import job: %+v", importJob)
	}

	latest, err := repo.GetLatestLeaveImport(context.Background())
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if latest == nil || latest.ID != jobID {
		t.Fatalf("unexpected latest import: %+v", latest)
	}
}
