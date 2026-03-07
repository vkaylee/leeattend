-- Allow multiple shifts per day in working patterns (split shifts / ca gãy)
ALTER TABLE working_pattern_details
  DROP CONSTRAINT working_pattern_details_pattern_id_day_of_week_key;
