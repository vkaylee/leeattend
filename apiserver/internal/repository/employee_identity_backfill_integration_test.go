//go:build integration

package repository

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

const testIdentityBackfillSecret = "0123456789abcdef0123456789abcdef"

func TestIntegrationEmployeeIdentityBackfillIdempotentAndCollisionAware(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	// Clean any pre-existing profiles in transaction scope so aggregate counts are deterministic.
	if _, err := tx.Exec(ctx, `DELETE FROM employee_profiles`); err != nil {
		t.Fatalf("clear employee_profiles in tx: %v", err)
	}

	rawNID1 := "001-234.567"
	rawNID1Variant := "００１ 234/567" // Same canonical national ID -> collision!
	rawNIDUnique := "099-888-777"
	rawPhoneE164 := "+84 912-345-678"
	rawPhoneE164Variant := "+84912345678" // Same E.164 -> collision!
	rawPhoneInvalid := "0912345678"       // National-only -> rejected, hash stays NULL.

	fixtures := []struct {
		code       string
		phone      *string
		nationalID *string
	}{
		{"EMP-ID-1", &rawPhoneE164, &rawNID1},
		{"EMP-ID-2", &rawPhoneE164Variant, &rawNID1Variant},
		{"EMP-ID-3", &rawPhoneInvalid, &rawNIDUnique},
		{"EMP-ID-4", nil, nil},
	}

	for _, f := range fixtures {
		empID := uuid.New()
		if _, err := tx.Exec(
			ctx,
			`INSERT INTO employees (id, full_name)
			 VALUES ($1, 'identity-backfill-test')`,
			empID,
		); err != nil {
			t.Fatalf("insert employee %s: %v", f.code, err)
		}
		if _, err := tx.Exec(
			ctx,
			`INSERT INTO employee_tenures (id, employee_id, employee_number, hire_type, start_date)
			 VALUES ($1, $2, $3, 'initial', DATE '2026-01-01')`,
			uuid.New(),
			empID,
			f.code,
		); err != nil {
			t.Fatalf("insert tenure %s: %v", f.code, err)
		}
		if _, err := tx.Exec(
			ctx,
			`INSERT INTO employee_profiles (id, employee_id, phone, national_id)
			 VALUES ($1, $2, $3, $4)`,
			uuid.New(),
			empID,
			f.phone,
			f.nationalID,
		); err != nil {
			t.Fatalf("insert profile %s: %v", f.code, err)
		}
	}

	repo := EmployeeIdentityRepository{
		DB: tx,
		BeginTx: func(context.Context) (service.TxBeginner, error) {
			return tx, nil
		},
	}

	// Secret missing must fail closed before touching DB.
	if _, err := repo.BackfillIdentityLookupHashes(ctx, identity.Hasher{}, IdentityBackfillOptions{BatchSize: 2}); !errors.Is(err, identity.ErrIdentityHashSecretMissing) {
		t.Fatalf("missing secret err = %v, want ErrIdentityHashSecretMissing", err)
	}

	hasher, err := identity.NewHasher(testIdentityBackfillSecret)
	if err != nil {
		t.Fatalf("NewHasher: %v", err)
	}

	// Dry run computes counts and collisions without mutating rows.
	dryReport, err := repo.BackfillIdentityLookupHashes(ctx, hasher, IdentityBackfillOptions{BatchSize: 2, DryRun: true})
	if err != nil {
		t.Fatalf("dry-run backfill: %v", err)
	}
	if !dryReport.DryRun || dryReport.Batches != 2 || dryReport.Scanned != 4 || dryReport.Updated != 3 || dryReport.Unchanged != 1 {
		t.Fatalf("unexpected dry-run report: %+v", dryReport)
	}
	if dryReport.SkippedInvalidPhone != 1 || dryReport.NationalIDCollisionGroups != 1 || dryReport.NationalIDCollisionRows != 2 || dryReport.PhoneCollisionGroups != 1 || dryReport.PhoneCollisionRows != 2 || dryReport.UniqueIndexReady {
		t.Fatalf("unexpected dry-run collision/skip report: %+v", dryReport)
	}
	readinessBefore, err := repo.IdentityReadiness(ctx)
	if err != nil {
		t.Fatalf("readiness before write: %v", err)
	}
	if readinessBefore.WithNationalIDHash != 0 || readinessBefore.WithPhoneHash != 0 {
		t.Fatalf("dry run must not persist hashes: %+v", readinessBefore)
	}

	// First real run populates hashes in bounded batches and detects collisions without choosing a winner.
	firstReport, err := repo.BackfillIdentityLookupHashes(ctx, hasher, IdentityBackfillOptions{BatchSize: 2})
	if err != nil {
		t.Fatalf("first backfill: %v", err)
	}
	if firstReport.DryRun || firstReport.Batches != 2 || firstReport.Scanned != 4 || firstReport.Updated != 3 || firstReport.Unchanged != 1 || firstReport.SkippedInvalidPhone != 1 {
		t.Fatalf("unexpected first backfill report: %+v", firstReport)
	}
	if firstReport.NationalIDCollisionGroups != 1 || firstReport.NationalIDCollisionRows != 2 || firstReport.PhoneCollisionGroups != 1 || firstReport.PhoneCollisionRows != 2 {
		t.Fatalf("expected 1 national_id and 1 phone collision group (2 rows each), got: %+v", firstReport)
	}
	if firstReport.UniqueIndexReady || firstReport.UniqueNationalIDIndexReady || firstReport.UniquePhoneIndexReady {
		t.Fatalf("unique index must remain blocked while collisions exist: %+v", firstReport)
	}

	// Verify aggregate report JSON contains no raw PII or hashes.
	encoded, err := json.Marshal(firstReport)
	if err != nil {
		t.Fatalf("marshal report: %v", err)
	}
	for _, sensitive := range []string{"001-234", "001234567", "+84912345678", "0912345678", testIdentityBackfillSecret} {
		if strings.Contains(string(encoded), sensitive) {
			t.Fatalf("report JSON leaked sensitive value %q: %s", sensitive, string(encoded))
		}
	}

	// Verify raw columns remained completely untouched.
	var untouchedRaw string
	if err := tx.QueryRow(ctx, `SELECT national_id FROM employee_profiles WHERE phone = $1`, rawPhoneInvalid).Scan(&untouchedRaw); err != nil {
		t.Fatalf("read untouched raw column: %v", err)
	}
	if untouchedRaw != rawNIDUnique {
		t.Fatalf("raw national_id mutated: got %q, want %q", untouchedRaw, rawNIDUnique)
	}

	// Second run is strictly idempotent: zero updates, zero drift.
	secondReport, err := repo.BackfillIdentityLookupHashes(ctx, hasher, IdentityBackfillOptions{BatchSize: 2})
	if err != nil {
		t.Fatalf("second backfill: %v", err)
	}
	if secondReport.Updated != 0 || secondReport.Unchanged != 4 || secondReport.Scanned != 4 {
		t.Fatalf("rerun must be idempotent with 0 updates, got: %+v", secondReport)
	}

	// Check constraint rejects malformed lookup hash values.
	var sp pgx.Tx
	sp, err = tx.Begin(ctx)
	if err != nil {
		t.Fatalf("begin savepoint: %v", err)
	}
	if _, err := sp.Exec(ctx, `UPDATE employee_profiles SET national_id_lookup_hash = 'not-a-valid-hex-hash' WHERE id = (SELECT id FROM employee_profiles LIMIT 1)`); err == nil {
		t.Fatal("expected invalid lookup hash format to fail check constraint")
	}
	_ = sp.Rollback(ctx)
}
