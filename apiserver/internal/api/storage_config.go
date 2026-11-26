package api

import (
	"context"
	"net/http"
	"strings"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/crypto"
)

type StorageConfigRequest struct {
	EndpointURL      string  `json:"endpoint_url"`
	Region           string  `json:"region"`
	BucketName       string  `json:"bucket_name"`
	PathPrefix       *string `json:"path_prefix"`
	AccessKey        string  `json:"access_key"`
	SecretKey        string  `json:"secret_key"`
	UsePathStyle     bool    `json:"use_path_style"`
	PresignedTTLSecs int32   `json:"presigned_ttl_secs"`
	MaxFileSize      int64   `json:"max_file_size"`
}

type StorageConfigResponse struct {
	ID              uuid.UUID `json:"id"`
	EndpointURL     string    `json:"endpoint_url"`
	Region          string    `json:"region"`
	BucketName      string    `json:"bucket_name"`
	PathPrefix      *string   `json:"path_prefix"`
	AccessKeyMasked string    `json:"access_key_masked"`
	UsePathStyle    bool      `json:"use_path_style"`
	IsActive        bool      `json:"is_active"`
}

type StorageConfigRecord struct {
	StorageConfigResponse
	AccessKeyEnc         string
	SecretKeyEnc         string
	EncryptionKeyVersion string
	PresignedTTLSecs     int32
	MaxFileSize          int64
}

type StorageConfigRepository interface {
	GetStorageConfig(context.Context) (*StorageConfigRecord, error)
	UpsertStorageConfig(context.Context, StorageConfigRecord) (*StorageConfigRecord, error)
	DeleteStorageConfig(context.Context) error
}

type StorageConnectionTester interface {
	Test(context.Context, StorageConfigRequest) error
}

type StorageConfigHandler struct {
	Repository    StorageConfigRepository
	Authorizer    Authorizer
	Crypto        *crypto.Service
	StorageTester StorageConnectionTester
}

func (h *StorageConfigHandler) GetStorage(w http.ResponseWriter, r *http.Request) {
	if _, ok := memberContext(w, r, h.Authorizer); !ok {
		return
	}
	x, e := h.Repository.GetStorageConfig(r.Context())
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	if x == nil {
		apperror.WriteJSON(w, http.StatusOK, nil)
		return
	}
	response := x.StorageConfigResponse
	response.AccessKeyMasked = "***"
	apperror.WriteJSON(w, http.StatusOK, response)
}

func (h *StorageConfigHandler) PutStorage(w http.ResponseWriter, r *http.Request) {
	if _, ok := memberContext(w, r, h.Authorizer); !ok {
		return
	}
	req, e := DecodeJSON[StorageConfigRequest](r, 1<<20)
	if e != nil || strings.TrimSpace(req.EndpointURL) == "" || strings.TrimSpace(req.BucketName) == "" || req.AccessKey == "" || req.SecretKey == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Crypto == nil {
		apperror.WriteError(w, r, apperror.InternalString("storage encryption unavailable"))
		return
	}
	access, e := h.Crypto.Encrypt(req.AccessKey)
	if e != nil {
		apperror.WriteError(w, r, apperror.InternalString("failed to encrypt credentials"))
		return
	}
	secret, e := h.Crypto.Encrypt(req.SecretKey)
	if e != nil {
		apperror.WriteError(w, r, apperror.InternalString("failed to encrypt credentials"))
		return
	}
	rec := StorageConfigRecord{
		StorageConfigResponse: StorageConfigResponse{
			ID:           uuid.New(),
			EndpointURL:  req.EndpointURL,
			Region:       req.Region,
			BucketName:   req.BucketName,
			PathPrefix:   req.PathPrefix,
			UsePathStyle: req.UsePathStyle,
			IsActive:     true,
		},
		AccessKeyEnc:         access,
		SecretKeyEnc:         secret,
		EncryptionKeyVersion: strings.Split(access, ":")[0],
		PresignedTTLSecs:     req.PresignedTTLSecs,
		MaxFileSize:          req.MaxFileSize,
	}
	x, e := h.Repository.UpsertStorageConfig(r.Context(), rec)
	if e != nil {
		writeRBACError(w, r, e)
		return
	}
	response := x.StorageConfigResponse
	response.AccessKeyMasked = "***"
	apperror.WriteJSON(w, http.StatusOK, response)
}

func (h *StorageConfigHandler) DeleteStorage(w http.ResponseWriter, r *http.Request) {
	if _, ok := memberContext(w, r, h.Authorizer); !ok {
		return
	}
	if e := h.Repository.DeleteStorageConfig(r.Context()); e != nil {
		writeRBACError(w, r, e)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, struct{}{})
}

func (h *StorageConfigHandler) TestStorage(w http.ResponseWriter, r *http.Request) {
	if _, ok := memberContext(w, r, h.Authorizer); !ok {
		return
	}
	req, e := DecodeJSON[StorageConfigRequest](r, 1<<20)
	if e != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.StorageTester != nil {
		if e = h.StorageTester.Test(r.Context(), req); e != nil {
			apperror.WriteError(w, r, apperror.Internal(e))
			return
		}
	}
	apperror.WriteJSON(w, http.StatusOK, true)
}
