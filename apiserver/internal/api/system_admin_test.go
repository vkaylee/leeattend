package api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockSystemAdminMock struct {
	roles               []SystemRoleInfo
	userRoles           map[uuid.UUID][]string
	clusters            map[uuid.UUID]SystemDBCluster
	settings            map[string]json.RawMessage
	tenants             map[uuid.UUID]SystemTenant
	countPlatformAdmins int64
}

func newMockSystemAdmin() *mockSystemAdminMock {
	return &mockSystemAdminMock{
		userRoles:           make(map[uuid.UUID][]string),
		clusters:            make(map[uuid.UUID]SystemDBCluster),
		settings:            make(map[string]json.RawMessage),
		tenants:             make(map[uuid.UUID]SystemTenant),
		countPlatformAdmins: 2,
	}
}

func (m *mockSystemAdminMock) ListSystemRoles(context.Context) ([]SystemRoleInfo, error) {
	return m.roles, nil
}

func (m *mockSystemAdminMock) GetUserSystemRoles(_ context.Context, userID uuid.UUID) ([]string, error) {
	return m.userRoles[userID], nil
}

func (m *mockSystemAdminMock) AssignSystemRole(_ context.Context, userID, _ uuid.UUID, role SystemRoleType) error {
	m.userRoles[userID] = append(m.userRoles[userID], string(role))
	return nil
}

func (m *mockSystemAdminMock) RevokeSystemRole(_ context.Context, userID uuid.UUID, role string) error {
	if role == "platform_admin" && m.countPlatformAdmins <= 1 {
		return ErrLastPlatformAdmin
	}
	existing := m.userRoles[userID]
	found := false
	newRoles := make([]string, 0, len(existing))
	for _, r := range existing {
		if r == role && !found {
			found = true
			continue
		}
		newRoles = append(newRoles, r)
	}
	if !found {
		return ErrSystemRoleNotFound
	}
	m.userRoles[userID] = newRoles
	if role == "platform_admin" {
		m.countPlatformAdmins--
	}
	return nil
}

func (m *mockSystemAdminMock) ListDBClusters(context.Context) ([]SystemDBCluster, error) {
	clusters := make([]SystemDBCluster, 0, len(m.clusters))
	for _, c := range m.clusters {
		clusters = append(clusters, c)
	}
	return clusters, nil
}

func (m *mockSystemAdminMock) CreateDBCluster(_ context.Context, req CreateDBClusterRequest) (SystemDBCluster, error) {
	id := uuid.New()
	c := SystemDBCluster{
		ID:               id,
		Name:             req.Name,
		AdminDatabaseURL: MaskDatabaseURL(req.AdminDatabaseURL),
		Region:           req.Region,
		MaxTenants:       req.MaxTenants,
		ActiveTenants:    0,
		IsActive:         req.IsActive,
		CreatedAt:        time.Now().UTC(),
		UpdatedAt:        time.Now().UTC(),
	}
	m.clusters[id] = c
	return c, nil
}

func (m *mockSystemAdminMock) DeleteDBCluster(_ context.Context, id uuid.UUID) error {
	delete(m.clusters, id)
	return nil
}

func (m *mockSystemAdminMock) UpdateDBClusterStatus(_ context.Context, id uuid.UUID, active bool) (SystemDBCluster, error) {
	c, ok := m.clusters[id]
	if !ok {
		return SystemDBCluster{}, ErrClusterNotFound
	}
	c.IsActive = active
	c.UpdatedAt = time.Now().UTC()
	m.clusters[id] = c
	return c, nil
}

func (m *mockSystemAdminMock) Get(_ context.Context, key string) (json.RawMessage, bool, error) {
	v, ok := m.settings[key]
	return v, ok, nil
}

func (m *mockSystemAdminMock) Set(_ context.Context, key string, val json.RawMessage) error {
	m.settings[key] = val
	return nil
}

func (m *mockSystemAdminMock) ListTenants(context.Context) ([]SystemTenant, error) {
	tenants := make([]SystemTenant, 0, len(m.tenants))
	for _, t := range m.tenants {
		tenants = append(tenants, t)
	}
	return tenants, nil
}

func (m *mockSystemAdminMock) CreateTenant(_ context.Context, id uuid.UUID, name string, _ uuid.UUID, encryptedURL *string) (SystemTenant, error) {
	t := SystemTenant{
		ID:                   id,
		Name:                 name,
		IsBYOD:               encryptedURL != nil,
		Timezone:             "Asia/Ho_Chi_Minh",
		CreatedAt:            time.Now().UTC(),
		EncryptedDatabaseURL: encryptedURL,
	}
	m.tenants[id] = t
	return t, nil
}

func (m *mockSystemAdminMock) GetTenant(_ context.Context, id uuid.UUID) (SystemTenant, error) {
	t, ok := m.tenants[id]
	if !ok {
		return SystemTenant{}, ErrTenantNotFound
	}
	return t, nil
}

func (m *mockSystemAdminMock) DeleteTenant(_ context.Context, id uuid.UUID) error {
	delete(m.tenants, id)
	return nil
}

func (m *mockSystemAdminMock) UpdateTenantDatabase(_ context.Context, id uuid.UUID, encryptedURL *string) error {
	t, ok := m.tenants[id]
	if !ok {
		return ErrTenantNotFound
	}
	t.EncryptedDatabaseURL = encryptedURL
	t.IsBYOD = encryptedURL != nil
	m.tenants[id] = t
	return nil
}

type mockCryptoService struct {
	decryptFails bool
}

func (c mockCryptoService) Encrypt(s string) (string, error) {
	return "v1:enc:" + s, nil
}

func (c mockCryptoService) Decrypt(s string) (string, error) {
	if c.decryptFails {
		return "", errors.New("decryption failed")
	}
	if strings.HasPrefix(s, "v1:enc:") {
		return strings.TrimPrefix(s, "v1:enc:"), nil
	}
	return s, nil
}

type mockEmailService struct {
	fail bool
	msg  string
}

func (e mockEmailService) SendTestEmail(_ context.Context, to string) (string, error) {
	if e.fail {
		return "", errors.New("connection to smtp server failed")
	}
	if e.msg != "" {
		return e.msg, nil
	}
	return "Test email sent successfully. Check " + to + " for the message.", nil
}

func adminRequest(method, path string, body string, adminID uuid.UUID) *http.Request {
	var r *http.Request
	if body != "" {
		r = httptest.NewRequest(method, path, bytes.NewBufferString(body))
	} else {
		r = httptest.NewRequest(method, path, nil)
	}
	if adminID != uuid.Nil {
		r = r.WithContext(httpinfra.WithUserIDForTest(r.Context(), adminID))
	}
	return r
}

func setupTestHandler() (SystemAdminHandler, *mockSystemAdminMock, uuid.UUID) {
	adminID := uuid.New()
	mock := newMockSystemAdmin()
	mock.userRoles[adminID] = []string{"platform_admin"}
	h := SystemAdminHandler{
		RolesRepo:     mock,
		Clusters:      mock,
		Settings:      mock,
		Tenants:       mock,
		Roles:         systemHealthRepoMock{roles: []string{"platform_admin"}},
		Crypto:        mockCryptoService{},
		EmailSender:   mockEmailService{},
		PgBouncerHost: "pgbouncer.local",
		PgBouncerPort: 6432,
	}
	return h, mock, adminID
}

func TestSystemAdminPlatformAdminAuth(t *testing.T) {
	h, _, _ := setupTestHandler()

	// 1. Unauthenticated -> 401
	rec := httptest.NewRecorder()
	h.GetSMTPConfig(rec, adminRequest(http.MethodGet, "/api/v1/system/smtp/config", "", uuid.Nil))
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 for unauthenticated, got %d", rec.Code)
	}

	rec = httptest.NewRecorder()
	h.ListRoles(rec, adminRequest(http.MethodGet, "/api/v1/system/roles", "", uuid.Nil))
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 for unauthenticated, got %d", rec.Code)
	}

	// 2. Non-PlatformAdmin (Support) -> 403
	hSupport := h
	hSupport.Roles = systemHealthRepoMock{roles: []string{"support"}}
	rec = httptest.NewRecorder()
	hSupport.GetSMTPConfig(rec, adminRequest(http.MethodGet, "/api/v1/system/smtp/config", "", uuid.New()))
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for support role, got %d", rec.Code)
	}

	// 3. PlatformAdmin -> 200
	rec = httptest.NewRecorder()
	h.GetSMTPConfig(rec, adminRequest(http.MethodGet, "/api/v1/system/smtp/config", "", uuid.New()))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for platform_admin, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestSystemAdminSMTPConfigAndTest(t *testing.T) {
	h, mock, adminID := setupTestHandler()

	// 1. Initial empty config returns {}
	rec := httptest.NewRecorder()
	h.GetSMTPConfig(rec, adminRequest(http.MethodGet, "/api/v1/system/smtp/config", "", adminID))
	if rec.Code != http.StatusOK || rec.Body.String() != "{}" {
		t.Fatalf("expected 200 with {}, got %d: %s", rec.Code, rec.Body.String())
	}

	// 2. PUT SMTP config saves JSON
	configJSON := `{"host":"smtp.example.com","port":587,"username":"user"}`
	rec = httptest.NewRecorder()
	h.SetSMTPConfig(rec, adminRequest(http.MethodPut, "/api/v1/system/smtp/config", configJSON, adminID))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 on put config, got %d: %s", rec.Code, rec.Body.String())
	}
	if string(mock.settings["smtp"]) != configJSON {
		t.Fatalf("expected setting to be saved in mock, got %s", string(mock.settings["smtp"]))
	}

	// 3. POST SMTP test validation: empty or no @ -> 400
	rec = httptest.NewRecorder()
	h.TestSMTP(rec, adminRequest(http.MethodPost, "/api/v1/system/smtp/test", `{"to_email":""}`, adminID))
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty to_email, got %d", rec.Code)
	}

	rec = httptest.NewRecorder()
	h.TestSMTP(rec, adminRequest(http.MethodPost, "/api/v1/system/smtp/test", `{"to_email":"notanemail"}`, adminID))
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for email without @, got %d", rec.Code)
	}

	// 4. POST SMTP test failure: returns 200 with success: false
	hFail := h
	hFail.EmailSender = mockEmailService{fail: true}
	rec = httptest.NewRecorder()
	hFail.TestSMTP(rec, adminRequest(http.MethodPost, "/api/v1/system/smtp/test", `{"to_email":"admin@example.com"}`, adminID))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 on test failure, got %d", rec.Code)
	}
	var failResp SMTPTestResponse
	if err := json.NewDecoder(rec.Body).Decode(&failResp); err != nil || failResp.Success {
		t.Fatalf("expected success:false on failure, got %+v", failResp)
	}

	// 5. POST SMTP test success: returns 200 with success: true and updates last_verified_at
	rec = httptest.NewRecorder()
	h.TestSMTP(rec, adminRequest(http.MethodPost, "/api/v1/system/smtp/test", `{"to_email":"admin@example.com"}`, adminID))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 on test success, got %d", rec.Code)
	}
	var okResp SMTPTestResponse
	if err := json.NewDecoder(rec.Body).Decode(&okResp); err != nil || !okResp.Success {
		t.Fatalf("expected success:true, got %+v", okResp)
	}
	if !strings.Contains(string(mock.settings["smtp"]), "last_verified_at") {
		t.Fatalf("expected last_verified_at in smtp config, got %s", string(mock.settings["smtp"]))
	}
}

func TestSystemAdminRoles(t *testing.T) {
	h, mock, adminID := setupTestHandler()
	targetUser := uuid.New()
	mock.userRoles[targetUser] = []string{"support"}

	// 1. GET /api/v1/system/roles
	rec := httptest.NewRecorder()
	h.ListRoles(rec, adminRequest(http.MethodGet, "/api/v1/system/roles", "", adminID))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 on list roles, got %d", rec.Code)
	}

	// 2. GET /api/v1/system/users/{user_id}/roles
	req := adminRequest(http.MethodGet, "/api/v1/system/users/"+targetUser.String()+"/roles", "", adminID)
	req.SetPathValue("user_id", targetUser.String())
	rec = httptest.NewRecorder()
	h.GetUserRoles(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 on get user roles, got %d", rec.Code)
	}

	// 3. POST /api/v1/system/users/{user_id}/roles - assign unknown role -> 400
	req = adminRequest(http.MethodPost, "/api/v1/system/users/"+targetUser.String()+"/roles", `{"role_name":"superuser"}`, adminID)
	req.SetPathValue("user_id", targetUser.String())
	rec = httptest.NewRecorder()
	h.AssignRole(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 on unknown role, got %d", rec.Code)
	}

	// 4. POST /api/v1/system/users/{user_id}/roles - assign valid role -> 204
	req = adminRequest(http.MethodPost, "/api/v1/system/users/"+targetUser.String()+"/roles", `{"role_name":"billing_admin"}`, adminID)
	req.SetPathValue("user_id", targetUser.String())
	rec = httptest.NewRecorder()
	h.AssignRole(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 on assign role, got %d", rec.Code)
	}

	// 5. DELETE role - unknown role -> 400
	req = adminRequest(http.MethodDelete, "/api/v1/system/users/"+targetUser.String()+"/roles/unknown_role", "", adminID)
	req.SetPathValue("user_id", targetUser.String())
	req.SetPathValue("role_name", "unknown_role")
	rec = httptest.NewRecorder()
	h.RevokeRole(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for unknown role revoke, got %d", rec.Code)
	}

	// 6. DELETE role - cannot revoke last platform_admin guard -> 400
	mock.countPlatformAdmins = 1
	mock.userRoles[targetUser] = []string{"platform_admin"}
	req = adminRequest(http.MethodDelete, "/api/v1/system/users/"+targetUser.String()+"/roles/platform_admin", "", adminID)
	req.SetPathValue("user_id", targetUser.String())
	req.SetPathValue("role_name", "platform_admin")
	rec = httptest.NewRecorder()
	h.RevokeRole(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 when revoking last platform admin, got %d", rec.Code)
	}

	// 7. DELETE role - succeeds with count > 1 -> 204
	mock.countPlatformAdmins = 2
	rec = httptest.NewRecorder()
	h.RevokeRole(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 when multiple platform admins, got %d", rec.Code)
	}
}

func TestSystemAdminDBClusters(t *testing.T) {
	h, _, adminID := setupTestHandler()

	// 1. List DB clusters -> 200
	rec := httptest.NewRecorder()
	h.ListDbClusters(rec, adminRequest(http.MethodGet, "/api/v1/system/db-clusters", "", adminID))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 on list clusters, got %d", rec.Code)
	}

	// 2. Create DB cluster -> 200
	createBody := `{"name":"cluster-1","admin_database_url":"postgres://admin:secret@db1:5432/postgres","region":"ap-southeast-1","max_tenants":500,"is_active":true}`
	rec = httptest.NewRecorder()
	h.CreateDbCluster(rec, adminRequest(http.MethodPost, "/api/v1/system/db-clusters", createBody, adminID))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 on create cluster, got %d: %s", rec.Code, rec.Body.String())
	}
	var cluster SystemDBCluster
	if err := json.NewDecoder(rec.Body).Decode(&cluster); err != nil {
		t.Fatalf("failed to decode cluster: %v", err)
	}
	if cluster.AdminDatabaseURL != "postgres://admin:***@db1:5432/postgres" {
		t.Fatalf("expected password masked with ***, got %s", cluster.AdminDatabaseURL)
	}

	// 3. Patch cluster status -> 200
	patchBody := `{"is_active":false}`
	req := adminRequest(http.MethodPatch, "/api/v1/system/db-clusters/"+cluster.ID.String()+"/status", patchBody, adminID)
	req.SetPathValue("id", cluster.ID.String())
	rec = httptest.NewRecorder()
	h.UpdateDbClusterStatus(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 on patch status, got %d", rec.Code)
	}

	// 4. Delete cluster -> 204 even missing ID
	missingID := uuid.New()
	req = adminRequest(http.MethodDelete, "/api/v1/system/db-clusters/"+missingID.String(), "", adminID)
	req.SetPathValue("id", missingID.String())
	rec = httptest.NewRecorder()
	h.DeleteDbCluster(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 on delete cluster, got %d", rec.Code)
	}
}

func TestSystemAdminTenants(t *testing.T) {
	h, _, adminID := setupTestHandler()

	// 1. Create tenant -> 201
	createBody := `{"name":"Acme Tenant","database_url":"postgres://tenant:pass123@db2:5432/tenant_db"}`
	rec := httptest.NewRecorder()
	h.CreateTenant(rec, adminRequest(http.MethodPost, "/api/v1/system/tenants", createBody, adminID))
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 on create tenant, got %d: %s", rec.Code, rec.Body.String())
	}
	var tenant SystemTenantResponse
	if err := json.NewDecoder(rec.Body).Decode(&tenant); err != nil {
		t.Fatalf("failed to decode tenant response: %v", err)
	}
	if !tenant.IsBYOD {
		t.Fatalf("expected is_byod true")
	}

	// 2. List tenants -> 200
	rec = httptest.NewRecorder()
	h.ListTenants(rec, adminRequest(http.MethodGet, "/api/v1/system/tenants", "", adminID))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 on list tenants, got %d", rec.Code)
	}

	// 3. Get tenant stats -> 200 with masked password
	req := adminRequest(http.MethodGet, "/api/v1/system/tenants/"+tenant.ID.String()+"/stats", "", adminID)
	req.SetPathValue("tenant_id", tenant.ID.String())
	rec = httptest.NewRecorder()
	h.GetTenantStats(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 on tenant stats, got %d: %s", rec.Code, rec.Body.String())
	}
	var stats TenantStatsResponse
	if err := json.NewDecoder(rec.Body).Decode(&stats); err != nil {
		t.Fatalf("failed to decode stats: %v", err)
	}
	if stats.MaskedDBURL == nil || *stats.MaskedDBURL != "postgres://tenant:***@db2:5432/tenant_db" {
		t.Fatalf("expected masked db url with ***, got %v", stats.MaskedDBURL)
	}
	if stats.Connection.Host != "pgbouncer.local" || stats.Connection.Port != "6432" {
		t.Fatalf("unexpected connection info: %+v", stats.Connection)
	}

	// 4. Update tenant database URL -> 200 with empty body
	updateBody := `{"database_url":"postgres://newtenant:newpass@db3:5432/tenant_db2"}`
	req = adminRequest(http.MethodPut, "/api/v1/system/tenants/"+tenant.ID.String()+"/database", updateBody, adminID)
	req.SetPathValue("tenant_id", tenant.ID.String())
	rec = httptest.NewRecorder()
	h.UpdateTenantDatabase(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 on update tenant db, got %d", rec.Code)
	}
	if rec.Body.Len() != 0 {
		t.Fatalf("expected empty body on update tenant db, got %s", rec.Body.String())
	}

	// 5. Delete tenant -> 204
	req = adminRequest(http.MethodDelete, "/api/v1/system/tenants/"+tenant.ID.String(), "", adminID)
	req.SetPathValue("tenant_id", tenant.ID.String())
	rec = httptest.NewRecorder()
	h.DeleteTenant(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 on delete tenant, got %d", rec.Code)
	}
}

func TestMaskDatabaseURL(t *testing.T) {
	tests := []struct {
		input string
		want  string
	}{
		{"postgres://user:password@localhost:5432/dbname", "postgres://user:***@localhost:5432/dbname"},
		{"postgres://app:super_secret_123!@db.host.internal:5432/prod?sslmode=require", "postgres://app:***@db.host.internal:5432/prod?sslmode=require"},
		{"postgres://localhost:5432/dbname", "postgres://localhost:5432/dbname"},
		{"invalid-url", "*** REDACTED ***"},
		{"", "*** REDACTED ***"},
	}
	for _, tc := range tests {
		got := MaskDatabaseURL(tc.input)
		if got != tc.want {
			t.Errorf("MaskDatabaseURL(%q) = %q, want %q", tc.input, got, tc.want)
		}
	}
}
