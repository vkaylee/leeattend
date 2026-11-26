package repository

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type StorageConfigRepository struct{ SystemDB DBTX }

func (r StorageConfigRepository) GetStorageConfig(ctx context.Context) (*api.StorageConfigRecord, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	var rec api.StorageConfigRecord
	err := r.SystemDB.QueryRow(ctx, `SELECT id, endpoint_url, region, bucket_name, path_prefix, access_key_enc, secret_key_enc, encryption_key_version, use_path_style, presigned_ttl_secs, max_file_size, is_active FROM storage_configs LIMIT 1`).Scan(&rec.ID, &rec.EndpointURL, &rec.Region, &rec.BucketName, &rec.PathPrefix, &rec.AccessKeyEnc, &rec.SecretKeyEnc, &rec.EncryptionKeyVersion, &rec.UsePathStyle, &rec.PresignedTTLSecs, &rec.MaxFileSize, &rec.IsActive)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &rec, nil
}

func (r StorageConfigRepository) UpsertStorageConfig(ctx context.Context, rec api.StorageConfigRecord) (*api.StorageConfigRecord, error) {
	if r.SystemDB == nil {
		return nil, errors.New("nil system db")
	}
	id := rec.ID
	if id == uuid.Nil {
		id = uuid.New()
	}
	var res api.StorageConfigRecord
	err := r.SystemDB.QueryRow(ctx, `UPDATE storage_configs SET
endpoint_url = $1,
region = $2,
bucket_name = $3,
path_prefix = $4,
access_key_enc = $5,
secret_key_enc = $6,
encryption_key_version = $7,
use_path_style = $8,
presigned_ttl_secs = $9,
max_file_size = $10,
is_active = $11,
updated_at = NOW()
RETURNING id, endpoint_url, region, bucket_name, path_prefix, access_key_enc, secret_key_enc, encryption_key_version, use_path_style, presigned_ttl_secs, max_file_size, is_active`,
		rec.EndpointURL, rec.Region, rec.BucketName, rec.PathPrefix, rec.AccessKeyEnc, rec.SecretKeyEnc, rec.EncryptionKeyVersion, rec.UsePathStyle, rec.PresignedTTLSecs, rec.MaxFileSize, rec.IsActive).Scan(&res.ID, &res.EndpointURL, &res.Region, &res.BucketName, &res.PathPrefix, &res.AccessKeyEnc, &res.SecretKeyEnc, &res.EncryptionKeyVersion, &res.UsePathStyle, &res.PresignedTTLSecs, &res.MaxFileSize, &res.IsActive)
	if err == nil {
		return &res, nil
	}
	if !errors.Is(err, pgx.ErrNoRows) {
		return nil, mapConstraintError(err)
	}

	err = r.SystemDB.QueryRow(ctx, `INSERT INTO storage_configs (id, endpoint_url, region, bucket_name, path_prefix, access_key_enc, secret_key_enc, encryption_key_version, use_path_style, presigned_ttl_secs, max_file_size, is_active, updated_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
RETURNING id, endpoint_url, region, bucket_name, path_prefix, access_key_enc, secret_key_enc, encryption_key_version, use_path_style, presigned_ttl_secs, max_file_size, is_active`,
		id, rec.EndpointURL, rec.Region, rec.BucketName, rec.PathPrefix, rec.AccessKeyEnc, rec.SecretKeyEnc, rec.EncryptionKeyVersion, rec.UsePathStyle, rec.PresignedTTLSecs, rec.MaxFileSize, rec.IsActive).Scan(&res.ID, &res.EndpointURL, &res.Region, &res.BucketName, &res.PathPrefix, &res.AccessKeyEnc, &res.SecretKeyEnc, &res.EncryptionKeyVersion, &res.UsePathStyle, &res.PresignedTTLSecs, &res.MaxFileSize, &res.IsActive)
	if err != nil {
		return nil, mapConstraintError(err)
	}
	return &res, nil
}

func (r StorageConfigRepository) DeleteStorageConfig(ctx context.Context) error {
	if r.SystemDB == nil {
		return errors.New("nil system db")
	}
	_, err := r.SystemDB.Exec(ctx, `DELETE FROM storage_configs`)
	return err
}
