use lazy_static::lazy_static;
use log::info;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::Result;

lazy_static! {
    static ref MIGRATIONS: Migrations<'static> = Migrations::new(vec![
        M::up(include_str!("./migrations/001_CreateLinks.sql")),
        M::up(include_str!("./migrations/002_CreateLinksFTS.sql")),
    ]);
}

/// Runs an embedded set of SQL migrations to ensure the connected database
/// has all appropriate DDL in place before the application begins operating
///
pub(crate) fn apply_migrations(conn: &mut Connection) -> Result<()> {
    info!("Applying migrations...");
    conn.pragma_update(None, "journal_mode", "WAL")?;
    MIGRATIONS.to_latest(conn)?;
    info!("Migrations completed");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::MIGRATIONS;

    #[test]
    fn migrations_test() {
        assert!(MIGRATIONS.validate().is_ok());
    }
}