package testutil

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"
)

// Fixture is the executable subset of the canonical migration fixture format.
type Fixture struct {
	SchemaVersion string `json:"schema_version"`
	Metadata      struct {
		ID string `json:"id"`
	} `json:"metadata"`
	Request struct {
		Method     string            `json:"method"`
		Path       string            `json:"path"`
		PathParams map[string]string `json:"path_params"`
		Query      map[string]any    `json:"query"`
		Headers    map[string]string `json:"headers"`
		Body       json.RawMessage   `json:"body"`
	} `json:"request"`
	ExpectedResponse struct {
		Status  int               `json:"status"`
		Headers map[string]string `json:"headers"`
		Body    json.RawMessage   `json:"body"`
	} `json:"expected_response"`
}

// Result is deterministic machine-readable response capture.
type Result struct {
	FixtureID string              `json:"fixture_id"`
	BaseURL   string              `json:"base_url"`
	Status    int                 `json:"status"`
	Headers   map[string][]string `json:"headers"`
	Body      json.RawMessage     `json:"body"`
	Duration  time.Duration       `json:"duration_ns"`
}

// LoadFixture decodes one canonical fixture JSON document.
func LoadFixture(data []byte) (Fixture, error) {
	var fixture Fixture
	if err := json.Unmarshal(data, &fixture); err != nil {
		return Fixture{}, fmt.Errorf("decode fixture: %w", err)
	}
	if fixture.SchemaVersion == "" || fixture.Metadata.ID == "" || fixture.Request.Method == "" || fixture.Request.Path == "" {
		return Fixture{}, fmt.Errorf("fixture missing required identity or request fields")
	}
	return fixture, nil
}

// BaseURL selects the immutable default runtime endpoint.
func BaseURL(goRuntime bool) string {
	if goRuntime {
		return "http://127.0.0.1:3001"
	}
	return "http://127.0.0.1:3000"
}

// RunFixture executes fixture against baseURL. No normalization is applied here;
// comparison code must apply only explicitly declared P2-08 rules.
func RunFixture(client *http.Client, fixture Fixture, baseURL string) (Result, error) {
	if client == nil {
		client = http.DefaultClient
	}
	path := fixture.Request.Path
	for key, value := range fixture.Request.PathParams {
		path = strings.ReplaceAll(path, "{"+key+"}", url.PathEscape(value))
	}
	u, err := url.Parse(strings.TrimRight(baseURL, "/") + path)
	if err != nil {
		return Result{}, fmt.Errorf("fixture %s: build URL: %w", fixture.Metadata.ID, err)
	}
	query := u.Query()
	for key, value := range fixture.Request.Query {
		query.Set(key, fmt.Sprint(value))
	}
	u.RawQuery = query.Encode()

	body := fixture.Request.Body
	if len(body) == 0 || string(body) == "null" {
		body = nil
	}
	req, err := http.NewRequest(fixture.Request.Method, u.String(), bytes.NewReader(body))
	if err != nil {
		return Result{}, fmt.Errorf("fixture %s: build request: %w", fixture.Metadata.ID, err)
	}
	for key, value := range fixture.Request.Headers {
		req.Header.Set(key, value)
	}

	started := time.Now()
	resp, err := client.Do(req)
	if err != nil {
		return Result{}, fmt.Errorf("fixture %s: request: %w", fixture.Metadata.ID, err)
	}
	defer resp.Body.Close()
	responseBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return Result{}, fmt.Errorf("fixture %s: read response: %w", fixture.Metadata.ID, err)
	}
	headers := make(map[string][]string, len(resp.Header))
	for key, values := range resp.Header {
		headers[key] = append([]string(nil), values...)
	}
	return Result{
		FixtureID: fixture.Metadata.ID,
		BaseURL:   strings.TrimRight(baseURL, "/"),
		Status:    resp.StatusCode,
		Headers:   headers,
		Body:      json.RawMessage(responseBody),
		Duration:  time.Since(started),
	}, nil
}
