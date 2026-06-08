SELECT bucket, COUNT(*)::bigint AS count FROM (
    SELECT CASE
        WHEN emp.end_date - emp.start_date < 90 THEN '< 3 months'
        WHEN emp.end_date - emp.start_date < 180 THEN '3-6 months'
        WHEN emp.end_date - emp.start_date < 365 THEN '6-12 months'
        WHEN emp.end_date - emp.start_date < 730 THEN '1-2 years'
        WHEN emp.end_date - emp.start_date < 1825 THEN '2-5 years'
        ELSE '> 5 years'
    END AS bucket
    FROM employees e
    JOIN employments emp ON e.id = emp.employee_id
    WHERE emp.workspace_id = $1
      AND emp.deleted_at IS NULL
      AND emp.status = 'terminated'
      AND emp.end_date IS NOT NULL
      AND emp.end_date >= $2 AND emp.end_date <= $3
) sub
GROUP BY bucket
ORDER BY CASE bucket
    WHEN '< 3 months' THEN 1
    WHEN '3-6 months' THEN 2
    WHEN '6-12 months' THEN 3
    WHEN '1-2 years' THEN 4
    WHEN '2-5 years' THEN 5
    WHEN '> 5 years' THEN 6
    ELSE 7
END
