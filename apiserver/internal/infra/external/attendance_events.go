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

// AttendanceEvent carries provider-side attendance-source identity.
type AttendanceEvent struct {
	ExternalEventID  string          `json:"external_event_id"`
	ProviderSourceID string          `json:"source_id"`
	OccurredAt       string          `json:"occurred_at"`
	Type             string          `json:"type,omitempty"`
	Data             json.RawMessage `json:"data,omitempty"`
}

type AttendanceEventsPage struct {
	Events     []AttendanceEvent `json:"events"`
	NextCursor string            `json:"next_cursor"`
}

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
		return "external provider error"
	}
	if e.Title != "" {
		return e.Title
	}
	if e.Detail != "" {
		return e.Detail
	}
	return fmt.Sprintf("external provider returned HTTP %d", e.Status)
}

type AttendanceEventsClient struct {
	HTTP      *HTTPClient
	BaseURL   string
	Token     string
	PageLimit int
}

func (c AttendanceEventsClient) GetPage(ctx context.Context, accountID, sourceID, cursor string, requestID string) (AttendanceEventsPage, error) {
	if c.HTTP == nil || c.HTTP.Client == nil {
		return AttendanceEventsPage{}, errors.New("attendance HTTP client is required")
	}
	if accountID == "" || sourceID == "" {
		return AttendanceEventsPage{}, errors.New("account ID and source ID are required")
	}
	base, err := url.Parse(strings.TrimRight(c.BaseURL, "/"))
	if err != nil || base.Scheme != "https" || base.Host == "" {
		return AttendanceEventsPage{}, errors.New("attendance provider URL must be HTTPS")
	}
	base.Path = strings.TrimRight(base.Path, "/") + "/v1/accounts/" + url.PathEscape(accountID) + "/sources/" + url.PathEscape(sourceID) + "/attendance-events"
	q := base.Query()
	limit := c.PageLimit
	if limit <= 0 {
		limit = 100
	}
	q.Set("cursor", cursor)
	q.Set("limit", strconv.Itoa(limit))
	base.RawQuery = q.Encode()
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, base.String(), nil)
	if err != nil {
		return AttendanceEventsPage{}, err
	}
	req.Header.Set("Accept", "application/json")
	if requestID != "" {
		req.Header.Set("X-Request-ID", requestID)
	}
	if c.Token != "" {
		req.Header.Set("Authorization", "Bearer "+c.Token)
	}
	res, err := c.HTTP.Client.Do(req)
	if err != nil {
		if errors.Is(err, context.Canceled) || errors.Is(err, context.DeadlineExceeded) {
			return AttendanceEventsPage{}, &ProblemError{Status: 0, Retryable: true, Detail: "attendance provider request canceled"}
		}
		return AttendanceEventsPage{}, err
	}
	defer res.Body.Close()
	body, err := io.ReadAll(io.LimitReader(res.Body, maxAttendanceEventsBody+1))
	if err != nil {
		return AttendanceEventsPage{}, err
	}
	if len(body) > maxAttendanceEventsBody {
		return AttendanceEventsPage{}, errors.New("attendance provider response exceeds limit")
	}
	if res.StatusCode < 200 || res.StatusCode >= 300 {
		problem := &ProblemError{Status: res.StatusCode, Retryable: res.StatusCode == http.StatusTooManyRequests || res.StatusCode >= 500, RetryAfter: res.Header.Get("Retry-After")}
		_ = json.Unmarshal(body, problem)
		return AttendanceEventsPage{}, problem
	}
	var page AttendanceEventsPage
	if err := json.Unmarshal(body, &page); err != nil {
		return AttendanceEventsPage{}, fmt.Errorf("decode attendance provider response: %w", err)
	}
	return page, nil
}
