WITH RECURSIVE bookmark_folder_paths AS (
    -- Base Case for Recursion
    -- First we grab all the top-level folders (those
    -- under the toolbar/ and menu/) items.
    -- This gives us a dataset to start recursing for folders
    -- with parents matching these ids.
    SELECT id,
           parent,
           title,
           title as folder_path,
           1 as level
    FROM moz_bookmarks
    WHERE parent IN (2, 3) -- Only children of menu/ and toolbar/
      AND type = 2         -- Only folders

    UNION ALL

    -- Recursion Cases
    -- For each parent item already in the table, we
    -- fetch children based on the child's parent column
    -- matching a row already in the table.
    SELECT b.id,
           b.parent,
           b.title,
           bfp.folder_path || ' / ' || b.title,
           bfp.level + 1
    FROM moz_bookmarks b
             JOIN bookmark_folder_paths bfp ON b.parent = bfp.id
    WHERE b.type = 2 -- Only folders
)
SELECT b.guid,
       CAST(COALESCE(b.title, p.title) AS TEXT) as title,
       CAST(COALESCE(fp.folder_path, '') AS TEXT) as subtitle,
       p.url,
       p.last_visit_date / 1000000 as last_visit,
       b.lastModified / 1000000 as last_modified,
       p.frecency,
       o.frecency as origin_frecency
FROM moz_bookmarks b
         INNER JOIN moz_places p ON b.fk = p.id
         INNER JOIN bookmark_folder_paths fp ON b.parent = fp.id
         INNER JOIN moz_origins o ON o.id = p.origin_id
WHERE b.type = 1 -- Only actual bookmarks
AND p.guid IN ('44EG8dAVYJXq')
ORDER BY subtitle, title;

