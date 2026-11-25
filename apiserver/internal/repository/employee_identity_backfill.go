package repository

import (
	"context"
	"errors"
	"fmt"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

const (
	defaultIdentityBackfillBatchSize = 200
	maxIdentityBackfillBatchSize     = 1000
)

var (
	ErrIdentityBackfillInvalidBatchSize = errors.New("identity backfill batch size must be between 1 and 1000")
)

// IdentityBackfillOptions configures bounded idempotent lookup-hash population.
type IdentityBackfillOptions struct {
	BatchSize int
	DryRun    bool
}

// IdentityBackfillReport contains aggregate counts and readiness flags only (never PII or hashes).
type IdentityBackfillReport struct {
	DryRun                     bool `json:"dry_run"`
	Batches                    int  `json:"batches"`
	Scanned                    int  `json:"scanned"`
	Updated                    int  `json:"updated"`
	Unchanged                  int  `json:"unchanged"`
	ClearedEmpty               int  `json:"cleared_empty"`
	SkippedInvalidPhone        int  `json:"skipped_invalid_phone"`
	NationalIDCollisionGroups  int  `json:"national_id_collision_groups"`
	NationalIDCollisionRows    int  `json:"national_id_collision_rows"`
	PhoneCollisionGroups       int  `json:"phone_collision_groups"`
	PhoneCollisionRows         int  `json:"phone_collision_rows"`
	UniqueNationalIDIndexReady bool `json:"unique_national_id_index_ready"`
	UniquePhoneIndexReady      bool `json:"unique_phone_index_ready"`
	UniqueIndexReady           bool `json:"unique_index_ready"`
}

// IdentityReadinessReport contains aggregate lookup coverage and collision counts only.
// ponytail: derive collision queue from non-unique hash groups in employee_profiles; add a dedicated triage table only if HR needs per-collision assignment/notes.
type IdentityReadinessReport struct {
	TotalProfiles              int  `json:"total_profiles"`
	WithRawNationalID          int  `json:"with_raw_national_id"`
	WithNationalIDHash         int  `json:"with_national_id_hash"`
	WithRawPhone               int  `json:"with_raw_phone"`
	WithPhoneHash              int  `json:"with_phone_hash"`
	NationalIDCollisionGroups  int  `json:"national_id_collision_groups"`
	NationalIDCollisionRows    int  `json:"national_id_collision_rows"`
	PhoneCollisionGroups       int  `json:"phone_collision_groups"`
	PhoneCollisionRows         int  `json:"phone_collision_rows"`
	UniqueNationalIDIndexReady bool `json:"unique_national_id_index_ready"`
	UniquePhoneIndexReady      bool `json:"unique_phone_index_ready"`
	UniqueIndexReady           bool `json:"unique_index_ready"`
}

// EmployeeIdentityRepository manages employee profile identity lookup hashes and readiness checks.
type EmployeeIdentityRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (service.TxBeginner, error)
}

func (r EmployeeIdentityRepository) db() (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("employee identity repository: database is not configured")
	}
	return r.DB, nil
}

func (r EmployeeIdentityRepository) txBeginner(ctx context.Context) (service.TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := r.db()
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return nil, errors.New("employee identity repository: database does not support transactions")
	}
	return beginner, nil
}

type identityBackfillRow struct {
	id                   uuid.UUID
	phone                *string
	nationalID           *string
	phoneLookupHash      *string
	nationalIDLookupHash *string
}

type identityHashUpdate struct {
	id                   uuid.UUID
	nationalIDLookupHash *string
	phoneLookupHash      *string
}

// BackfillIdentityLookupHashes normalizes and HMAC-hashes existing employee_profiles phone and
// national_id values in bounded, idempotent batches. Missing secret fails closed before DB access.
// Collisions are never auto-resolved; aggregate collision counts block unique-index activation.
func (r EmployeeIdentityRepository) BackfillIdentityLookupHashes(
	ctx context.Context,
	hasher identity.Hasher,
	opts IdentityBackfillOptions,
) (IdentityBackfillReport, error) {
	batchSize, db, beginner, err := r.prepareIdentityBackfill(ctx, hasher, opts)
	if err != nil {
		return IdentityBackfillReport{}, err
	}

	report := IdentityBackfillReport{DryRun: opts.DryRun}
	dryCounts := newDryCollisionTracker(opts.DryRun)
	lastID := uuid.Nil

	for {
		batch, err := r.fetchIdentityBackfillBatch(ctx, db, lastID, batchSize)
		if err != nil {
			return IdentityBackfillReport{}, err
		}
		if len(batch) == 0 {
			break
		}
		report.Batches++
		updates, err := planIdentityBackfillBatch(batch, hasher, &report, dryCounts)
		if err != nil {
			return IdentityBackfillReport{}, err
		}
		if !opts.DryRun && len(updates) > 0 {
			if err := applyIdentityBackfillBatch(ctx, beginner, updates); err != nil {
				return IdentityBackfillReport{}, err
			}
		}
		lastID = batch[len(batch)-1].id
	}

	if err := r.finalizeBackfillCollisions(ctx, &report, dryCounts); err != nil {
		return IdentityBackfillReport{}, err
	}
	return report, nil
}

func (r EmployeeIdentityRepository) prepareIdentityBackfill(
	ctx context.Context,
	hasher identity.Hasher,
	opts IdentityBackfillOptions,
) (int, DBTX, service.TxBeginner, error) {
	if err := hasher.Validate(); err != nil {
		return 0, nil, nil, err
	}
	batchSize := opts.BatchSize
	if batchSize == 0 {
		batchSize = defaultIdentityBackfillBatchSize
	}
	if batchSize < 0 || batchSize > maxIdentityBackfillBatchSize {
		return 0, nil, nil, ErrIdentityBackfillInvalidBatchSize
	}
	db, err := r.db()
	if err != nil {
		return 0, nil, nil, err
	}
	if opts.DryRun {
		return batchSize, db, nil, nil
	}
	beginner, err := r.txBeginner(ctx)
	if err != nil {
		return 0, nil, nil, err
	}
	return batchSize, db, beginner, nil
}

type dryCollisionTracker struct {
	nid   map[string]int
	phone map[string]int
}

func newDryCollisionTracker(enabled bool) *dryCollisionTracker {
	if !enabled {
		return nil
	}
	return &dryCollisionTracker{
		nid:   make(map[string]int),
		phone: make(map[string]int),
	}
}

func (t *dryCollisionTracker) record(nid, phone *string) {
	if t == nil {
		return
	}
	if nid != nil {
		t.nid[*nid]++
	}
	if phone != nil {
		t.phone[*phone]++
	}
}

func planIdentityBackfillBatch(
	batch []identityBackfillRow,
	hasher identity.Hasher,
	report *IdentityBackfillReport,
	dryCounts *dryCollisionTracker,
) ([]identityHashUpdate, error) {
	updates := make([]identityHashUpdate, 0, len(batch))
	for _, row := range batch {
		report.Scanned++
		newNID, err := computeNationalIDHash(hasher, row.nationalID)
		if err != nil {
			return nil, err
		}
		newPhone, invalidPhone, err := computePhoneHash(hasher, row.phone)
		if err != nil {
			return nil, err
		}
		if invalidPhone {
			report.SkippedInvalidPhone++
		}
		dryCounts.record(newNID, newPhone)

		if sameOptionalString(row.nationalIDLookupHash, newNID) && sameOptionalString(row.phoneLookupHash, newPhone) {
			report.Unchanged++
			continue
		}
		if clearedLookupHash(row.nationalIDLookupHash, newNID) || clearedLookupHash(row.phoneLookupHash, newPhone) {
			report.ClearedEmpty++
		}
		report.Updated++
		updates = append(updates, identityHashUpdate{
			id:                   row.id,
			nationalIDLookupHash: newNID,
			phoneLookupHash:      newPhone,
		})
	}
	return updates, nil
}

func clearedLookupHash(oldHash, newHash *string) bool {
	return oldHash != nil && newHash == nil
}

func applyIdentityBackfillBatch(
	ctx context.Context,
	beginner service.TxBeginner,
	updates []identityHashUpdate,
) error {
	return service.WithTx(ctx, beginner, func(txCtx context.Context, tx pgx.Tx) error {
		for _, u := range updates {
			if _, execErr := tx.Exec(
				txCtx,
				`UPDATE employee_profiles
				 SET national_id_lookup_hash = $2,
				     phone_lookup_hash = $3
				 WHERE id = $1
				   AND (national_id_lookup_hash IS DISTINCT FROM $2
				        OR phone_lookup_hash IS DISTINCT FROM $3)`,
				u.id,
				u.nationalIDLookupHash,
				u.phoneLookupHash,
			); execErr != nil {
				return fmt.Errorf("update profile identity lookup hashes: %w", execErr)
			}
		}
		return nil
	})
}

func (r EmployeeIdentityRepository) finalizeBackfillCollisions(
	ctx context.Context,
	report *IdentityBackfillReport,
	dryCounts *dryCollisionTracker,
) error {
	if dryCounts != nil {
		report.NationalIDCollisionGroups, report.NationalIDCollisionRows = countCollisions(dryCounts.nid)
		report.PhoneCollisionGroups, report.PhoneCollisionRows = countCollisions(dryCounts.phone)
	} else {
		readiness, err := r.IdentityReadiness(ctx)
		if err != nil {
			return err
		}
		report.NationalIDCollisionGroups = readiness.NationalIDCollisionGroups
		report.NationalIDCollisionRows = readiness.NationalIDCollisionRows
		report.PhoneCollisionGroups = readiness.PhoneCollisionGroups
		report.PhoneCollisionRows = readiness.PhoneCollisionRows
	}
	report.UniqueNationalIDIndexReady = report.NationalIDCollisionGroups == 0
	report.UniquePhoneIndexReady = report.PhoneCollisionGroups == 0
	report.UniqueIndexReady = report.UniqueNationalIDIndexReady && report.UniquePhoneIndexReady
	return nil
}

// IdentityReadiness returns PII-free aggregate coverage and collision counts for employee_profiles.
func (r EmployeeIdentityRepository) IdentityReadiness(ctx context.Context) (IdentityReadinessReport, error) {
	db, err := r.db()
	if err != nil {
		return IdentityReadinessReport{}, err
	}
	var out IdentityReadinessReport
	err = db.QueryRow(ctx, `
		WITH nid_collisions AS (
			SELECT count(*) AS rows_in_group
			FROM employee_profiles
			WHERE national_id_lookup_hash IS NOT NULL
			GROUP BY national_id_lookup_hash
			HAVING count(*) > 1
		),
		phone_collisions AS (
			SELECT count(*) AS rows_in_group
			FROM employee_profiles
			WHERE phone_lookup_hash IS NOT NULL
			GROUP BY phone_lookup_hash
			HAVING count(*) > 1
		)
		SELECT
			(SELECT count(*) FROM employee_profiles) AS total_profiles,
			(SELECT count(*) FROM employee_profiles WHERE national_id IS NOT NULL AND btrim(national_id) <> '') AS with_raw_national_id,
			(SELECT count(*) FROM employee_profiles WHERE national_id_lookup_hash IS NOT NULL) AS with_national_id_hash,
			(SELECT count(*) FROM employee_profiles WHERE phone IS NOT NULL AND btrim(phone) <> '') AS with_raw_phone,
			(SELECT count(*) FROM employee_profiles WHERE phone_lookup_hash IS NOT NULL) AS with_phone_hash,
			(SELECT count(*) FROM nid_collisions) AS nid_collision_groups,
			(SELECT COALESCE(sum(rows_in_group), 0) FROM nid_collisions) AS nid_collision_rows,
			(SELECT count(*) FROM phone_collisions) AS phone_collision_groups,
			(SELECT COALESCE(sum(rows_in_group), 0) FROM phone_collisions) AS phone_collision_rows
	`).Scan(
		&out.TotalProfiles,
		&out.WithRawNationalID,
		&out.WithNationalIDHash,
		&out.WithRawPhone,
		&out.WithPhoneHash,
		&out.NationalIDCollisionGroups,
		&out.NationalIDCollisionRows,
		&out.PhoneCollisionGroups,
		&out.PhoneCollisionRows,
	)
	if err != nil {
		return IdentityReadinessReport{}, fmt.Errorf("query employee identity readiness: %w", err)
	}
	out.UniqueNationalIDIndexReady = out.NationalIDCollisionGroups == 0
	out.UniquePhoneIndexReady = out.PhoneCollisionGroups == 0
	out.UniqueIndexReady = out.UniqueNationalIDIndexReady && out.UniquePhoneIndexReady
	return out, nil
}

func (r EmployeeIdentityRepository) fetchIdentityBackfillBatch(
	ctx context.Context,
	db DBTX,
	afterID uuid.UUID,
	limit int,
) ([]identityBackfillRow, error) {
	rows, err := db.Query(
		ctx,
		`SELECT id, phone, national_id, phone_lookup_hash, national_id_lookup_hash
		 FROM employee_profiles
		 WHERE id > $1
		 ORDER BY id ASC
		 LIMIT $2`,
		afterID,
		limit,
	)
	if err != nil {
		return nil, fmt.Errorf("select profile identity backfill batch: %w", err)
	}
	defer rows.Close()

	batch := make([]identityBackfillRow, 0, limit)
	for rows.Next() {
		var row identityBackfillRow
		if err := rows.Scan(
			&row.id,
			&row.phone,
			&row.nationalID,
			&row.phoneLookupHash,
			&row.nationalIDLookupHash,
		); err != nil {
			return nil, fmt.Errorf("scan profile identity backfill row: %w", err)
		}
		batch = append(batch, row)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate profile identity backfill rows: %w", err)
	}
	return batch, nil
}

func computeNationalIDHash(hasher identity.Hasher, raw *string) (*string, error) {
	if raw == nil {
		return nil, nil
	}
	hash, present, err := hasher.HashNationalID(*raw)
	if err != nil || !present {
		return nil, err
	}
	return &hash, nil
}

func computePhoneHash(hasher identity.Hasher, raw *string) (*string, bool, error) {
	if raw == nil {
		return nil, false, nil
	}
	hash, present, err := hasher.HashPhone(*raw)
	if errors.Is(err, identity.ErrInvalidPhone) {
		return nil, true, nil
	}
	if err != nil || !present {
		return nil, false, err
	}
	return &hash, false, nil
}

func sameOptionalString(a, b *string) bool {
	if a == nil && b == nil {
		return true
	}
	if a == nil || b == nil {
		return false
	}
	return *a == *b
}

func countCollisions(counts map[string]int) (groups, rows int) {
	for _, n := range counts {
		if n > 1 {
			groups++
			rows += n
		}
	}
	return groups, rows
}
