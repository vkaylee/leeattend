WITH today_entries AS (
    SELECT te.punch_in_time, te.punch_out_time
    FROM time_entries te
    JOIN employments emp ON te.employment_id = emp.id AND emp.deleted_at IS NULL
    WHERE te.punch_in_time >= $1
      AND te.punch_in_time < $2
      AND emp.workspace_id = $3
      AND emp.status = 'active'
      AND CURRENT_DATE BETWEEN emp.start_date AND COALESCE(emp.end_date, '9999-12-31'::date)
),
check_ins AS (
    SELECT
        (EXTRACT(HOUR FROM punch_in_time)::int * 2
         + FLOOR(EXTRACT(MINUTE FROM punch_in_time) / 30)::int) AS slot_index,
        COUNT(*) AS check_in
    FROM today_entries
    GROUP BY slot_index
),
check_outs AS (
    SELECT
        (EXTRACT(HOUR FROM punch_out_time)::int * 2
         + FLOOR(EXTRACT(MINUTE FROM punch_out_time) / 30)::int) AS slot_index,
        COUNT(*) AS check_out
    FROM today_entries
    WHERE punch_out_time IS NOT NULL
    GROUP BY slot_index
),
slots AS (
    SELECT generate_series(0, 47) AS slot_index
),
slot_labels AS (
    SELECT slot_index,
           TO_CHAR((TIME '00:00' + (slot_index * INTERVAL '30 minutes')), 'HH24:MI')
           || '-' ||
           TO_CHAR((TIME '00:00' + ((slot_index + 1) * INTERVAL '30 minutes')), 'HH24:MI')
           AS time_slot
    FROM slots
)
SELECT sl.time_slot::text,
       COALESCE(ci.check_in, 0)::bigint AS check_in,
       COALESCE(co.check_out, 0)::bigint AS check_out
FROM slot_labels sl
LEFT JOIN check_ins ci ON sl.slot_index = ci.slot_index
LEFT JOIN check_outs co ON sl.slot_index = co.slot_index
WHERE COALESCE(ci.check_in, 0) > 0 OR COALESCE(co.check_out, 0) > 0
ORDER BY sl.slot_index
