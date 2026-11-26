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

func assertStorageResponseMasked(t *testing.T, rec *httptest.ResponseRecorder) {
	t.Helper()
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp StorageConfigResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("failed decoding storage response: %v", err)
	}
	if resp.AccessKeyMasked != "***" {
		t.Errorf("expected access_key_masked = '***', got %q", resp.AccessKeyMasked)
	}
}

func assertStoredCredentialsEncrypted(t *testing.T, cryptoSvc *crypto.Service, stored *StorageConfigRecord, rawAccess, rawSecret string) {
	t.Helper()
	if stored == nil {
		t.Fatal("storage config was not saved in repo")
	}
	if stored.AccessKeyEnc == rawAccess || stored.SecretKeyEnc == rawSecret {
		t.Fatal("credentials stored as plaintext in DB!")
	}
	decryptedAccess, err := cryptoSvc.Decrypt(stored.AccessKeyEnc)
	if err != nil || decryptedAccess != rawAccess {
		t.Fatalf("decryption failed or mismatched: %v, got %q", err, decryptedAccess)
	}
	decryptedSecret, err := cryptoSvc.Decrypt(stored.SecretKeyEnc)
	if err != nil || decryptedSecret != rawSecret {
		t.Fatalf("decryption failed or mismatched: %v, got %q", err, decryptedSecret)
	}
}

func newStorageTestFixture(t *testing.T) (StorageConfigHandler, *mockStorageConfigRepo, *crypto.Service, []byte) {
	t.Helper()
	cryptoSvc, err := crypto.NewService("v1", map[string][]byte{"v1": bytes.Repeat([]byte("k"), 32)})
	if err != nil {
		t.Fatal(err)
	}
	repo := &mockStorageConfigRepo{}
	h := StorageConfigHandler{
		Repository:    repo,
		Authorizer:    rbacAuthorizer{member: true},
		Crypto:        cryptoSvc,
		StorageTester: dummyStorageTester{err: nil},
	}
	reqBody, _ := json.Marshal(StorageConfigRequest{
		EndpointURL:      "https://s3.ap-southeast-1.amazonaws.com",
		Region:           "ap-southeast-1",
		BucketName:       "test-bucket",
		AccessKey:        "AKIAIOSFODNN7EXAMPLE",
		SecretKey:        "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
		UsePathStyle:     true,
		PresignedTTLSecs: 900,
		MaxFileSize:      10485760,
	})
	return h, repo, cryptoSvc, reqBody
}

func TestStorageConfig_CredentialEncryptionMasking(t *testing.T) {
	actorID := uuid.New()
	h, repo, cryptoSvc, reqBody := newStorageTestFixture(t)
	withActor := func(method, target string, body []byte) *http.Request {
		req := httptest.NewRequest(method, target, bytes.NewBuffer(body))
		return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	}

	t.Run("put encrypts and masks", func(t *testing.T) {
		rec := httptest.NewRecorder()
		h.PutStorage(rec, withActor(http.MethodPut, "/api/v1/settings/storage", reqBody))
		assertStorageResponseMasked(t, rec)
		assertStoredCredentialsEncrypted(t, cryptoSvc, repo.storageConfig, "AKIAIOSFODNN7EXAMPLE", "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY")
	})
	t.Run("get returns masked", func(t *testing.T) {
		rec := httptest.NewRecorder()
		h.GetStorage(rec, withActor(http.MethodGet, "/api/v1/settings/storage", nil))
		assertStorageResponseMasked(t, rec)
	})
	t.Run("test connection success and failure", func(t *testing.T) {
		rec := httptest.NewRecorder()
		h.TestStorage(rec, withActor(http.MethodPost, "/api/v1/settings/storage/test", reqBody))
		var testResult bool
		if rec.Code != http.StatusOK || json.Unmarshal(rec.Body.Bytes(), &testResult) != nil || !testResult {
			t.Fatalf("expected 200 true, got %d %s", rec.Code, rec.Body.String())
		}
		h.StorageTester = dummyStorageTester{err: errors.New("s3 connection timeout")}
		rec = httptest.NewRecorder()
		h.TestStorage(rec, withActor(http.MethodPost, "/api/v1/settings/storage/test", reqBody))
		if rec.Code != http.StatusInternalServerError {
			t.Fatalf("expected 500 on test failure, got %d", rec.Code)
		}
	})
	t.Run("delete removes config", func(t *testing.T) {
		rec := httptest.NewRecorder()
		h.DeleteStorage(rec, withActor(http.MethodDelete, "/api/v1/settings/storage", nil))
		if rec.Code != http.StatusOK || repo.storageConfig != nil {
			t.Fatalf("delete failed: status=%d remaining=%v", rec.Code, repo.storageConfig)
		}
	})
}
