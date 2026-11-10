package timezone

import (
	"errors"
	"fmt"
	"time"
)

var ErrInvalidTimezone = errors.New("invalid timezone")

func ParseTimezone(name string) *time.Location {
	if name == "" {
		return time.UTC
	}
	location, err := time.LoadLocation(name)
	if err != nil {
		return time.UTC
	}
	return location
}

func ValidateTZ(name string) (*time.Location, error) {
	if name == "" {
		return nil, fmt.Errorf("%w: timezone is required", ErrInvalidTimezone)
	}
	location, err := time.LoadLocation(name)
	if err != nil {
		return nil, fmt.Errorf("%w: %s", ErrInvalidTimezone, name)
	}
	return location, nil
}

func LocalToUTC(location *time.Location, year, month, day, hour, minute, second int) (time.Time, error) {
	local, err := localTime(location, year, month, day, hour, minute, second)
	if err != nil {
		return time.Time{}, err
	}
	return local.UTC(), nil
}

func LocalToTZ(location *time.Location, year, month, day, hour, minute, second int) (time.Time, error) {
	return localTime(location, year, month, day, hour, minute, second)
}

func NaiveTimeToTZOvernight(clock time.Time, reference time.Time, date time.Time, location *time.Location) (time.Time, error) {
	if location == nil {
		location = time.UTC
	}
	resolvedDate := date.In(location)
	if clock.Hour() < reference.Hour() || clock.Hour() == reference.Hour() && clock.Minute() < reference.Minute() {
		resolvedDate = resolvedDate.AddDate(0, 0, 1)
	}
	return localTime(location, resolvedDate.Year(), int(resolvedDate.Month()), resolvedDate.Day(), clock.Hour(), clock.Minute(), clock.Second())
}

func StartOfDayUTC(location *time.Location, date time.Time) time.Time {
	if location == nil {
		location = time.UTC
	}
	local := date.In(location)
	return time.Date(local.Year(), local.Month(), local.Day(), 0, 0, 0, 0, location).UTC()
}
func StartOfNextDayUTC(location *time.Location, date time.Time) time.Time {
	if location == nil {
		location = time.UTC
	}
	local := date.In(location).AddDate(0, 0, 1)
	return time.Date(local.Year(), local.Month(), local.Day(), 0, 0, 0, 0, location).UTC()
}
func EndOfDayUTC(location *time.Location, date time.Time) time.Time {
	return StartOfNextDayUTC(location, date).Add(-time.Nanosecond)
}

func localTime(location *time.Location, year, month, day, hour, minute, second int) (time.Time, error) {
	if location == nil {
		location = time.UTC
	}
	if month < 1 || month > 12 || day < 1 || day > 31 || hour < 0 || hour > 23 || minute < 0 || minute > 59 || second < 0 || second > 59 {
		return time.Time{}, fmt.Errorf("%w: invalid local date/time", ErrInvalidTimezone)
	}
	return time.Date(year, time.Month(month), day, hour, minute, second, 0, location), nil
}
