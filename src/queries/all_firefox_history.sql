SELECT p.guid,
       p.url,
       CAST(COALESCE(p.title, '') AS TEXT)      as title,
       COALESCE(p.last_visit_date / 1000000, 0) as last_visit,
       COALESCE(p.frecency, 0)                  as frecency,
       COALESCE(o.frecency, 0)                  as origin_frecency
FROM moz_places p
         LEFT JOIN moz_origins o ON o.id = p.origin_id
ORDER BY frecency DESC;

SELECT *
FROM moz_origins;