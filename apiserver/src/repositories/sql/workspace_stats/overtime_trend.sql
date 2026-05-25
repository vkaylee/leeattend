SELECT ads.date,
       COALESCE(SUM(ads.system_detected_ot_minutes), 0)::bigint AS system_ot_minutes,
       COALESCE(SUM(ads.approved_ot_minutes), 0)::bigint AS approved_ot_minutes
FROM attendance_daily_summaries ads
WHERE ads.workspace_id = $1
  AND ads.date >= ($2::date - INTERVAL '29 days')
  AND ads.date <= $2
GROUP BY ads.date
ORDER BY ads.date ASC
