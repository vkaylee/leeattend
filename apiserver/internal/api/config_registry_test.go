package api

import (
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

func TestValidateConfigEntry(t *testing.T) {
	text := "fixed"
	valid := ConfigEntryRequest{
		ScopeType:     "organization",
		ConfigKey:     "payroll.std_working_day_mode",
		Value:         ConfigValue{Text: &text},
		EffectiveFrom: "2026-11-12",
	}
	if err := validateConfigEntry(valid); err != nil {
		t.Fatalf("valid config rejected: %v", err)
	}

	cases := []struct {
		name string
		edit func(*ConfigEntryRequest)
	}{
		{"unknown key", func(r *ConfigEntryRequest) { r.ConfigKey = "payroll.unknown" }},
		{"wrong type", func(r *ConfigEntryRequest) { value := decimal.NewFromInt(1); r.Value = ConfigValue{Decimal: &value} }},
		{"wrong scope", func(r *ConfigEntryRequest) { r.ScopeType = "employee"; id := uuid.New(); r.ScopeID = &id }},
		{"missing scope id", func(r *ConfigEntryRequest) { r.ScopeType = "department" }},
		{"invalid date", func(r *ConfigEntryRequest) { r.EffectiveFrom = "12/11/2026" }},
		{"inverted date range", func(r *ConfigEntryRequest) { to := "2026-11-01"; r.EffectiveTo = &to }},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			request := valid
			tc.edit(&request)
			if err := validateConfigEntry(request); err == nil {
				t.Fatal("expected validation error")
			}
		})
	}
}

func TestConfigValueTypeRejectsMultipleValues(t *testing.T) {
	text := "fixed"
	value := ConfigValue{Text: &text, Bool: new(bool)}
	if _, ok := configValueType(value); ok {
		t.Fatal("expected multiple typed values to be rejected")
	}
}
