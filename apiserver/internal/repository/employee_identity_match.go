package repository

import (
	"context"
	"fmt"
	"sort"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
)

const (
	defaultMatchHardLimit = 10
	maxMatchHardLimit     = 20
	defaultMatchSoftLimit = 5
	maxMatchSoftLimit     = 10
	maxSoftCandidateScan  = 200
)

const employeeMatchCandidateColumnsSQL = `
	e.id,
	e.full_name,
	COALESCE(latest_tenure.employee_number, '') AS employee_code,
	CASE
		WHEN e.deleted_at IS NOT NULL THEN 'terminated'
		WHEN EXISTS (
			SELECT 1 FROM employee_tenures t
			WHERE t.employee_id = e.id
			  AND t.deleted_at IS NULL
			  AND (t.end_date IS NULL OR t.end_date >= ` + orgTodaySQL + `)
		) THEN 'active'
		WHEN EXISTS (
			SELECT 1 FROM employments emp
			WHERE emp.employee_id = e.id
			  AND emp.deleted_at IS NULL
			  AND emp.status = 'active'
			  AND (emp.end_date IS NULL OR emp.end_date >= ` + orgTodaySQL + `)
		) THEN 'active'
		WHEN EXISTS (
			SELECT 1 FROM employee_tenures t
			WHERE t.employee_id = e.id AND t.deleted_at IS NULL
		) THEN 'terminated'
		ELSE 'terminated'
	END AS status`

const employeeMatchFromSQL = `
FROM employee_profiles ep
JOIN employees e ON e.id = ep.employee_id
LEFT JOIN LATERAL (
	SELECT t.employee_number
	FROM employee_tenures t
	WHERE t.employee_id = e.id AND t.deleted_at IS NULL
	ORDER BY (t.end_date IS NULL) DESC, t.start_date DESC, t.id DESC
	LIMIT 1
) latest_tenure ON TRUE`

type matchBaseRow struct {
	id           uuid.UUID
	fullName     string
	employeeCode string
	status       string
}

// MatchEmployees executes read-only hard (exact lookup hash) and soft (normalized name + DOB/partial phone)
// candidate lookup and returns minimal redacted candidate records only.
// ponytail: soft matching filters DOB/phone suffix in SQL (capped at 200 rows) and verifies Unicode-normalized full_name in Go; add a normalized_full_name index only if single-DOB or 4-digit-suffix buckets exceed 200 profiles.
func (r EmployeeIdentityRepository) MatchEmployees(
	ctx context.Context,
	query api.EmployeeMatchQuery,
) ([]api.EmployeeMatchCandidate, error) {
	if !query.HasCriteria() {
		return []api.EmployeeMatchCandidate{}, nil
	}
	db, err := requireDB(r.DB, "employee identity repository")
	if err != nil {
		return nil, err
	}

	hardLimit := clampMatchLimit(query.HardLimit, defaultMatchHardLimit, maxMatchHardLimit)
	softLimit := clampMatchLimit(query.SoftLimit, defaultMatchSoftLimit, maxMatchSoftLimit)

	hardCandidates, hardSeen, err := fetchHardMatchCandidates(ctx, db, query, hardLimit)
	if err != nil {
		return nil, err
	}

	softCandidates, err := fetchSoftMatchCandidates(ctx, db, query, hardSeen, softLimit)
	if err != nil {
		return nil, err
	}

	result := make([]api.EmployeeMatchCandidate, 0, len(hardCandidates)+len(softCandidates))
	result = append(result, hardCandidates...)
	result = append(result, softCandidates...)
	return result, nil
}

func clampMatchLimit(requested, fallback, ceiling int) int {
	if requested <= 0 {
		return fallback
	}
	if requested > ceiling {
		return ceiling
	}
	return requested
}

func fetchHardMatchCandidates(
	ctx context.Context,
	db DBTX,
	query api.EmployeeMatchQuery,
	limit int,
) ([]api.EmployeeMatchCandidate, map[uuid.UUID]bool, error) {
	seen := make(map[uuid.UUID]bool)
	sqlText, args := buildHardMatchSQL(query, limit)
	if sqlText == "" {
		return nil, seen, nil
	}

	rows, err := db.Query(ctx, sqlText, args...)
	if err != nil {
		return nil, nil, fmt.Errorf("query hard employee matches: %w", err)
	}
	defer rows.Close()

	candidates := make([]api.EmployeeMatchCandidate, 0)
	for rows.Next() {
		var row matchBaseRow
		var matchedNID, matchedPhone bool
		if err := rows.Scan(&row.id, &row.fullName, &row.employeeCode, &row.status, &matchedNID, &matchedPhone); err != nil {
			return nil, nil, fmt.Errorf("scan hard employee match: %w", err)
		}
		reasons := hardReasons(matchedNID, matchedPhone)
		if len(reasons) == 0 || seen[row.id] {
			continue
		}
		seen[row.id] = true
		candidates = append(candidates, api.EmployeeMatchCandidate{
			EmployeeID:   row.id,
			DisplayName:  row.fullName,
			EmployeeCode: row.employeeCode,
			Status:       row.status,
			Tier:         api.MatchTierHard,
			Reasons:      reasons,
		})
	}
	if err := rows.Err(); err != nil {
		return nil, nil, fmt.Errorf("iterate hard employee matches: %w", err)
	}
	return candidates, seen, nil
}

func buildHardMatchSQL(query api.EmployeeMatchQuery, limit int) (string, []any) {
	hasNID := query.NationalIDHash != ""
	hasPhone := query.PhoneHash != ""
	switch {
	case hasNID && hasPhone:
		return `SELECT` + employeeMatchCandidateColumnsSQL + `,
			(ep.national_id_lookup_hash = $1) AS matched_national_id,
			(ep.phone_lookup_hash = $2) AS matched_phone` +
			employeeMatchFromSQL + `
		WHERE ep.national_id_lookup_hash = $1
		   OR ep.phone_lookup_hash = $2
		ORDER BY e.id ASC
		LIMIT $3`, []any{query.NationalIDHash, query.PhoneHash, limit}
	case hasNID:
		return `SELECT` + employeeMatchCandidateColumnsSQL + `,
			TRUE AS matched_national_id,
			FALSE AS matched_phone` +
			employeeMatchFromSQL + `
		WHERE ep.national_id_lookup_hash = $1
		ORDER BY e.id ASC
		LIMIT $2`, []any{query.NationalIDHash, limit}
	case hasPhone:
		return `SELECT` + employeeMatchCandidateColumnsSQL + `,
			FALSE AS matched_national_id,
			TRUE AS matched_phone` +
			employeeMatchFromSQL + `
		WHERE ep.phone_lookup_hash = $1
		ORDER BY e.id ASC
		LIMIT $2`, []any{query.PhoneHash, limit}
	default:
		return "", nil
	}
}

func hardReasons(matchedNID, matchedPhone bool) []string {
	reasons := make([]string, 0, 2)
	if matchedNID {
		reasons = append(reasons, api.MatchReasonNationalIDExact)
	}
	if matchedPhone {
		reasons = append(reasons, api.MatchReasonPhoneExact)
	}
	return reasons
}

func fetchSoftMatchCandidates(
	ctx context.Context,
	db DBTX,
	query api.EmployeeMatchQuery,
	hardSeen map[uuid.UUID]bool,
	limit int,
) ([]api.EmployeeMatchCandidate, error) {
	if query.NormalizedFullName == "" || (query.NormalizedDOB == "" && query.PartialPhoneDigits == "") {
		return nil, nil
	}

	byID := make(map[uuid.UUID]*api.EmployeeMatchCandidate)
	if query.NormalizedDOB != "" {
		dobSQL := `SELECT` + employeeMatchCandidateColumnsSQL + employeeMatchFromSQL + `
			WHERE ep.date_of_birth = $1::date
			ORDER BY e.id ASC
			LIMIT $2`
		if err := collectSoftRows(ctx, db, dobSQL, []any{query.NormalizedDOB, maxSoftCandidateScan}, query.NormalizedFullName, api.MatchReasonNameDOBReview, hardSeen, byID); err != nil {
			return nil, err
		}
	}
	if query.PartialPhoneDigits != "" {
		phoneSQL := `SELECT` + employeeMatchCandidateColumnsSQL + employeeMatchFromSQL + `
			WHERE ep.phone IS NOT NULL
			  AND RIGHT(regexp_replace(ep.phone, '[^0-9]', '', 'g'), length($1)) = $1
			ORDER BY e.id ASC
			LIMIT $2`
		if err := collectSoftRows(ctx, db, phoneSQL, []any{query.PartialPhoneDigits, maxSoftCandidateScan}, query.NormalizedFullName, api.MatchReasonNamePhoneReview, hardSeen, byID); err != nil {
			return nil, err
		}
	}

	return sortAndBoundSoftCandidates(byID, limit), nil
}

func collectSoftRows(
	ctx context.Context,
	db DBTX,
	sqlText string,
	args []any,
	normalizedTargetName string,
	reason string,
	hardSeen map[uuid.UUID]bool,
	byID map[uuid.UUID]*api.EmployeeMatchCandidate,
) error {
	rows, err := db.Query(ctx, sqlText, args...)
	if err != nil {
		return fmt.Errorf("query soft employee matches (%s): %w", reason, err)
	}
	defer rows.Close()

	for rows.Next() {
		var row matchBaseRow
		if err := rows.Scan(&row.id, &row.fullName, &row.employeeCode, &row.status); err != nil {
			return fmt.Errorf("scan soft employee match (%s): %w", reason, err)
		}
		if hardSeen[row.id] || identity.NormalizeName(row.fullName) != normalizedTargetName {
			continue
		}
		mergeSoftCandidate(byID, row, reason)
	}
	if err := rows.Err(); err != nil {
		return fmt.Errorf("iterate soft employee matches (%s): %w", reason, err)
	}
	return nil
}

func mergeSoftCandidate(byID map[uuid.UUID]*api.EmployeeMatchCandidate, row matchBaseRow, reason string) {
	if existing, ok := byID[row.id]; ok {
		for _, r := range existing.Reasons {
			if r == reason {
				return
			}
		}
		existing.Reasons = append(existing.Reasons, reason)
		return
	}
	byID[row.id] = &api.EmployeeMatchCandidate{
		EmployeeID:   row.id,
		DisplayName:  row.fullName,
		EmployeeCode: row.employeeCode,
		Status:       row.status,
		Tier:         api.MatchTierSoft,
		Reasons:      []string{reason},
	}
}

func sortAndBoundSoftCandidates(byID map[uuid.UUID]*api.EmployeeMatchCandidate, limit int) []api.EmployeeMatchCandidate {
	if len(byID) == 0 {
		return nil
	}
	candidates := make([]api.EmployeeMatchCandidate, 0, len(byID))
	for _, candidate := range byID {
		candidates = append(candidates, *candidate)
	}
	sort.Slice(candidates, func(i, j int) bool {
		return candidates[i].EmployeeID.String() < candidates[j].EmployeeID.String()
	})
	if len(candidates) > limit {
		candidates = candidates[:limit]
	}
	return candidates
}
