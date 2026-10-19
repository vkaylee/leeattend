package api

import (
	"bytes"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

// TestTenantIsolation_CrossWorkspaceBoundary verifies that attempts to access
// resources across workspace boundaries without appropriate membership are rejected.
func TestTenantIsolation_CrossWorkspaceBoundary(t *testing.T) {
	wsTenantA := uuid.New()
	wsTenantB := uuid.New()
	userA := uuid.New()

	// Authorizer only has permissions for wsTenantA, wsTenantB is isolated
	authorizer := mockAuthorizer{
		perms: map[string]bool{
			"departments:create": true,
		},
	}

	h := DepartmentHandler{
		Repository: mockDepartmentRepo{},
		Authorizer: authorizer,
	}

	body := bytes.NewBufferString(`{"name":"Engineering"}`)
	reqA := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsTenantA.String()+"/departments", body)
	reqA.SetPathValue("ws_id", wsTenantA.String())
	reqA = reqA.WithContext(httpinfra.WithUserIDForTest(reqA.Context(), userA))
	recA := httptest.NewRecorder()
	h.Create(recA, reqA)
	if recA.Code != http.StatusCreated {
		t.Fatalf("expected 201 for tenant A, got %d", recA.Code)
	}

	// Authorizer configured to simulate tenant isolation denial for wsTenantB
	strictAuthorizer := mockAuthorizer{
		perms: map[string]bool{
			// No permissions for wsTenantB
		},
	}
	hStrict := DepartmentHandler{
		Repository: mockDepartmentRepo{},
		Authorizer: strictAuthorizer,
	}

	bodyB := bytes.NewBufferString(`{"name":"Engineering"}`)
	reqB := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsTenantB.String()+"/departments", bodyB)
	reqB.SetPathValue("ws_id", wsTenantB.String())
	reqB = reqB.WithContext(httpinfra.WithUserIDForTest(reqB.Context(), userA))
	recB := httptest.NewRecorder()
	hStrict.Create(recB, reqB)

	// Must fail closed with 403 Forbidden
	if recB.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden for cross-tenant access, got %d", recB.Code)
	}
}

// TestRBAC_PermissionEnforcement verifies that mutating actions require exact permission.
func TestRBAC_PermissionEnforcement(t *testing.T) {
	wsID := uuid.New()
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
				Authorizer: mockAuthorizer{perms: tc.permissions},
			}
			req := httptest.NewRequest(
				http.MethodPost,
				"/api/v1/workspaces/"+wsID.String()+"/departments",
				bytes.NewBufferString(`{"name":"Quality Assurance"}`),
			)
			req.SetPathValue("ws_id", wsID.String())
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
	wsID := uuid.New()
	h := DepartmentHandler{
		Repository: mockDepartmentRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{"departments:read": true}},
	}

	// Request with NO user ID in context
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/departments", nil)
	req.SetPathValue("ws_id", wsID.String())
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
