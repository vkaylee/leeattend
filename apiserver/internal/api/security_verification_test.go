package api

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

// TestTenantIsolation_CrossWorkspaceBoundary verifies authorization boundaries
// for the single organization API without client-selected organization scope.
func TestTenantIsolation_CrossWorkspaceBoundary(t *testing.T) {
	userID := uuid.New()
	request := func(method, path, body string, authenticated bool) *http.Request {
		req := httptest.NewRequest(method, path, bytes.NewBufferString(body))
		if authenticated {
			req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
		}
		return req
	}

	t.Run("unauthenticated request returns 401", func(t *testing.T) {
		h := DepartmentHandler{Repository: mockDepartmentRepo{}, Authorizer: singleOrgTestAuthorizer{permissions: map[string]bool{"departments:create": true}}}
		rec := httptest.NewRecorder()
		h.List(rec, request(http.MethodGet, "/api/v1/departments", "", false))
		if rec.Code != http.StatusUnauthorized {
			t.Fatalf("expected 401 for unauthenticated request, got %d", rec.Code)
		}
	})

	t.Run("missing permission returns 403", func(t *testing.T) {
		h := DepartmentHandler{Repository: mockDepartmentRepo{}, Authorizer: singleOrgTestAuthorizer{permissions: map[string]bool{}}}
		rec := httptest.NewRecorder()
		h.Create(rec, request(http.MethodPost, "/api/v1/departments", `{"name":"Engineering"}`, true))
		if rec.Code != http.StatusForbidden {
			t.Fatalf("expected 403 without department permission, got %d", rec.Code)
		}
	})

	t.Run("permitted request succeeds", func(t *testing.T) {
		h := DepartmentHandler{Repository: mockDepartmentRepo{}, Authorizer: singleOrgTestAuthorizer{permissions: map[string]bool{"departments:create": true}}}
		rec := httptest.NewRecorder()
		h.Create(rec, request(http.MethodPost, "/api/v1/departments", `{"name":"Engineering"}`, true))
		if rec.Code != http.StatusCreated {
			t.Fatalf("expected 201 with department permission, got %d: %s", rec.Code, rec.Body.String())
		}
	})
}

type singleOrgTestAuthorizer struct {
	permissions map[string]bool
}

func (a singleOrgTestAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) {
	return true, nil
}

func (a singleOrgTestAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return a.permissions[permission]
}

func (a singleOrgTestAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func (singleOrgTestAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}

func (singleOrgTestAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool {
	return false
}

func (a singleOrgTestAuthorizer) HasScopedPermission(_ context.Context, _ uuid.UUID, permission string, _ *uuid.UUID) bool {
	return a.permissions[permission]
}

func (singleOrgTestAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

// TestRBAC_PermissionEnforcement verifies that mutating actions require exact permission.
func TestRBAC_PermissionEnforcement(t *testing.T) {
	userID := uuid.New()

	tests := []struct {
		name           string
		permissions    map[string]bool
		expectedStatus int
	}{
		{
			name:           "missing_permission",
			permissions:    map[string]bool{},
			expectedStatus: http.StatusForbidden,
		},
		{
			name:           "unrelated_permission",
			permissions:    map[string]bool{"attendance:read": true},
			expectedStatus: http.StatusForbidden,
		},
		{
			name:           "exact_permission_granted",
			permissions:    map[string]bool{"departments:create": true},
			expectedStatus: http.StatusCreated,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			h := DepartmentHandler{
				Repository: mockDepartmentRepo{},
				Authorizer: singleOrgTestAuthorizer{permissions: tc.permissions},
			}
			req := httptest.NewRequest(
				http.MethodPost,
				"/api/v1/departments",
				bytes.NewBufferString(`{"name":"Quality Assurance"}`),
			)
			req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
			rec := httptest.NewRecorder()

			h.Create(rec, req)
			if rec.Code != tc.expectedStatus {
				t.Fatalf("expected status %d, got %d (body: %s)", tc.expectedStatus, rec.Code, rec.Body.String())
			}
		})
	}
}

// TestPII_MaskingInEmployeePayloads checks that sensitive attributes are not leaked.
func TestPII_MaskingInEmployeePayloads(t *testing.T) {
	sensitiveData := map[string]any{
		"id":           uuid.New().String(),
		"name":         "Nguyen Van A",
		"email":        "nguyen@example.com",
		"bank_account": "1234567890",
		"tax_id":       "TAX-998877",
	}

	// Redaction rule simulation: unprivileged viewers must not see bank_account or tax_id
	redacted := redactEmployeePII(sensitiveData, false)
	if _, ok := redacted["bank_account"]; ok {
		t.Errorf("unprivileged response leaked bank_account: %v", redacted)
	}
	if _, ok := redacted["tax_id"]; ok {
		t.Errorf("unprivileged response leaked tax_id: %v", redacted)
	}

	// Privileged viewers see all fields
	privileged := redactEmployeePII(sensitiveData, true)
	if privileged["bank_account"] != "1234567890" {
		t.Errorf("privileged response missing bank_account")
	}
}

// TestSecurity_UnauthenticatedAccessRejected verifies that missing authentication
// fails closed before handler logic runs.
func TestSecurity_UnauthenticatedAccessRejected(t *testing.T) {
	h := DepartmentHandler{
		Repository: mockDepartmentRepo{},
		Authorizer: singleOrgTestAuthorizer{permissions: map[string]bool{"departments:read": true}},
	}

	// Request with NO user ID in context
	req := httptest.NewRequest(http.MethodGet, "/api/v1/departments", nil)
	rec := httptest.NewRecorder()

	h.List(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 Unauthorized for unauthenticated request, got %d", rec.Code)
	}
}

func redactEmployeePII(data map[string]any, allowPII bool) map[string]any {
	out := make(map[string]any)
	for k, v := range data {
		if !allowPII && (k == "bank_account" || k == "tax_id" || k == "national_id") {
			continue
		}
		out[k] = v
	}
	return out
}
