package apperror

import (
	"encoding/json"
	"errors"
	"net/http"
	"strings"
	"testing"
)

func TestStatusCodesAndErrorCodes(t *testing.T) {
	cases := []struct {
		err          *AppError
		wantStatus   int
		wantCode     ErrorCode
		wantInternal bool
	}{
		{Database(errors.New("db down")), http.StatusInternalServerError, CodeDatabaseError, true},
		{Redis(errors.New("redis timeout")), http.StatusInternalServerError, CodeRedisError, true},
		{Internal(errors.New("fatal")), http.StatusInternalServerError, CodeInternalError, true},
		{BackgroundJob(errors.New("queue fail")), http.StatusInternalServerError, CodeBackgroundJobError, true},
		{Template(errors.New("render fail")), http.StatusInternalServerError, CodeTemplateError, true},
		{Unauthorized(""), http.StatusUnauthorized, CodeUnauthorized, false},
		{Forbidden(""), http.StatusForbidden, CodeForbidden, false},
		{NotFound(""), http.StatusNotFound, CodeNotFound, false},
		{Validation(""), http.StatusBadRequest, CodeValidationError, false},
		{PayloadTooLarge(""), http.StatusRequestEntityTooLarge, CodePayloadTooLarge, false},
		{Conflict(""), http.StatusConflict, CodeConflict, false},
		{ServiceUnavailable(""), http.StatusServiceUnavailable, CodeServiceUnavailable, false},
	}

	for _, tc := range cases {
		if tc.err.Status != tc.wantStatus {
			t.Errorf("code %s got status %d, want %d", tc.err.Code, tc.err.Status, tc.wantStatus)
		}
		if tc.err.Code != tc.wantCode {
			t.Errorf("got code %s, want %s", tc.err.Code, tc.wantCode)
		}
		if tc.err.IsInternal() != tc.wantInternal {
			t.Errorf("code %s IsInternal got %v, want %v", tc.err.Code, tc.err.IsInternal(), tc.wantInternal)
		}
	}
}

func TestInternalErrorMaskingAndRedaction(t *testing.T) {
	sensitiveSQL := "SELECT password_hash FROM users WHERE ssn = '0123456789'"
	dbErr := Database(errors.New(sensitiveSQL))

	// 1. Wire message must be translated from error.internal and scrubbed of SQL
	wireMsgVi := dbErr.Translate("vi")
	if strings.Contains(wireMsgVi, sensitiveSQL) || strings.Contains(wireMsgVi, "password") {
		t.Fatalf("sensitive SQL leaked in wire message vi: %s", wireMsgVi)
	}
	if wireMsgVi != "Đã xảy ra lỗi hệ thống." {
		t.Errorf("expected masked message, got %s", wireMsgVi)
	}

	wireMsgEn := dbErr.Translate("en")
	if wireMsgEn != "An internal server error occurred." {
		t.Errorf("expected masked English message, got %s", wireMsgEn)
	}

	// 2. Wire JSON envelope must not contain the internal error
	status, env := ToWireEnvelope(dbErr, "en")
	if status != http.StatusInternalServerError {
		t.Errorf("got status %d, want 500", status)
	}
	rawBytes, _ := json.Marshal(env)
	if strings.Contains(string(rawBytes), sensitiveSQL) {
		t.Fatalf("sensitive SQL leaked in JSON payload: %s", string(rawBytes))
	}

	// 3. Error() and Unwrap() retain the cause for server-side logging
	if !strings.Contains(dbErr.Error(), sensitiveSQL) {
		t.Errorf("Error() should retain internal cause for server log, got: %s", dbErr.Error())
	}
	if !errors.Is(dbErr, dbErr.InternalErr) {
		t.Errorf("Unwrap() should return internal error")
	}
}

func TestGoldenJSONSpecifications(t *testing.T) {
	// 3.1 Standard Validation Error (400)
	t.Run("3.1 Standard Validation Error", func(t *testing.T) {
		err := Validation(KeyFullNameRequired)
		_, envVi := ToWireEnvelope(err, "vi")
		bytesVi, _ := json.Marshal(envVi)
		wantVi := `{"error":{"code":"VALIDATION_ERROR","message":"Họ và tên là bắt buộc"}}`
		if string(bytesVi) != wantVi {
			t.Errorf("got %s, want %s", string(bytesVi), wantVi)
		}

		_, envEn := ToWireEnvelope(err, "en")
		bytesEn, _ := json.Marshal(envEn)
		wantEn := `{"error":{"code":"VALIDATION_ERROR","message":"Full name is required"}}`
		if string(bytesEn) != wantEn {
			t.Errorf("got %s, want %s", string(bytesEn), wantEn)
		}
	})

	// 3.2 Validation Error with Composed Details (400)
	t.Run("3.2 Validation Error with Composed Details", func(t *testing.T) {
		err := ValidationArgs(KeyValidation, map[string]string{"details": "Invalid action: submit"})
		_, env := ToWireEnvelope(err, "vi")
		bytes, _ := json.Marshal(env)
		want := `{"error":{"code":"VALIDATION_ERROR","message":"Lỗi kiểm tra dữ liệu: Invalid action: submit"}}`
		if string(bytes) != want {
			t.Errorf("got %s, want %s", string(bytes), want)
		}
	})

	// 3.3 Unauthorized Error (401)
	t.Run("3.3 Unauthorized Error", func(t *testing.T) {
		err := Unauthorized(KeyInvalidEmailOrPassword)
		_, env := ToWireEnvelope(err, "vi")
		bytes, _ := json.Marshal(env)
		want := `{"error":{"code":"UNAUTHORIZED","message":"Email hoặc mật khẩu không đúng"}}`
		if string(bytes) != want {
			t.Errorf("got %s, want %s", string(bytes), want)
		}
	})

	// 3.4 Forbidden / Permission Denied (403)
	t.Run("3.4 Forbidden / Permission Denied", func(t *testing.T) {
		err := Forbidden(KeyForbidden)
		_, env := ToWireEnvelope(err, "vi")
		bytes, _ := json.Marshal(env)
		want := `{"error":{"code":"FORBIDDEN","message":"Không có quyền truy cập"}}`
		if string(bytes) != want {
			t.Errorf("got %s, want %s", string(bytes), want)
		}
	})

	// 3.5 Resource Not Found (404)
	t.Run("3.5 Resource Not Found", func(t *testing.T) {
		err := NotFound(KeyEmployeeNotFound)
		_, env := ToWireEnvelope(err, "vi")
		bytes, _ := json.Marshal(env)
		want := `{"error":{"code":"NOT_FOUND","message":"Không tìm thấy nhân viên"}}`
		if string(bytes) != want {
			t.Errorf("got %s, want %s", string(bytes), want)
		}
	})

	// 3.6 Conflict with Retired Entity Retry Hint (409)
	t.Run("3.6 Conflict with Retry Hint", func(t *testing.T) {
		retry := map[string]string{
			"leave_type_id": "a1b2c3d4-e5f6-7a8b-9c0d-1e2f3a4b5c6d",
			"name":          "Nghỉ phép năm cũ",
			"reason":        "retired_leave_type",
		}
		err := ConflictWithRetry(KeyConflict, map[string]string{"details": "leave_type_code_retired"}, retry)
		_, env := ToWireEnvelope(err, "vi")
		bytes, _ := json.Marshal(env)

		var parsed map[string]any
		_ = json.Unmarshal(bytes, &parsed)
		errObj := parsed["error"].(map[string]any)

		if errObj["code"] != "CONFLICT" {
			t.Errorf("got code %v, want CONFLICT", errObj["code"])
		}
		if errObj["message"] != "Đã xảy ra xung đột dữ liệu.: leave_type_code_retired" {
			t.Errorf("got message %v", errObj["message"])
		}
		retryObj := errObj["retry"].(map[string]any)
		if retryObj["reason"] != "retired_leave_type" {
			t.Errorf("got retry reason %v", retryObj["reason"])
		}
	})

	// 3.8 Payload Too Large (413)
	t.Run("3.8 Payload Too Large", func(t *testing.T) {
		err := PayloadTooLarge(KeyUploadFileTooLarge)
		_, env := ToWireEnvelope(err, "vi")
		bytes, _ := json.Marshal(env)
		want := `{"error":{"code":"PAYLOAD_TOO_LARGE","message":"File tải lên vượt quá dung lượng cho phép"}}`
		if string(bytes) != want {
			t.Errorf("got %s, want %s", string(bytes), want)
		}
	})

	// 3.9 Internal Server Error — Masked (500)
	t.Run("3.9 Internal Server Error Masked", func(t *testing.T) {
		err := InternalString("sensitive query failed: column x not found")
		_, envVi := ToWireEnvelope(err, "vi")
		bytesVi, _ := json.Marshal(envVi)
		wantVi := `{"error":{"code":"INTERNAL_ERROR","message":"Đã xảy ra lỗi hệ thống."}}`
		if string(bytesVi) != wantVi {
			t.Errorf("got %s, want %s", string(bytesVi), wantVi)
		}

		_, envEn := ToWireEnvelope(err, "en")
		bytesEn, _ := json.Marshal(envEn)
		wantEn := `{"error":{"code":"INTERNAL_ERROR","message":"An internal server error occurred."}}`
		if string(bytesEn) != wantEn {
			t.Errorf("got %s, want %s", string(bytesEn), wantEn)
		}
	})

	// 3.10 Service Unavailable (503)
	t.Run("3.10 Service Unavailable", func(t *testing.T) {
		err := ServiceUnavailableArgs(KeyServiceUnavailable, map[string]string{
			"details": "This feature requires Redis for background processing",
		})
		_, env := ToWireEnvelope(err, "vi")
		bytes, _ := json.Marshal(env)
		want := `{"error":{"code":"SERVICE_UNAVAILABLE","message":"Dịch vụ hiện không khả dụng.: This feature requires Redis for background processing"}}`
		if string(bytes) != want {
			t.Errorf("got %s, want %s", string(bytes), want)
		}
	})
}
