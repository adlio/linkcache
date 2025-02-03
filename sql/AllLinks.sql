SELECT
             links.guid, links.url, links.title,
             links.subtitle, links.source,
             links.timestamp
             FROM links_fts
             JOIN links ON links_fts.url = links.url
             WHERE links_fts MATCH "lawyer update"
             ORDER BY rank;


SELECT guid, COUNT(*)
FROM links_fts
GROUP BY guid
HAVING COUNT(*) > 1;