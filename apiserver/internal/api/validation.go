package api

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

var (
	ErrInvalidInput = errors.New("invalid input")
	ErrNotFound     = errors.New("not found")
)

func DecodeJSON[T any](r *http.Request, limit int64) (T, error) {
	var value T
	if r.Body == nil {
		return value, fmt.Errorf("%w: empty body", ErrInvalidInput)
	}
	if limit > 0 {
		r.Body = http.MaxBytesReader(nil, r.Body, limit)
	}
	decoder := json.NewDecoder(r.Body)
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&value); err != nil {
		return value, fmt.Errorf("%w: %w", ErrInvalidInput, err)
	}
	var extra any
	if err := decoder.Decode(&extra); err != io.EOF {
		return value, fmt.Errorf("%w: multiple JSON values", ErrInvalidInput)
	}
	return value, nil
}

func ParseUUID(raw string) (uuid.UUID, error) {
	id, err := uuid.Parse(strings.TrimSpace(raw))
	if err != nil || id == uuid.Nil {
		return uuid.Nil, fmt.Errorf("%w: uuid", ErrInvalidInput)
	}
	return id, nil
}

func ParseDecimal(raw string) (decimal.Decimal, error) {
	value, err := decimal.NewFromString(strings.TrimSpace(raw))
	if err != nil {
		return decimal.Zero, fmt.Errorf("%w: decimal", ErrInvalidInput)
	}
	return value, nil
}

func ParseDate(raw string) (time.Time, error) {
	value, err := time.Parse("2006-01-02", raw)
	if err != nil {
		return time.Time{}, fmt.Errorf("%w: date", ErrInvalidInput)
	}
	return value, nil
}

func ParseTimezone(raw string) (*time.Location, error) {
	location, err := time.LoadLocation(raw)
	if err != nil {
		return nil, fmt.Errorf("%w: timezone", ErrInvalidInput)
	}
	return location, nil
}
