package repository

import (
	"context"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestEmployeeDepartmentRepository_TransferPreservesRelationshipOnSegmentSplit(t *testing.T) {
	employeeID := uuid.New()
	employerID := uuid.New()
	relationshipID := uuid.New()
	oldDepartmentID := uuid.New()
	newDepartmentID := uuid.New()
	segmentID := uuid.New()

	lockedEmployee := false
	insertArgs := []any(nil)
	tx := &fakeTx{}
	tx.fakeDBTX.queryRowFn = func(_ context.Context, query string, args ...any) pgx.Row {
		switch {
		case strings.Contains(query, "SELECT EXISTS") && strings.Contains(query, "FROM employees"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = true
				return nil
			}}
		case strings.Contains(query, "FOR UPDATE"):
			lockedEmployee = true
			if len(args) != 1 || args[0] != employeeID {
				t.Fatalf("employee lock args = %v, want %s", args, employeeID)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employeeID
				return nil
			}}
		case strings.Contains(query, "FROM departments"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = true
				return nil
			}}
		case strings.Contains(query, "FROM payroll_periods"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = false
				return nil
			}}
		default:
			t.Fatalf("unexpected query row: %s", query)
			return fakeRow{}
		}
	}
	tx.fakeDBTX.queryFn = func(_ context.Context, query string, args ...any) (pgx.Rows, error) {
		if !strings.Contains(query, "relationship_id") || !strings.Contains(query, "ORDER BY start_date ASC, id ASC") {
			t.Fatalf("segment query must select relationship_id with stable ordering: %s", query)
		}
		if len(args) != 1 || args[0] != employeeID {
			t.Fatalf("segment query args = %v, want %s", args, employeeID)
		}
		return &rowsStub{data: [][]any{{
			segmentID,
			employeeID,
			employerID,
			&relationshipID,
			nil,
			&oldDepartmentID,
			"active",
			"2026-01-01",
			nil,
			nil,
			"full_time",
			nil,
			nil,
			nil,
		}}}, nil
	}
	tx.fakeDBTX.execFn = func(_ context.Context, query string, args ...any) (pgconn.CommandTag, error) {
		if strings.Contains(query, "INSERT INTO employments") {
			if !strings.Contains(query, "relationship_id") {
				t.Fatalf("split insert missing relationship_id: %s", query)
			}
			insertArgs = args
		}
		return pgconn.NewCommandTag("UPDATE 1"), nil
	}

	repo := EmployeeDepartmentRepository{
		BeginTx: func(context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}
	result, err := repo.TransferEmployee(context.Background(), employeeID, api.TransferEmployeeRequest{
		DepartmentID:  newDepartmentID,
		EffectiveDate: "2026-04-01",
	})
	if err != nil {
		t.Fatalf("transfer employee: %v", err)
	}
	if !lockedEmployee {
		t.Fatal("transfer did not lock employee row")
	}
	if len(insertArgs) < 4 || insertArgs[3] != &relationshipID {
		t.Fatalf("split insert relationship argument = %v, want %s", insertArgs, relationshipID)
	}
	if result.EmployeeID != employeeID || result.DepartmentID != newDepartmentID {
		t.Fatalf("unexpected transfer result: %+v", result)
	}
}
