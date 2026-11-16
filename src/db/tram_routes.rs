

use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use crate::db::resources::tram_route::TramRoute;

pub static SQL_TRAM_ROUTE: &str = r#"
CREATE TABLE IF NOT EXISTS tram_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    start_stop_id INTEGER REFERENCES tram_stops(id) NOT NULL,
    end_stop_id INTEGER REFERENCES tram_stops(id) NOT NULL,
    operator_id INTEGER REFERENCES tram_operators(id) NOT NULL,
    created_at TEXT NOT NULL
)
"#;

#[derive(Clone)]
pub struct TramRoutes {
    pool: Pool<SqliteConnectionManager>,
}

impl TramRoutes {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<TramRoute>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, name, start_stop_id, end_stop_id, operator_id, created_at
            FROM tram_routes
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_tram_route(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &str) -> rusqlite::Result<Vec<TramRoute>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, name, start_stop_id, end_stop_id, operator_id, created_at
            FROM tram_routes
            WHERE
                name LIKE ?
            ORDER BY name
            LIMIT 50
            "#,
        )?;

        let search = format!("%{}%", query);

        let rows = statement.query_map(
            rusqlite::params![search],
            Self::row_to_tram_route,
        )?;

        rows.collect()
    }

    pub fn create(&self, route: &TramRoute) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO tram_routes (
                name,
                start_stop_id,
                end_stop_id,
                operator_id,
                created_at
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
            rusqlite::params![
                &route.name,
                route.start_stop_id,
                route.end_stop_id,
                route.operator_id,
                &route.created_at,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(&self, route: &TramRoute) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let updated = connection.execute(
            r#"
            UPDATE tram_routes
            SET
                name = ?,
                start_stop_id = ?,
                end_stop_id = ?,
                operator_id = ?,
                created_at = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                &route.name,
                route.start_stop_id,
                route.end_stop_id,
                route.operator_id,
                &route.created_at,
                route.id,
            ],
        )?;

        if updated == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM tram_routes WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    fn row_to_tram_route(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<TramRoute> {
        Ok(TramRoute {
            id: row.get("id")?,
            name: row.get("name")?,
            start_stop_id: row.get("start_stop_id")?,
            end_stop_id: row.get("end_stop_id")?,
            operator_id: row.get("operator_id")?,
            created_at: row.get("created_at")?,
        })
    }
}
