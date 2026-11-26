package api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/crypto"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockStorageConfigRepo struct {
	storageConfig *StorageConfigRecord
}

func (m *mockStorageConfigRepo) GetStorageConfig(_ context.Context) (*StorageConfigRecord, error) {
	return m.storageConfig, nil
}

func (m *mockStorageConfigRepo) UpsertStorageConfig(_ context.Context, rec StorageConfigRecord) (*StorageConfigRecord, error) {
	m.storageConfig = &rec
	return &rec, nil
}

func (m *mockStorageConfigRepo) DeleteStorageConfig(_ context.Context) error {
	m.storageConfig = nil
	return nil
}

type dummyStorageTester struct{ err error }

func (d dummyStorageTester) Test(_ context.Context, _ StorageConfigRequest) error { return d.err }

func TestStorageConfig_RequiresMembership(t *testing.T) {
	h := StorageConfigHandler{Authorizer: rbacAuthorizer{member: false}}
	for _, tc := range []struct {
		name   string
		method string
		handle http.HandlerFunc
	}{
		{"get", http.MethodGet, h.GetStorage},
		{"put", http.MethodPut, h.PutStorage},
		{"delete", http.MethodDelete, h.DeleteStorage},
		{"test", http.MethodPost, h.TestStorage},
	} {
		t.Run(tc.name, func(t *testing.T) {
			for _, authenticated := range []bool{false, true} {
				req := httptest.NewRequest(tc.method, "/api/v1/settings/storage", nil)
				want := http.StatusUnauthorized
				if authenticated {
					req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
					want = http.StatusForbidden
				}
				rec := httptest.NewRecorder()
				tc.handle(rec, req)
				if rec.Code != want {
					t.Fatalf("authenticated=%t: status %d, want %d", authenticated, rec.Code, want)
				}
			}
		})
	}
}

func TestStorageConfig_CredentialEncryptionMasking(t *testing.T) {
	actorID := uuid.New()
	cryptoSvc, err := crypto.NewService("v1", map[string][]byte{"v1": bytes.Repeat([]byte("k"), 32)})
	if err != nil {
		t.Fatal(err)
	}
	repo := &mockStorageConfigRepo{}
	tester := dummyStorageTester{err: nil}
	authz := rbacAuthorizer{member: true}
	h := StorageConfigHandler{
		Repository:    repo,
		Authorizer:    authz,
		Crypto:        cryptoSvc,
		StorageTester: tester,
	}

	rawAccess := "AKIAIOSFODNN7EXAMPLE"
	rawSecret := "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"

	// 1. PUT Storage settings
	reqBody, _ := json.Marshal(StorageConfigRequest{
		EndpointURL:      "https://s3.ap-southeast-1.amazonaws.com",
		Region:           "ap-southeast-1",
		BucketName:       "test-bucket",
		AccessKey:        rawAccess,
		SecretKey:        rawSecret,
		UsePathStyle:     true,
		PresignedTTLSecs: 900,
		MaxFileSize:      10485760,
	})
	req := httptest.NewRequest(http.MethodPut, "/api/v1/settings/storage", bytes.NewBuffer(reqBody))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec := httptest.NewRecorder()
	h.PutStorage(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp StorageConfigResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("failed decoding put storage response: %v", err)
	}
	if resp.AccessKeyMasked != "***" {
		t.Errorf("expected access_key_masked = '***', got %q", resp.AccessKeyMasked)
	}

	// Verify DB stored encrypted values, NOT plaintext
	if repo.storageConfig == nil {
		t.Fatal("storage config was not saved in repo")
	}
	if repo.storageConfig.AccessKeyEnc == rawAccess || repo.storageConfig.SecretKeyEnc == rawSecret {
		t.Fatal("credentials stored as plaintext in DB!")
	}
	decryptedAccess, err := cryptoSvc.Decrypt(repo.storageConfig.AccessKeyEnc)
	if err != nil || decryptedAccess != rawAccess {
		t.Fatalf("decryption failed or mismatched: %v, got %q", err, decryptedAccess)
	}
	decryptedSecret, err := cryptoSvc.Decrypt(repo.storageConfig.SecretKeyEnc)
	if err != nil || decryptedSecret != rawSecret {
		t.Fatalf("decryption failed or mismatched: %v, got %q", err, decryptedSecret)
	}

	// 2. GET Storage settings -> returns masked
	req = httptest.NewRequest(http.MethodGet, "/api/v1/settings/storage", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.GetStorage(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var getResp StorageConfigResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &getResp); err != nil {
		t.Fatalf("failed decoding get storage response: %v", err)
	}
	if getResp.AccessKeyMasked != "***" {
		t.Errorf("expected masked access key, got %q", getResp.AccessKeyMasked)
	}

	// 3. POST Storage test -> 200 true
	req = httptest.NewRequest(http.MethodPost, "/api/v1/settings/storage/test", bytes.NewBuffer(reqBody))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.TestStorage(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var testResult bool
	if err := json.Unmarshal(rec.Body.Bytes(), &testResult); err != nil || !testResult {
		t.Fatalf("expected true, got %v (err: %v)", testResult, err)
	}

	// Test storage test failure
	h.StorageTester = dummyStorageTester{err: errors.New("s3 connection timeout")}
	req = httptest.NewRequest(http.MethodPost, "/api/v1/settings/storage/test", bytes.NewBuffer(reqBody))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.TestStorage(rec, req)
	if rec.Code != http.StatusInternalServerError {
		t.Fatalf("expected 500 on test failure, got %d", rec.Code)
	}

	// 4. DELETE Storage settings -> 200
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/settings/storage", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.DeleteStorage(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if repo.storageConfig != nil {
		t.Errorf("storage config was not deleted")
	}
}
