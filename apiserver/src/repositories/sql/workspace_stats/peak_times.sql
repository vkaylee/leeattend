WITH slots AS (
    SELECT generate_series(0, 47) AS slot_index
),
slot_labels AS (
    SELECT slot_index,
           TO_CHAR((TIME '00:00' + (slot_index * INTERVAL '30 minutes')), 'HH24:MI')
           || '-' ||
           TO_CHAR((TIME '00:00' + ((slot_index + 1) * INTERVAL '30 minutes')), 'HH24:MI')
           AS time_slot,
           (TIME '00:00' + (slot_index * INTERVAL '30 minutes')) AS slot_start,
           (TIME '00:00' + ((slot_index + 1) * INTERVAL '30 minutes')) AS slot_end
    FROM slots
),
today_entries AS (
    SELECT te.punch_in_time, te.punch_out_time
    FROM time_entries te
    JOIN employments emp ON te.employment_id = emp.id AND emp.deleted_at IS NULL
    WHERE te.punch_in_time >= $1
      AND emp.workspace_id = $2
      AND emp.status = 'active'
      AND CURRENT_DATE BETWEEN emp.start_date AND COALESCE(emp.end_date, '9999-12-31'::date)
)
SELECT sl.time_slot::text,
       COALESCE(SUM(CASE WHEN te.punch_in_time::time >= sl.slot_start
                          AND te.punch_in_time::time < sl.slot_end
                         THEN 1 ELSE 0 END), 0)::bigint AS check_in,
       COALESCE(SUM(CASE WHEN te.punch_out_time IS NOT NULL
                          AND te.punch_out_time::time >= sl.slot_start
                          AND te.punch_out_time::time < sl.slot_end
                         THEN 1 ELSE 0 END), 0)::bigint AS check_out
FROM slot_labels sl
LEFT JOIN today_entries te ON TRUE
GROUP BY sl.slot_index, sl.time_slot
HAVING COALESCE(SUM(CASE WHEN te.punch_in_time::time >= sl.slot_start
                          AND te.punch_in_time::time < sl.slot_end
                         THEN 1 ELSE 0 END), 0) > 0
    OR COALESCE(SUM(CASE WHEN te.punch_out_time IS NOT NULL
                          AND te.punch_out_time::time >= sl.slot_start
                          AND te.punch_out_time::time < sl.slot_end
                         THEN 1 ELSE 0 END), 0) > 0
ORDER BY sl.slot_index
