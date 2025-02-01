CREATE TABLE IF NOT EXISTS links (
    url TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    subtitle TEXT,
    source TEXT,
    author TEXT,
    timestamp TEXT NOT NULL
);