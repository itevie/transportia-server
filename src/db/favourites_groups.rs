use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::db::resources::favourites_group::FavouritesGroup;

pub static SQL_FAVOURITES_GROUPS: &str = r#"
CREATE TABLE IF NOT EXISTS favourites_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    favourite_type INTEGER NOT NULL,
    owner_id INTEGER REFERENCES users(id) NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
)
"#;

#[derive(Clone)]
pub struct FavouriteGroups {
    pool: Pool<SqliteConnectionManager>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct FavouriteGroupQuery {
    pub favourite_type: Option<i64>,
    pub search: Option<String>,
    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

impl FavouriteGroups {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(
        &self,
        id: i64,
    ) -> rusqlite::Result<Option<FavouritesGroup>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT id, favourite_type, owner_id, name, created_at
            FROM favourites_groups
            WHERE id = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_favourite_group(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(
        &self,
        query: &FavouriteGroupQuery,
        owner_id: i64,
    ) -> rusqlite::Result<Vec<FavouritesGroup>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let page_length = query
            .page_length
            .unwrap_or(50)
            .clamp(1, 1000);

        let offset = query.offset.unwrap_or(0).max(0);

        let mut sql = String::from(
            r#"
            SELECT id, favourite_type, owner_id, name, created_at
            FROM favourites_groups
            WHERE owner_id = ?
            "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        params.push(Box::new(owner_id));

        if let Some(favourite_type) = query.favourite_type {
            sql.push_str(" AND favourite_type = ?");
            params.push(Box::new(favourite_type));
        }

        if let Some(search) = &query.search {
            sql.push_str(" AND name LIKE '%' || ? || '%'");
            params.push(Box::new(search.clone()));
        }

        sql.push_str(" ORDER BY name LIMIT ? OFFSET ?");
        params.push(Box::new(page_length));
        params.push(Box::new(offset));

        let mut statement = connection.prepare(&sql)?;

        statement
            .query_map(
                rusqlite::params_from_iter(
                    params.iter().map(|param| param.as_ref()),
                ),
                Self::row_to_favourite_group,
            )?
            .collect()
    }

    pub fn create(
        &self,
        favourite_type: i64,
        owner_id: i64,
        name: &str,
        created_at: &str,
    ) -> rusqlite::Result<i64> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO favourites_groups (
                favourite_type,
                owner_id,
                name,
                created_at
            )
            VALUES (?, ?, ?, ?)
            "#,
            rusqlite::params![
                favourite_type,
                owner_id,
                name,
                created_at,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(
        &self,
        group: &FavouritesGroup,
    ) -> rusqlite::Result<bool> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let affected = connection.execute(
            r#"
            UPDATE favourites_groups SET
                favourite_type = ?,
                name = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                group.favourite_type,
                &group.name,
                group.id,
            ],
        )?;

        Ok(affected > 0)
    }

    pub fn delete(
        &self,
        id: i64,
    ) -> rusqlite::Result<bool> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        // TODO: Make it so it removes this group from favourites

        let affected = connection.execute(
            "DELETE FROM favourites_groups WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(affected > 0)
    }

    fn row_to_favourite_group(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<FavouritesGroup> {
        Ok(FavouritesGroup {
            id: row.get("id")?,
            favourite_type: row.get("favourite_type")?,
            owner_id: row.get("owner_id")?,
            name: row.get("name")?,
            created_at: row.get("created_at")?,
        })
    }
}