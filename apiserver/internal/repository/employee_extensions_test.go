package repository

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type rowsStub struct {
	data  [][]any
	index int
}

func (r *rowsStub) Close()                                       {}
func (r *rowsStub) Err() error                                   { return nil }
func (r *rowsStub) CommandTag() pgconn.CommandTag                { return pgconn.CommandTag{} }
func (r *rowsStub) FieldDescriptions() []pgconn.FieldDescription { return nil }
func (r *rowsStub) Next() bool {
	r.index++
	return r.index <= len(r.data)
}
func (r *rowsStub) Scan(dest ...any) error {
	row := r.data[r.index-1]
	for i, v := range row {
		switch target := dest[i].(type) {
		case *uuid.UUID:
			*target = v.(uuid.UUID)
		case **uuid.UUID:
			if v == nil {
				*target = nil
			} else if ptr, ok := v.(*uuid.UUID); ok {
				*target = ptr
			} else {
				val := v.(uuid.UUID)
				*target = &val
			}
		case *string:
			*target = v.(string)
		case **string:
			if v == nil {
				*target = nil
			} else if ptr, ok := v.(*string); ok {
				*target = ptr
			} else {
				val := v.(string)
				*target = &val
			}
		case *int:
			*target = v.(int)
		case *int64:
			*target = v.(int64)
		case *bool:
			*target = v.(bool)
		case *time.Time:
			*target = v.(time.Time)
		case *decimal.Decimal:
			*target = v.(decimal.Decimal)
		}
	}
	return nil
}
func (r *rowsStub) Values() ([]any, error) { return r.data[r.index-1], nil }
func (r *rowsStub) RawValues() [][]byte    { return nil }
func (r *rowsStub) Conn() *pgx.Conn        { return nil }

type fakeTxBeginner struct {
	tx pgx.Tx
}

func (b fakeTxBeginner) Begin(context.Context) (pgx.Tx, error) {
	return b.tx, nil
}

type fakeTx struct {
	fakeDBTX
	committed bool
}

func (f *fakeTx) Commit(context.Context) error {
	f.committed = true
	return nil
}
func (f *fakeTx) Rollback(context.Context) error        { return nil }
func (f *fakeTx) Begin(context.Context) (pgx.Tx, error) { return f, nil }
func (f *fakeTx) CopyFrom(context.Context, pgx.Identifier, []string, pgx.CopyFromSource) (int64, error) {
	return 0, nil
}
func (f *fakeTx) SendBatch(context.Context, *pgx.Batch) pgx.BatchResults { return nil }
func (f *fakeTx) LargeObjects() pgx.LargeObjects                         { return pgx.LargeObjects{} }
func (f *fakeTx) Prepare(context.Context, string, string) (*pgconn.StatementDescription, error) {
	return nil, nil
}
func (f *fakeTx) Conn() *pgx.Conn { return nil }

func TestEmployeeConditionRepository_ListAndCreate(t *testing.T) {
	wsID, empID, employmentID, condID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
			if strings.Contains(sql, "FROM employees") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*bool)) = true
					return nil
				}}
			}
			if strings.Contains(sql, "INSERT INTO working_conditions") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*string)) = "2026-01-01"
					*(dest[1].(*string)) = "2026-12-31"
					return nil
				}}
			}
			if strings.Contains(sql, "FROM employments") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = employmentID
					return nil
				}}
			}
			if strings.Contains(sql, "count(*)") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*int64)) = 1
					return nil
				}}
			}
			return fakeRow{}
		},
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			if strings.Contains(sql, "FROM working_conditions") {
				return &rowsStub{
					data: [][]any{
						{condID, employmentID, "EARLY_LEAVE", nil, 60, true, "2026-01-01", nil},
					},
				}, nil
			}
			return &rowsStub{}, nil
		},
	}

	repo := EmployeeConditionRepository{DB: db}

	// 1. List
	res, err := repo.ListEmployeeConditions(context.Background(), wsID, empID, 1, 50)
	if err != nil {
		t.Fatalf("unexpected list error: %v", err)
	}
	if res.Total != 1 || len(res.Data) != 1 || res.Data[0].Type != "EARLY_LEAVE" {
		t.Fatalf("unexpected list response: %+v", res)
	}

	// 2. Create
	created, err := repo.CreateEmployeeCondition(context.Background(), wsID, empID, api.CreateEmployeeConditionRequest{
		Type:            "LATE_IN",
		MinuteAllowance: 15,
		IsPaid:          false,
		StartDate:       "2026-01-01",
	})
	if err != nil {
		t.Fatalf("unexpected create error: %v", err)
	}
	if created.Type != "LATE_IN" || created.EmploymentID != employmentID {
		t.Fatalf("unexpected created condition: %+v", created)
	}
}

func TestEmployeeDepartmentRepository_TransferSplit(t *testing.T) {
	wsID, empID, oldDeptID, newDeptID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	activeSegID := uuid.New()
	nextSegID := uuid.New()

	var updatedEndDate string
	var insertedNewDept uuid.UUID
	var cascadedDeptID uuid.UUID

	tx := &fakeTx{
		fakeDBTX: fakeDBTX{
			queryRowFn: func(_ context.Context, sql string, _ ...any) pgx.Row {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*bool)) = true
					return nil
				}}
			},
			queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
				if strings.Contains(sql, "FROM employments") {
					end1 := "2026-06-30"
					return &rowsStub{
						data: [][]any{
							{activeSegID, wsID, empID, uuid.New(), nil, oldDeptID, "active", "2026-01-01", end1, nil, "full_time", nil, nil, nil},
							{nextSegID, wsID, empID, uuid.New(), nil, oldDeptID, "active", "2026-07-01", nil, nil, "full_time", nil, nil, nil},
						},
					}, nil
				}
				return &rowsStub{}, nil
			},
			execFn: func(_ context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
				if strings.Contains(sql, "SET end_date = $1::date") {
					updatedEndDate = args[0].(string)
				}
				if strings.Contains(sql, "INSERT INTO employments") {
					insertedNewDept = args[5].(uuid.UUID)
				}
				if strings.Contains(sql, "UPDATE employments SET department_id = $1") {
					cascadedDeptID = args[0].(uuid.UUID)
				}
				return pgconn.NewCommandTag("UPDATE 1"), nil
			},
		},
	}

	repo := EmployeeDepartmentRepository{
		BeginTx: func(context.Context, uuid.UUID) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}

	res, err := repo.TransferEmployee(context.Background(), wsID, empID, api.TransferEmployeeRequest{
		DepartmentID:  newDeptID,
		EffectiveDate: "2026-03-01",
	})
	if err != nil {
		t.Fatalf("transfer failed: %v", err)
	}

	// EffectiveDate is 2026-03-01; previous end date must be 2026-02-28
	if updatedEndDate != "2026-02-28" {
		t.Fatalf("expected updated end_date 2026-02-28, got %s", updatedEndDate)
	}
	if insertedNewDept != newDeptID {
		t.Fatalf("expected inserted department %s, got %s", newDeptID, insertedNewDept)
	}
	if cascadedDeptID != newDeptID {
		t.Fatalf("expected cascaded next segment department %s, got %s", newDeptID, cascadedDeptID)
	}
	if res.DepartmentID != newDeptID || res.EffectiveDate != "2026-03-01" {
		t.Fatalf("unexpected history result: %+v", res)
	}
}

func TestCompensationHistoryRepository_Timeline(t *testing.T) {
	wsID, empID, emp1ID, emp2ID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	employerID, deptID := uuid.New(), uuid.New()
	baseDefID, bonusDefID := uuid.New(), uuid.New()
	now := time.Now().UTC()

	db := &fakeDBTX{
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			if strings.Contains(sql, "FROM employments") {
				return &rowsStub{
					data: [][]any{
						{emp1ID, employerID, deptID, nil, "2026-01-01"},
						{emp2ID, employerID, deptID, nil, "2026-04-01"},
					},
				}, nil
			}
			if strings.Contains(sql, "FROM employers") {
				return &rowsStub{
					data: [][]any{
						{employerID, "Acme Corp"},
					},
				}, nil
			}
			if strings.Contains(sql, "FROM departments") {
				return &rowsStub{
					data: [][]any{
						{deptID, "Engineering"},
					},
				}, nil
			}
			if strings.Contains(sql, "FROM employee_salary_components") {
				return &rowsStub{
					data: [][]any{
						// emp1: initial base pay 10,000,000 on 2026-01-01
						{uuid.New(), emp1ID, baseDefID, "10000000", "2026-01-01", now, now, baseDefID, wsID, "Base Salary", "BASE", now, now},
						// emp1: adjustment add bonus 2,000,000 on 2026-02-01
						{uuid.New(), emp1ID, bonusDefID, "2000000", "2026-02-01", now, now, bonusDefID, wsID, "Bonus", "BONUS", now, now},
						// emp2: transfer on 2026-04-01 with base pay 15,000,000
						{uuid.New(), emp2ID, baseDefID, "15000000", "2026-04-01", now, now, baseDefID, wsID, "Base Salary", "BASE", now, now},
					},
				}, nil
			}
			return &rowsStub{}, nil
		},
	}

	repo := CompensationHistoryRepository{DB: db}

	res, err := repo.GetEmployeeCompensationHistory(context.Background(), wsID, empID, nil)
	if err != nil {
		t.Fatalf("get compensation history failed: %v", err)
	}

	// Rust returns newest first -> 3 events
	if len(res.Timeline) != 3 {
		t.Fatalf("expected 3 events, got %d", len(res.Timeline))
	}

	// Newest event: 2026-04-01 (transfer to emp2)
	e0 := res.Timeline[0]
	if e0.EffectiveDate != "2026-04-01" || e0.ChangeType != "transfer" {
		t.Fatalf("expected transfer on 2026-04-01, got: %+v", e0)
	}
	if !e0.TotalCompensation.Equal(decimal.NewFromInt(15000000)) {
		t.Fatalf("expected total 15,000,000, got: %s", e0.TotalCompensation.String())
	}
	// Previous total was 12,000,000 -> delta = 3,000,000
	if !e0.ChangeDetails.Delta.Equal(decimal.NewFromInt(3000000)) {
		t.Fatalf("expected delta 3,000,000, got: %s", e0.ChangeDetails.Delta.String())
	}

	// Middle event: 2026-02-01 (adjustment in emp1: 10M + 2M = 12M)
	e1 := res.Timeline[1]
	if e1.EffectiveDate != "2026-02-01" || e1.ChangeType != "adjustment" {
		t.Fatalf("expected adjustment on 2026-02-01, got: %+v", e1)
	}
	if !e1.TotalCompensation.Equal(decimal.NewFromInt(12000000)) {
		t.Fatalf("expected total 12,000,000, got: %s", e1.TotalCompensation.String())
	}
	if !e1.ChangeDetails.Delta.Equal(decimal.NewFromInt(2000000)) {
		t.Fatalf("expected delta 2,000,000, got: %s", e1.ChangeDetails.Delta.String())
	}

	// Oldest event: 2026-01-01 (initial in emp1: 10M)
	e2 := res.Timeline[2]
	if e2.EffectiveDate != "2026-01-01" || e2.ChangeType != "initial" {
		t.Fatalf("expected initial on 2026-01-01, got: %+v", e2)
	}
	if !e2.TotalCompensation.Equal(decimal.NewFromInt(10000000)) {
		t.Fatalf("expected total 10,000,000, got: %s", e2.TotalCompensation.String())
	}
}
