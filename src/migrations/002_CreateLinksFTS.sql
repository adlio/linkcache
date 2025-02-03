CREATE VIRTUAL TABLE IF NOT EXISTS links_fts USING fts5 (
    guid, url, title, subtitle, source,
    tokenize='trigram'
);


CREATE TRIGGER IF NOT EXISTS links_upsert AFTER INSERT ON links
BEGIN
    DELETE FROM links_fts WHERE guid = new.guid;
    INSERT INTO links_fts
    (guid, url, title, subtitle, source)
    VALUES
    (new.guid, new.url, new.title, new.subtitle, new.source);
END;


CREATE TRIGGER IF NOT EXISTS links_update AFTER UPDATE ON links
BEGIN
    DELETE FROM links_fts WHERE guid = new.guid;

    INSERT INTO links_fts
    (guid, url, title, subtitle, source)
    VALUES
    (new.guid, new.url, new.title, new.subtitle, new.source);
END;


CREATE TRIGGER IF NOT EXISTS links_delete BEFORE DELETE ON links
BEGIN
    DELETE FROM links_fts WHERE guid = old.guid;
END;