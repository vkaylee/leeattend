package external

import (
	"bytes"
	"context"
	"io"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"
	"time"
)

func TestS3Client_PresignAndOperations(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if !strings.Contains(r.URL.RawQuery, "X-Amz-Signature=") {
			w.WriteHeader(http.StatusUnauthorized)
			return
		}
		if r.Method == http.MethodPut {
			w.WriteHeader(http.StatusOK)
			return
		}
		if r.Method == http.MethodGet {
			w.WriteHeader(http.StatusOK)
			_, _ = w.Write([]byte("object-content"))
			return
		}
		if r.Method == http.MethodDelete {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		w.WriteHeader(http.StatusBadRequest)
	}))
	defer server.Close()

	client, err := NewS3Client(server.URL, "test-bucket", "test-access", "test-secret", "us-east-1", true, server.Client())
	if err != nil {
		t.Fatal(err)
	}

	presigned, err := client.Presign(context.Background(), http.MethodGet, "reports/summary.csv", "text/csv", 10*time.Minute)
	if err != nil {
		t.Fatal(err)
	}
	u, err := url.Parse(presigned)
	if err != nil || u.Query().Get("X-Amz-Signature") == "" || u.Query().Get("X-Amz-Expires") != "600" {
		t.Fatalf("unexpected presigned URL: %s, %v", presigned, err)
	}

	if err := client.Put(context.Background(), "reports/summary.csv", "text/csv", bytes.NewReader([]byte("1,2,3"))); err != nil {
		t.Fatalf("put failed: %v", err)
	}
	resp, err := client.Get(context.Background(), "reports/summary.csv")
	if err != nil {
		t.Fatalf("get failed: %v", err)
	}
	defer resp.Body.Close()
	body, _ := io.ReadAll(resp.Body)
	if string(body) != "object-content" {
		t.Fatalf("unexpected body: %s", string(body))
	}
	if err := client.Delete(context.Background(), "reports/summary.csv"); err != nil {
		t.Fatalf("delete failed: %v", err)
	}
}
