package external

import (
	"context"
	"embed"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
)

//go:embed testdata/attendance_contract/v1/*.json
var attendanceContractFixtures embed.FS

func attendanceFixture(t *testing.T, name string) []byte {
	t.Helper()
	b, err := attendanceContractFixtures.ReadFile("testdata/attendance_contract/v1/" + name)
	if err != nil {
		t.Fatalf("read fixture %q: %v", name, err)
	}
	return b
}

func TestAttendanceEventsClientContractPages(t *testing.T) {
	server := httptest.NewTLSServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet || r.URL.Path != "/v1/accounts/acct-synthetic/devices/device-synthetic/attendance-events" {
			t.Errorf("unexpected request: %s %s", r.Method, r.URL.Path)
		}
		expectedQuery := "cursor=cursor-synthetic-start&limit=25"
		if r.URL.Query().Get("cursor") == "cursor-synthetic-next" {
			expectedQuery = "cursor=cursor-synthetic-next&limit=25"
		}
		if r.URL.RawQuery != expectedQuery {
			t.Errorf("unexpected query: %s", r.URL.RawQuery)
		}
		if r.Header.Get("Authorization") != "Bearer synthetic-token" || r.Header.Get("X-Request-Id") != "request-synthetic" {
			t.Errorf("unexpected request headers")
		}
		w.Header().Set("Content-Type", "application/json")
		if r.URL.Query().Get("cursor") == "cursor-synthetic-next" {
			_, _ = w.Write(attendanceFixture(t, "empty_terminal_page.json"))
			return
		}
		if r.URL.Query().Get("cursor") != "cursor-synthetic-start" {
			t.Errorf("unexpected cursor: %s", r.URL.RawQuery)
		}
		_, _ = w.Write(attendanceFixture(t, "success_page.json"))
	}))
	defer server.Close()
	client := AttendanceEventsClient{HTTP: &HTTPClient{Client: server.Client()}, BaseURL: server.URL, Token: "synthetic-token", PageLimit: 25}

	page, err := client.GetPage(context.Background(), "acct-synthetic", "device-synthetic", "cursor-synthetic-start", "request-synthetic")
	if err != nil || len(page.Events) != 1 || page.NextCursor != "cursor-synthetic-next" {
		t.Fatalf("unexpected success page: %+v, %v", page, err)
	}
	if page.Events[0].ExternalEventID != "evt-synthetic-001" || page.Events[0].DeviceID != "device-synthetic" || page.Events[0].OccurredAt != "2026-01-15T08:30:00Z" || page.Events[0].Type != "check_in" {
		t.Fatalf("unexpected event: %+v", page.Events[0])
	}
	var eventData map[string]string
	if err := json.Unmarshal(page.Events[0].Data, &eventData); err != nil || eventData["source"] != "contract-test" {
		t.Fatalf("unexpected event data: %s, %v", page.Events[0].Data, err)
	}

	page, err = client.GetPage(context.Background(), "acct-synthetic", "device-synthetic", page.NextCursor, "request-synthetic")
	if err != nil || len(page.Events) != 0 || page.NextCursor != "" {
		t.Fatalf("unexpected terminal page: %+v, %v", page, err)
	}
}

func TestAttendanceEventsClientRequiresHTTPS(t *testing.T) {
	client := AttendanceEventsClient{HTTP: &HTTPClient{Client: http.DefaultClient}, BaseURL: "http://provider.test", Token: "synthetic-token"}
	_, err := client.GetPage(context.Background(), "acct", "dev", "", "")
	if err == nil {
		t.Fatal("expected HTTPS URL validation error")
	}
}

func TestAttendanceEventsClientContractProblems(t *testing.T) {
	tests := []struct {
		name       string
		status     int
		fixture    string
		retryable  bool
		retryAfter string
	}{
		{name: "unauthorized", status: http.StatusUnauthorized, fixture: "unauthorized_problem.json"},
		{name: "rate limited", status: http.StatusTooManyRequests, fixture: "rate_limited_problem.json", retryable: true, retryAfter: "17"},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			server := httptest.NewTLSServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if r.Method != http.MethodGet || r.URL.Path != "/v1/accounts/acct/devices/dev/attendance-events" || r.URL.Query().Get("limit") != "100" {
					t.Errorf("unexpected request: %s %s?%s", r.Method, r.URL.Path, r.URL.RawQuery)
				}
				w.Header().Set("Content-Type", "application/problem+json")
				if tc.retryAfter != "" {
					w.Header().Set("Retry-After", tc.retryAfter)
				}
				w.WriteHeader(tc.status)
				_, _ = w.Write(attendanceFixture(t, tc.fixture))
			}))
			defer server.Close()
			client := AttendanceEventsClient{HTTP: &HTTPClient{Client: server.Client()}, BaseURL: server.URL, Token: "synthetic-token"}
			_, err := client.GetPage(context.Background(), "acct", "dev", "", "")
			var problem *ProblemError
			if !errors.As(err, &problem) || problem.Status != tc.status || problem.Retryable != tc.retryable || problem.RetryAfter != tc.retryAfter {
				t.Fatalf("unexpected problem: %#v", err)
			}
		})
	}
}

func TestAttendanceEventsClientContractMalformedAndCancellation(t *testing.T) {
	server := httptest.NewTLSServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(attendanceFixture(t, "malformed_response.json"))
	}))
	defer server.Close()
	client := AttendanceEventsClient{HTTP: &HTTPClient{Client: server.Client()}, BaseURL: server.URL, Token: "synthetic-token"}
	_, err := client.GetPage(context.Background(), "acct", "dev", "", "")
	if err == nil {
		t.Fatal("expected malformed response error")
	}

	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	_, err = client.GetPage(ctx, "acct", "dev", "", "")
	var problem *ProblemError
	if !errors.As(err, &problem) || !problem.Retryable {
		t.Fatalf("expected retryable cancellation error, got %#v", err)
	}
}
