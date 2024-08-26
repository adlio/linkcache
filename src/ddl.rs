use crate::Cache;
use crate::Result;

impl Cache {
    /// Initializes the index, its schema, and custom tokenization
    pub(crate) fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS links (
                url TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                subtitle TEXT,
                source TEXT,
                author TEXT,
                timestamp TEXT NOT NULL
            );


            CREATE VIRTUAL TABLE IF NOT EXISTS links_fts USING fts5 (
                url, title, subtitle, source, author,
                tokenize='trigram'
            );


            CREATE TRIGGER IF NOT EXISTS links_insert AFTER INSERT ON links
            BEGIN
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


            CREATE TRIGGER IF NOT EXISTS links_delete AFTER DELETE ON links
            BEGIN
                DELETE FROM links_fts WHERE url = old.url;
            END;
            ",
        )?;
        Ok(())
    }
}
