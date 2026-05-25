SELECT COUNT(*)::bigint as count
FROM employees e
WHERE e.workspace_id = $1 AND e.is_active = FALSE
  AND e.termination_date IS NOT NULL
  AND e.termination_date >= $2 AND e.termination_date <= $3
