package api

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
)

func TestReferenceAndValidationErrorContract(t *testing.T) {
	tests := []struct {
		name   string
		write  func(http.ResponseWriter, *http.Request, error)
		err    error
		status int
		code   apperror.ErrorCode
		key    string
	}{
		{"segment employment", (&WorkSegmentHandler{}).writeError, ErrEmploymentNotFound, 404, apperror.CodeNotFound, apperror.KeyNotFound},
		{"segment shift", (&WorkSegmentHandler{}).writeError, ErrShiftNotFound, 404, apperror.CodeNotFound, apperror.KeyNotFound},
		{"pattern employment", (&WorkingPatternHandler{}).writeError, ErrEmploymentNotFound, 404, apperror.CodeNotFound, apperror.KeyNotFound},
		{"transfer no employment", (&EmployeeDepartmentHandler{}).writeError, ErrNoActiveEmployment, 400, apperror.CodeValidationError, apperror.KeyValidation},
		{"transfer locked payroll", (&EmployeeDepartmentHandler{}).writeError, ErrConflict, 409, apperror.CodeConflict, apperror.KeyConflict},
		{"termination invalid date", (&ContractEmploymentHandler{}).error, ErrInvalidInput, 400, apperror.CodeValidationError, apperror.KeyValidation},
		{"termination missing employment retains key", (&ContractEmploymentHandler{}).error, ErrEmploymentNotFound, 404, apperror.CodeNotFound, apperror.KeyEmploymentNotFound},
		{"unknown error masked", (&WorkSegmentHandler{}).writeError, errors.New("private database detail"), 500, apperror.CodeInternalError, apperror.KeyInternal},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			w := httptest.NewRecorder()
			r := httptest.NewRequest(http.MethodGet, "/?lang=en", nil)
			tc.write(w, r, fmt.Errorf("wrapped: %w", tc.err))
			var body apperror.WireErrorEnvelope
			if err := json.Unmarshal(w.Body.Bytes(), &body); err != nil {
				t.Fatal(err)
			}
			if w.Code != tc.status || body.Error.Code != tc.code || body.Error.Message != apperror.Translate("en", tc.key) {
				t.Fatalf("got status %d, body %+v; want %d %s %s", w.Code, body, tc.status, tc.code, tc.key)
			}
		})
	}
}

func assertWireErrorEnvelope(t *testing.T, write func(http.ResponseWriter, *http.Request, error), err error, wantStatus int, wantCode apperror.ErrorCode, wantKey string) {
	t.Helper()
	w := httptest.NewRecorder()
	r := httptest.NewRequest(http.MethodPost, "/?lang=en", nil)
	write(w, r, err)
	var body apperror.WireErrorEnvelope
	if decErr := json.Unmarshal(w.Body.Bytes(), &body); decErr != nil {
		t.Fatalf("decode wire error: %v", decErr)
	}
	if w.Code != wantStatus || body.Error.Code != wantCode || body.Error.Message != apperror.Translate("en", wantKey) {
		t.Fatalf("got status %d, body %+v; want %d %s %s", w.Code, body, wantStatus, wantCode, wantKey)
	}
	if wantStatus == http.StatusInternalServerError && strings.Contains(body.Error.Message, "secret") {
		t.Fatalf("500 response leaked internal error: %q", body.Error.Message)
	}
}

func TestLifecycleGuardErrorContractAcrossHandlers(t *testing.T) {
	handlers := []struct {
		name  string
		write func(http.ResponseWriter, *http.Request, error)
	}{
		{"work_segments", (&WorkSegmentHandler{}).writeError},
		{"working_patterns", (&WorkingPatternHandler{}).writeError},
		{"rosters", (&RosterHandler{}).writeError},
		{"shifts", (&ShiftHandler{}).writeError},
		{"shift_swaps", (&ShiftSwapHandler{}).writeError},
		{"assignments", (&AssignmentHandler{}).assignmentError},
		{"attendance", (&AttendanceHandler{}).writeError},
		{"attendance_logs", (&AttendanceLogsHandler{}).fail},
		{"holidays", (&HolidayHandler{}).writeError},
	}

	cases := []struct {
		name   string
		err    error
		status int
		code   apperror.ErrorCode
		key    string
	}{
		{"blocked", ErrLifecycleMutationBlocked, http.StatusConflict, apperror.CodeConflict, apperror.KeyConflict},
		{"schema_unavailable", ErrLifecycleSchemaUnavailable, http.StatusServiceUnavailable, apperror.CodeServiceUnavailable, apperror.KeyServiceUnavailable},
		{"evidence_failed", fmt.Errorf("%w: secret pg failure", ErrLifecycleEvidenceFailed), http.StatusInternalServerError, apperror.CodeInternalError, apperror.KeyInternal},
		{"evidence_failed_overrides_blocked_join", errors.Join(ErrLifecycleMutationBlocked, ErrLifecycleEvidenceFailed), http.StatusInternalServerError, apperror.CodeInternalError, apperror.KeyInternal},
		{"evidence_failed_overrides_app_error", errors.Join(apperror.Conflict(apperror.KeyConflict), ErrLifecycleEvidenceFailed), http.StatusInternalServerError, apperror.CodeInternalError, apperror.KeyInternal},
	}

	for _, h := range handlers {
		for _, tc := range cases {
			t.Run(h.name+"/"+tc.name, func(t *testing.T) {
				assertWireErrorEnvelope(t, h.write, tc.err, tc.status, tc.code, tc.key)
			})
		}
	}
}

func TestShiftSwapLockedPeriodReturnsConflict(t *testing.T) {
	assertWireErrorEnvelope(t, (&ShiftSwapHandler{}).writeError, ErrScheduleLockedPeriod, http.StatusConflict, apperror.CodeConflict, apperror.KeyConflict)
}

func TestRemainingMapperPoliciesAndEdgeCases(t *testing.T) {
	// Newly caught Slice 5 unmapped repository sentinels
	t.Run("attendance idempotency conflict returns 409", func(t *testing.T) {
		assertWireErrorEnvelope(t, (&AttendanceHandler{}).writeError, ErrAttendanceIdempotencyConflict, http.StatusConflict, apperror.CodeConflict, apperror.KeyConflict)
	})
	t.Run("assignment unique conflict returns 409", func(t *testing.T) {
		assertWireErrorEnvelope(t, (&AssignmentHandler{}).assignmentError, fmt.Errorf("%w: unique_constraint", ErrConflict), http.StatusConflict, apperror.CodeConflict, apperror.KeyConflict)
	})
	t.Run("shift swap no active employment returns 400", func(t *testing.T) {
		assertWireErrorEnvelope(t, (&ShiftSwapHandler{}).writeError, ErrNoActiveEmployment, http.StatusBadRequest, apperror.CodeValidationError, apperror.KeyValidation)
	})
	t.Run("shift swap forbidden returns 403", func(t *testing.T) {
		assertWireErrorEnvelope(t, (&ShiftSwapHandler{}).writeError, ErrShiftSwapForbidden, http.StatusForbidden, apperror.CodeForbidden, apperror.KeyForbidden)
	})

	// Sentinel-first + AppError passthrough policies preserved (must NOT invert precedence)
	t.Run("attendance_policies sentinel beats wrapped AppError and passthrough works", func(t *testing.T) {
		mixed := errors.Join(apperror.Conflict(apperror.KeyConflict), ErrNotFound)
		assertWireErrorEnvelope(t, (&AttendancePolicyHandler{}).fail, mixed, http.StatusNotFound, apperror.CodeNotFound, apperror.KeyNotFound)
		assertWireErrorEnvelope(t, (&AttendancePolicyHandler{}).fail, apperror.Forbidden(apperror.KeyNotOrganizationMember), http.StatusForbidden, apperror.CodeForbidden, apperror.KeyNotOrganizationMember)
		assertWireErrorEnvelope(t, (&AttendancePolicyHandler{}).fail, errors.New("secret policy db error"), http.StatusInternalServerError, apperror.CodeInternalError, apperror.KeyInternal)
	})

	t.Run("overtime sentinel beats wrapped AppError and passthrough works", func(t *testing.T) {
		mixed := errors.Join(apperror.NotFound(apperror.KeyNotFound), ErrOvertimeConflict)
		assertWireErrorEnvelope(t, (&OvertimeHandler{}).fail, mixed, http.StatusConflict, apperror.CodeConflict, apperror.KeyConflict)
		assertWireErrorEnvelope(t, (&OvertimeHandler{}).fail, apperror.Validation(apperror.KeyValidation), http.StatusBadRequest, apperror.CodeValidationError, apperror.KeyValidation)
		assertWireErrorEnvelope(t, (&OvertimeHandler{}).fail, errors.New("secret overtime db error"), http.StatusInternalServerError, apperror.CodeInternalError, apperror.KeyInternal)
	})

	// Special validation fallback policy preserved
	t.Run("employee_match preserves 503 for missing secret and 400 validation fallback", func(t *testing.T) {
		assertWireErrorEnvelope(t, writeMatchBuildError, identity.ErrIdentityHashSecretMissing, http.StatusServiceUnavailable, apperror.CodeServiceUnavailable, apperror.KeyServiceUnavailable)
		assertWireErrorEnvelope(t, writeMatchBuildError, errors.New("bad dob"), http.StatusBadRequest, apperror.CodeValidationError, apperror.KeyValidation)
	})

	// ShiftComplianceError policies keep distinct keys
	t.Run("shifts vs working_patterns keep distinct compliance error keys", func(t *testing.T) {
		assertWireErrorEnvelope(t, (&ShiftHandler{}).writeError, &ShiftComplianceError{Hard: true}, http.StatusBadRequest, apperror.CodeValidationError, apperror.KeyShiftOverlapDetected)
		assertWireErrorEnvelope(t, (&ShiftHandler{}).writeError, &ShiftComplianceError{Hard: false}, http.StatusConflict, apperror.CodeConflict, apperror.KeyComplianceOverrideRequired)
		assertWireErrorEnvelope(t, (&WorkingPatternHandler{}).writeError, &ShiftComplianceError{Hard: true}, http.StatusBadRequest, apperror.CodeValidationError, apperror.KeyValidation)
		assertWireErrorEnvelope(t, (&WorkingPatternHandler{}).writeError, &ShiftComplianceError{Hard: false}, http.StatusConflict, apperror.CodeConflict, apperror.KeyConflict)
	})
}
