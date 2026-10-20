package api

import (
	"context"
	"fmt"
	"net/http"

	"github.com/vkaylee/leeattend/apiserver/internal/infra/external"
)

type s3StorageTester struct{ endpoint string }

// NewS3StorageTester creates connection test implementation. Empty endpoint means
// S3 is disabled; test succeeds without network access.
func NewS3StorageTester(endpoint string) StorageConnectionTester {
	return s3StorageTester{endpoint: endpoint}
}

func (s s3StorageTester) Test(ctx context.Context, req StorageConfigRequest) error {
	if s.endpoint == "" && req.EndpointURL == "" {
		return nil
	}
	endpoint := req.EndpointURL
	if endpoint == "" {
		endpoint = s.endpoint
	}
	client, err := external.NewS3Client(endpoint, req.BucketName, req.AccessKey, req.SecretKey, req.Region, req.UsePathStyle, http.DefaultClient)
	if err != nil {
		return fmt.Errorf("storage client: %w", err)
	}
	resp, err := client.Get(ctx, "")
	if err != nil {
		return fmt.Errorf("storage health check: %w", err)
	}
	_ = resp.Body.Close()
	if resp.StatusCode >= http.StatusBadRequest {
		return fmt.Errorf("storage health check returned %s", resp.Status)
	}
	return nil
}
