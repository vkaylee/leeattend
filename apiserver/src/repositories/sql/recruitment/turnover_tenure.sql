SELECT bucket, COUNT(*)::bigint AS count FROM (
    SELECT CASE
        WHEN e.termination_date - e.hire_date < 90 THEN '< 3 months'
        WHEN e.termination_date - e.hire_date < 180 THEN '3-6 months'
        WHEN e.termination_date - e.hire_date < 365 THEN '6-12 months'
        WHEN e.termination_date - e.hire_date < 730 THEN '1-2 years'
        WHEN e.termination_date - e.hire_date < 1825 THEN '2-5 years'
        ELSE '> 5 years'
    END AS bucket
    FROM employees e
    WHERE e.workspace_id = $1 AND e.is_active = FALSE
      AND e.termination_date IS NOT NULL
      AND e.termination_date >= $2 AND e.termination_date <= $3
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
