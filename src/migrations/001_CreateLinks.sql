CREATE TABLE IF NOT EXISTS links
(
    guid      TEXT PRIMARY KEY,
    url       TEXT NOT NULL,
    title     TEXT NOT NULL,
    subtitle  TEXT,
    source    TEXT,
    timestamp TEXT NOT NULL
);