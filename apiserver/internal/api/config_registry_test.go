package api

import (
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
)

func TestValidateConfigEntry(t *testing.T) {
	weight := decimal.RequireFromString("0.6")
	valid := ConfigEntryRequest{
		ScopeType:     "organization",
		ConfigKey:     "payroll.kpi.output_weight",
		Value:         ConfigValue{Decimal: &weight},
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
		{"removed legacy key", func(r *ConfigEntryRequest) { r.ConfigKey = "payroll.fixed_std_days" }},
		{"wrong type", func(r *ConfigEntryRequest) { text := "fixed"; r.Value = ConfigValue{Text: &text} }},
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

func TestValidateScheduleLifecycleActivationConfig(t *testing.T) {
	employerID := uuid.New()
	enabled := true
	request := ConfigEntryRequest{
		ScopeType: "employer", ScopeID: &employerID, ConfigKey: "schedule.lifecycle.enabled",
		Value: ConfigValue{Bool: &enabled}, EffectiveFrom: "2026-11-13",
	}
	if err := validateConfigEntry(request); err != nil {
		t.Fatalf("valid lifecycle activation rejected: %v", err)
	}
	request.ScopeType = "organization"
	request.ScopeID = nil
	if err := validateConfigEntry(request); err == nil {
		t.Fatal("organization-scoped lifecycle activation should be rejected")
	}
}

func TestValidateKPIParameters(t *testing.T) {
	value := decimal.RequireFromString("0.6")
	for _, key := range []string{"payroll.kpi.output_weight", "payroll.kpi.quality_weight", "payroll.kpi.target_bonus", "payroll.kpi.score_cap", "payroll.kpi.bonus_cap"} {
		if err := validateConfigEntry(ConfigEntryRequest{ConfigKey: key, Value: ConfigValue{Decimal: &value}, EffectiveFrom: "2026-11-14"}); err != nil {
			t.Fatalf("%s rejected: %v", key, err)
		}
	}
}

func TestConfigValueTypeRejectsMultipleValues(t *testing.T) {
	text := "fixed"
	value := ConfigValue{Text: &text, Bool: new(bool)}
	if _, ok := configValueType(value); ok {
		t.Fatal("expected multiple typed values to be rejected")
	}
}
