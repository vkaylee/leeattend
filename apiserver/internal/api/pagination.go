package api

import (
	"errors"
	"fmt"
	"math"
	"net/url"
	"strconv"
)

const (
	DefaultPage  = 1
	DefaultLimit = 50
	MaxLimit     = 3000
)

var ErrInvalidPagination = errors.New("invalid pagination")

type Pagination struct{ Page, Limit, Offset int }

func ParsePagination(values url.Values) (Pagination, error) {
	page, limit := DefaultPage, DefaultLimit
	var err error
	if raw := values.Get("page"); raw != "" {
		page, err = strconv.Atoi(raw)
		if err != nil {
			return Pagination{}, fmt.Errorf("%w: page", ErrInvalidPagination)
		}
	}
	if raw := values.Get("limit"); raw != "" {
		limit, err = strconv.Atoi(raw)
		if err != nil {
			return Pagination{}, fmt.Errorf("%w: limit", ErrInvalidPagination)
		}
	}
	if page < 1 || limit < 1 || limit > MaxLimit {
		return Pagination{}, ErrInvalidPagination
	}
	if page > (math.MaxInt/limit)+1 {
		return Pagination{}, ErrInvalidPagination
	}
	return Pagination{Page: page, Limit: limit, Offset: (page - 1) * limit}, nil
}

type SortOption struct {
	Field string
	Desc  bool
}

func ParseSort(raw string, allowlist map[string]string) (SortOption, error) {
	if raw == "" {
		return SortOption{}, nil
	}
	desc := raw[0] == '-'
	field := raw
	if desc {
		field = raw[1:]
	}
	if allowlist[field] == "" {
		return SortOption{}, fmt.Errorf("sort field %q is not allowed", field)
	}
	return SortOption{Field: allowlist[field], Desc: desc}, nil
}
