
SELECT p.guid,
       CAST(COALESCE(p.title, '') AS TEXT) as title,
       p.url,
       p.last_visit_date / 1000000 as last_visit,
       p.frecency,
       o.frecency as origin_frecency
FROM moz_places p
         INNER JOIN moz_origins o ON o.id = p.origin_id
WHERE p.frecency > 10000
ORDER BY p.frecency DESC;
