package repository

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type employmentRelationshipTestTx struct {
	fakeDBTX
	committed  bool
	rolledBack bool
}

func (tx *employmentRelationshipTestTx) Commit(context.Context) error {
	tx.committed = true
	return nil
}

func (tx *employmentRelationshipTestTx) Rollback(context.Context) error {
	tx.rolledBack = true
	return nil
}

func (tx *employmentRelationshipTestTx) Begin(context.Context) (pgx.Tx, error) { return tx, nil }
func (tx *employmentRelationshipTestTx) CopyFrom(context.Context, pgx.Identifier, []string, pgx.CopyFromSource) (int64, error) {
	return 0, nil
}
func (tx *employmentRelationshipTestTx) SendBatch(context.Context, *pgx.Batch) pgx.BatchResults {
	return nil
}
func (tx *employmentRelationshipTestTx) LargeObjects() pgx.LargeObjects { return pgx.LargeObjects{} }
func (tx *employmentRelationshipTestTx) Prepare(context.Context, string, string) (*pgconn.StatementDescription, error) {
	return nil, nil
}
func (tx *employmentRelationshipTestTx) Conn() *pgx.Conn { return nil }

func TestEmploymentRepository_CreateRelationshipCommitsRelationshipAndSegment(t *testing.T) {
	employeeID := uuid.New()
	employerID := uuid.New()
	relationshipID := uuid.New()
	segmentID := uuid.New()
	tx := newEmploymentRelationshipCreateTx(employeeID, employerID, relationshipID, segmentID, nil)

	repo := EmploymentRepository{
		BeginTx: func(context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}
	result, err := repo.CreateEmploymentRelationship(context.Background(), employeeID, api.CreateEmploymentRelationshipRequest{
		EmployerID:     employerID,
		Status:         "active",
		StartDate:      "2026-01-01",
		EmploymentType: "full_time",
	})
	if err != nil {
		t.Fatalf("create relationship: %v", err)
	}
	if !tx.committed || tx.rolledBack {
		t.Fatalf("transaction state committed/rolled_back = %t/%t, want true/false", tx.committed, tx.rolledBack)
	}
	if result.Relationship.ID == uuid.Nil || result.Segment.ID == uuid.Nil {
		t.Fatalf("create result missing relationship or segment IDs: %+v", result)
	}
	if result.Relationship.EmployeeID != employeeID || result.Relationship.EmployerID != employerID {
		t.Fatalf("relationship identity = %s/%s, want %s/%s", result.Relationship.EmployeeID, result.Relationship.EmployerID, employeeID, employerID)
	}
}

func TestEmploymentRepository_CreateRelationshipRollsBackWhenSegmentInsertFails(t *testing.T) {
	employeeID := uuid.New()
	employerID := uuid.New()
	segmentErr := errors.New("segment insert failed")
	tx := newEmploymentRelationshipCreateTx(employeeID, employerID, uuid.New(), uuid.New(), segmentErr)

	repo := EmploymentRepository{
		BeginTx: func(context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}
	_, err := repo.CreateEmploymentRelationship(context.Background(), employeeID, api.CreateEmploymentRelationshipRequest{
		EmployerID:     employerID,
		Status:         "active",
		StartDate:      "2026-01-01",
		EmploymentType: "full_time",
	})
	if !errors.Is(err, segmentErr) {
		t.Fatalf("create error = %v, want %v", err, segmentErr)
	}
	if tx.committed || !tx.rolledBack {
		t.Fatalf("transaction state committed/rolled_back = %t/%t, want false/true", tx.committed, tx.rolledBack)
	}
}

func TestEmploymentRepository_CreateRelationshipMissingEmployeeDoesNotCommit(t *testing.T) {
	employeeID := uuid.New()
	employerID := uuid.New()
	tx := newEmploymentRelationshipCreateTx(employeeID, employerID, uuid.New(), uuid.New(), nil)
	tx.fakeDBTX.queryRowFn = func(_ context.Context, query string, _ ...any) pgx.Row {
		if strings.Contains(query, "FROM employees") && strings.Contains(query, "EXISTS") {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = false
				return nil
			}}
		}
		t.Fatalf("unexpected query after missing employee: %s", query)
		return fakeRow{}
	}

	repo := EmploymentRepository{
		BeginTx: func(context.Context) (TxBeginner, error) {
			return fakeTxBeginner{tx: tx}, nil
		},
	}
	_, err := repo.CreateEmploymentRelationship(context.Background(), employeeID, api.CreateEmploymentRelationshipRequest{
		EmployerID:     employerID,
		Status:         "active",
		StartDate:      "2026-01-01",
		EmploymentType: "full_time",
	})
	if !errors.Is(err, api.ErrEmployeeNotFound) {
		t.Fatalf("create error = %v, want %v", err, api.ErrEmployeeNotFound)
	}
	if tx.committed || !tx.rolledBack {
		t.Fatalf("transaction state committed/rolled_back = %t/%t, want false/true", tx.committed, tx.rolledBack)
	}
}

func newEmploymentRelationshipCreateTx(employeeID, employerID, relationshipID, segmentID uuid.UUID, segmentErr error) *employmentRelationshipTestTx {
	tx := &employmentRelationshipTestTx{}
	tx.fakeDBTX.queryRowFn = func(_ context.Context, query string, args ...any) pgx.Row {
		switch {
		case strings.Contains(query, "FROM employees") && strings.Contains(query, "EXISTS"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = true
				return nil
			}}
		case strings.Contains(query, "FROM employers") && strings.Contains(query, "EXISTS"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = true
				return nil
			}}
		case strings.Contains(query, "FROM employee_tenures") && strings.Contains(query, "FOR UPDATE"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = uuid.New()
				return nil
			}}
		case strings.Contains(query, "FOR UPDATE"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = employeeID
				return nil
			}}
		case strings.Contains(query, "INSERT INTO employment_relationships"):
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = relationshipID
				*(dest[1].(*uuid.UUID)) = employeeID
				*(dest[2].(*uuid.UUID)) = employerID
				*(dest[3].(*uuid.UUID)) = uuid.New()
				*(dest[4].(*string)) = "active"
				*(dest[5].(*string)) = "2026-01-01"
				*(dest[6].(**string)) = nil
				*(dest[7].(**string)) = nil
				*(dest[8].(**string)) = nil
				*(dest[9].(*time.Time)) = time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
				*(dest[10].(*time.Time)) = time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
				return nil
			}}
		case strings.Contains(query, "INSERT INTO employments"):
			return fakeRow{scanFn: func(dest ...any) error {
				if segmentErr != nil {
					return segmentErr
				}
				*(dest[0].(*uuid.UUID)) = segmentID
				*(dest[1].(*uuid.UUID)) = employeeID
				*(dest[2].(*uuid.UUID)) = employerID
				*(dest[3].(**uuid.UUID)) = nil
				*(dest[4].(**uuid.UUID)) = nil
				*(dest[5].(*string)) = "active"
				*(dest[6].(*string)) = "2026-01-01"
				*(dest[7].(**string)) = nil
				*(dest[8].(**string)) = nil
				*(dest[9].(*string)) = "full_time"
				*(dest[10].(**uuid.UUID)) = nil
				*(dest[11].(*time.Time)) = time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
				*(dest[12].(*time.Time)) = time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
				*(dest[13].(**string)) = nil
				*(dest[14].(**string)) = nil
				return nil
			}}
		default:
			panic("unexpected query: " + query)
		}
	}
	return tx
}
