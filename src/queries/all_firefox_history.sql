SELECT p.guid,
       p.url,
       CAST(COALESCE(p.title, '') AS TEXT)      as title,
       COALESCE(p.last_visit_date / 1000000, 0) as last_visit,
       COALESCE(p.frecency, 0)                  as frecency,
       COALESCE(o.frecency, 0)                  as origin_frecency
FROM moz_places p
         LEFT JOIN moz_origins o ON o.id = p.origin_id
WHERE 1 = 1
  AND (
    (p.frecency >= 500)
        OR (p.frecency >= 100 AND o.frecency >= 1000)
    )
  AND p.url NOT LIKE 'https://www.google.com/search%'
ORDER BY frecency DESC LIMIT 5000;

