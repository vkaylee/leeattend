WITH latest_dept AS (
    SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
    FROM employee_department_history edh
    JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
    WHERE edh.effective_date <= $3
    ORDER BY edh.employee_id, edh.effective_date DESC
),
first_punches AS (
    SELECT te.employee_id, MIN(te.punch_in_time AT TIME ZONE $4) AS first_punch
    FROM time_entries te
    WHERE (te.punch_in_time AT TIME ZONE $4)::date = $3
      AND te.employee_id IN (SELECT employee_id FROM latest_dept)
    GROUP BY te.employee_id
)
SELECT ld.department_id,
       COUNT(DISTINCT fp.employee_id)::bigint AS late_count
FROM first_punches fp
JOIN latest_dept ld ON ld.employee_id = fp.employee_id
JOIN shift_rosters re ON re.employee_id = fp.employee_id AND re.date = $3
JOIN shifts s ON s.id = re.shift_id
WHERE fp.first_punch::time > s.start_time
GROUP BY ld.department_id
