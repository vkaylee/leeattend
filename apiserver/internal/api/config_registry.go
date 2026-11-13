package api

import (
	"encoding/json"
	"errors"
	"strings"
	"time"

	"github.com/google/uuid"
)

type ConfigValueType string

const (
	ConfigValueInt     ConfigValueType = "int"
	ConfigValueDecimal ConfigValueType = "decimal"
	ConfigValueText    ConfigValueType = "text"
	ConfigValueBool    ConfigValueType = "bool"
	ConfigValueJSON    ConfigValueType = "json"
)

type ConfigKeyDefinition struct {
	Type           ConfigValueType
	AllowedScopes  []string
	EffectiveDated bool
}

var configKeyRegistry = map[string]ConfigKeyDefinition{
	"payroll.std_working_day_mode":   {Type: ConfigValueText, AllowedScopes: []string{"organization"}, EffectiveDated: true},
	"payroll.fixed_std_days":         {Type: ConfigValueDecimal, AllowedScopes: []string{"organization", "department", "employment"}, EffectiveDated: true},
	"payroll.working_days_per_week":  {Type: ConfigValueInt, AllowedScopes: []string{"organization"}, EffectiveDated: true},
	"payroll.hours_per_day":          {Type: ConfigValueDecimal, AllowedScopes: []string{"organization", "department", "employment"}, EffectiveDated: true},
	"payroll.max_ot_hours_per_day":   {Type: ConfigValueDecimal, AllowedScopes: []string{"organization"}, EffectiveDated: true},
	"payroll.max_ot_hours_per_month": {Type: ConfigValueDecimal, AllowedScopes: []string{"organization"}, EffectiveDated: true},
	"payroll.kpi.output_weight":      {Type: ConfigValueDecimal, AllowedScopes: []string{"organization"}, EffectiveDated: true},
	"payroll.kpi.quality_weight":     {Type: ConfigValueDecimal, AllowedScopes: []string{"organization"}, EffectiveDated: true},
	"payroll.kpi.target_bonus":       {Type: ConfigValueDecimal, AllowedScopes: []string{"organization"}, EffectiveDated: true},
	"payroll.kpi.score_cap":          {Type: ConfigValueDecimal, AllowedScopes: []string{"organization"}, EffectiveDated: true},
	"payroll.kpi.bonus_cap":          {Type: ConfigValueDecimal, AllowedScopes: []string{"organization"}, EffectiveDated: true},
	"overtime.max_hours_per_month":   {Type: ConfigValueDecimal, AllowedScopes: []string{"organization", "employer", "department", "employee", "employment"}, EffectiveDated: true},
	"schedule.lifecycle.enabled":     {Type: ConfigValueBool, AllowedScopes: []string{"employer"}, EffectiveDated: true},
}

var (
	errUnknownConfigKey = errors.New("unknown config key")
	errConfigValueType  = errors.New("config value type does not match registry")
	errConfigScope      = errors.New("config scope is not allowed for key")
	errConfigScopeID    = errors.New("config scope id is required or forbidden")
	errConfigDate       = errors.New("config effective date is invalid")
	errConfigDateRange  = errors.New("config effective date range is invalid")
)

func configValueType(value ConfigValue) (ConfigValueType, bool) {
	count := 0
	var valueType ConfigValueType
	if value.Int != nil {
		count++
		valueType = ConfigValueInt
	}
	if value.Decimal != nil {
		count++
		valueType = ConfigValueDecimal
	}
	if value.Text != nil {
		count++
		valueType = ConfigValueText
	}
	if value.Bool != nil {
		count++
		valueType = ConfigValueBool
	}
	if len(value.JSON) > 0 && json.Valid(value.JSON) {
		count++
		valueType = ConfigValueJSON
	}
	return valueType, count == 1
}

func validateConfigEntry(request ConfigEntryRequest) error {
	key := strings.TrimSpace(request.ConfigKey)
	definition, ok := configKeyRegistry[key]
	if !ok {
		return errUnknownConfigKey
	}

	scope := request.ScopeType
	if scope == "" {
		scope = "organization"
	}
	if !isSupportedConfigScope(scope) {
		return errConfigScope
	}
	if scope == "organization" {
		if request.ScopeID != nil {
			return errConfigScopeID
		}
	} else if request.ScopeID == nil || *request.ScopeID == uuid.Nil {
		return errConfigScopeID
	}

	allowed := false
	for _, candidate := range definition.AllowedScopes {
		if candidate == scope {
			allowed = true
			break
		}
	}
	if !allowed {
		return errConfigScope
	}

	valueType, validValue := configValueType(request.Value)
	if !validValue || valueType != definition.Type {
		return errConfigValueType
	}

	from, err := time.Parse("2006-01-02", strings.TrimSpace(request.EffectiveFrom))
	if err != nil {
		return errConfigDate
	}
	if request.EffectiveTo != nil && strings.TrimSpace(*request.EffectiveTo) != "" {
		to, parseErr := time.Parse("2006-01-02", strings.TrimSpace(*request.EffectiveTo))
		if parseErr != nil {
			return errConfigDate
		}
		if to.Before(from) {
			return errConfigDateRange
		}
	}
	return nil
}
