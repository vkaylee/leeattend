package api

import (
	"bytes"
	"errors"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"
)

func TestParsePagination_DefaultsAndBounds(t *testing.T) {
	p, err := ParsePagination(url.Values{})
	if err != nil || p.Page != 1 || p.Limit != 50 || p.Offset != 0 {
		t.Fatalf("unexpected defaults: %+v, %v", p, err)
	}

	vals := url.Values{"page": []string{"2"}, "limit": []string{"20"}}
	p, err = ParsePagination(vals)
	if err != nil || p.Page != 2 || p.Limit != 20 || p.Offset != 20 {
		t.Fatalf("unexpected values: %+v, %v", p, err)
	}

	_, err = ParsePagination(url.Values{"limit": []string{"3001"}})
	if !errors.Is(err, ErrInvalidPagination) {
		t.Fatalf("expected limit failure, got %v", err)
	}

	_, err = ParsePagination(url.Values{"page": []string{"9223372036854775807"}, "limit": []string{"3000"}})
	if !errors.Is(err, ErrInvalidPagination) {
		t.Fatalf("expected offset overflow failure, got %v", err)
	}
}

func TestDecodeJSON_DisallowUnknownFields(t *testing.T) {
	type payload struct {
		Name string `json:"name"`
	}
	req := httptest.NewRequest(http.MethodPost, "/", bytes.NewBufferString(`{"name":"a","extra":true}`))
	_, err := DecodeJSON[payload](req, 1024)
	if !errors.Is(err, ErrInvalidInput) {
		t.Fatalf("expected unknown field failure, got %v", err)
	}
}
