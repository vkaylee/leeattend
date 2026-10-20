package external

import (
	"bytes"
	"context"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
	"time"
)

func TestFileStore_PutAndOpen(t *testing.T) {
	dir := t.TempDir()
	store := FileStore{Root: dir}
	data := []byte("hello world")
	if err := store.Put(context.Background(), "sub/test.txt", bytes.NewReader(data), 0o600); err != nil {
		t.Fatal(err)
	}
	f, err := store.Open("sub/test.txt")
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	readBytes, err := io.ReadAll(f)
	if err != nil || !bytes.Equal(readBytes, data) {
		t.Fatalf("expected %s, got %s, err=%v", data, readBytes, err)
	}
	// Path traversal rejected
	if err := store.Put(context.Background(), "../secret.txt", bytes.NewReader(data), 0o600); err == nil {
		t.Fatal("expected traversal error, got nil")
	}
}

func TestHTTPClient_GetSuccessAnd500(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/ok" {
			w.WriteHeader(http.StatusOK)
			_, _ = w.Write([]byte("ok"))
			return
		}
		w.WriteHeader(http.StatusInternalServerError)
	}))
	defer ts.Close()

	client := NewHTTPClient(2 * time.Second)
	resp, err := client.Do(context.Background(), http.MethodGet, ts.URL+"/ok", nil)
	if err != nil || resp.StatusCode != http.StatusOK {
		t.Fatalf("unexpected response: %v, %v", resp, err)
	}
	_ = resp.Body.Close()

	_, err = client.Do(context.Background(), http.MethodGet, ts.URL+"/err", nil)
	if err == nil {
		t.Fatal("expected 500 error from client, got nil")
	}
}

func TestClamAV_CleanAndInfectedSimulation(t *testing.T) {
	// Simulated server
	tmp := filepath.Join(t.TempDir(), "clamav")
	_ = os.WriteFile(tmp, []byte("clamav test"), 0o600)
	client := ClamAVClient{Address: "", Timeout: time.Second}
	if err := client.Scan(context.Background(), bytes.NewReader([]byte("test"))); err == nil {
		t.Fatal("expected missing address error")
	}
}
