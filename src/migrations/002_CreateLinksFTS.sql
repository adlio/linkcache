CREATE VIRTUAL TABLE IF NOT EXISTS links_fts USING fts5 (
    url, title, subtitle, source, author,
    tokenize='trigram'
);


CREATE TRIGGER IF NOT EXISTS links_upsert AFTER INSERT ON links
BEGIN
    DELETE FROM links_fts WHERE url = new.url AND title = new.title;
    INSERT INTO links_fts
    (url, title, subtitle, source, author)
    VALUES
    (new.url, new.title, new.subtitle, new.source, new.author);
END;


CREATE TRIGGER IF NOT EXISTS links_update AFTER UPDATE ON links
BEGIN
    INSERT OR REPLACE INTO links_fts
    (url, title, subtitle, source, author)
    VALUES
    (new.url, new.title, new.subtitle, new.source, new.author);
END;


CREATE TRIGGER IF NOT EXISTS links_delete BEFORE DELETE ON links
BEGIN
    DELETE FROM links_fts WHERE url = old.url;
END;