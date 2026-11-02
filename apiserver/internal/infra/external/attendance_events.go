package external

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"strings"
)

const maxAttendanceEventsBody = 4 << 20

// AttendanceEvent is provider event data kept opaque except stable identity and time.
type AttendanceEvent struct {
	ExternalEventID string          `json:"external_event_id"`
	DeviceID        string          `json:"device_id"`
	OccurredAt      string          `json:"occurred_at"`
	Type            string          `json:"type,omitempty"`
	Data            json.RawMessage `json:"data,omitempty"`
}

type AttendanceEventsPage struct {
	Events     []AttendanceEvent `json:"events"`
	NextCursor string            `json:"next_cursor"`
}

// ProblemError represents an RFC 9457 provider error. Retryable marks transport, 429, and 5xx failures.
type ProblemError struct {
	Type       string `json:"type,omitempty"`
	Title      string `json:"title,omitempty"`
	Status     int    `json:"status,omitempty"`
	Detail     string `json:"detail,omitempty"`
	Instance   string `json:"instance,omitempty"`
	Retryable  bool   `json:"-"`
	RetryAfter string `json:"-"`
}

func (e *ProblemError) Error() string {
	if e == nil {
		return "attendance provider error"
	}
	if e.Detail != "" {
		return fmt.Sprintf("attendance provider: %s", e.Detail)
	}
	if e.Title != "" {
		return fmt.Sprintf("attendance provider: %s", e.Title)
	}
	return fmt.Sprintf("attendance provider: status %d", e.Status)
}

// AttendanceEventsClient pulls one provider page. It does not schedule or retry calls.
type AttendanceEventsClient struct {
	HTTP      *HTTPClient
	BaseURL   string
	Token     string
	PageLimit int
}

func (c AttendanceEventsClient) GetPage(ctx context.Context, accountID, deviceID, cursor string, requestID string) (AttendanceEventsPage, error) {
	if c.HTTP == nil || c.HTTP.Client == nil {
		return AttendanceEventsPage{}, errors.New("attendance HTTP client is required")
	}
	if accountID == "" || deviceID == "" {
		return AttendanceEventsPage{}, errors.New("account ID and device ID are required")
	}
	base, err := url.Parse(strings.TrimRight(c.BaseURL, "/"))
	if err != nil || base.Scheme != "https" || base.Host == "" {
		return AttendanceEventsPage{}, errors.New("attendance provider base URL must be HTTPS")
	}
	base.Path = strings.TrimRight(base.Path, "/") + "/v1/accounts/" + url.PathEscape(accountID) + "/devices/" + url.PathEscape(deviceID) + "/attendance-events"
	q := base.Query()
	limit := c.PageLimit
	if limit <= 0 {
		limit = 100
	}
	q.Set("limit", strconv.Itoa(limit))
	if cursor != "" {
		q.Set("cursor", cursor)
	}
	base.RawQuery = q.Encode()
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, base.String(), nil)
	if err != nil {
		return AttendanceEventsPage{}, fmt.Errorf("create attendance request: %w", err)
	}
	req.Header.Set("Authorization", "Bearer "+c.Token)
	if requestID != "" {
		req.Header.Set("X-Request-Id", requestID)
	}
	resp, err := c.HTTP.Client.Do(req)
	if err != nil {
		return AttendanceEventsPage{}, &ProblemError{Title: "transport failure", Detail: "attendance provider request failed", Retryable: true}
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		problem := ProblemError{Status: resp.StatusCode, Retryable: resp.StatusCode == http.StatusTooManyRequests || resp.StatusCode >= 500, RetryAfter: resp.Header.Get("Retry-After")}
		limited := io.LimitReader(resp.Body, maxAttendanceEventsBody)
		if err := json.NewDecoder(limited).Decode(&problem); err != nil {
			problem.Detail = "provider returned an invalid error response"
		}
		problem.Status = resp.StatusCode
		problem.Retryable = resp.StatusCode == http.StatusTooManyRequests || resp.StatusCode >= 500
		problem.RetryAfter = resp.Header.Get("Retry-After")
		return AttendanceEventsPage{}, &problem
	}
	var page AttendanceEventsPage
	if err := json.NewDecoder(io.LimitReader(resp.Body, maxAttendanceEventsBody)).Decode(&page); err != nil {
		return AttendanceEventsPage{}, fmt.Errorf("decode attendance provider response: %w", err)
	}
	return page, nil
}
