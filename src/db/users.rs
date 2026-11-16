use r2d2::Pool;
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use crate::db::resources::user::User;

pub static SQL_USER: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    display_name TEXT,
    bio TEXT,
    password_hash TEXT NOT NULL,
    trusted_user INTEGER NOT NULL DEFAULT 0,
    admin INTEGER NOT NULL DEFAULT 0
)
"#;

#[derive(Clone)]
pub struct Users {
    pool: Pool<SqliteConnectionManager>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct UserQuery {
    pub search: Option<String>,
    pub page_length: Option<i64>,
    pub offset: Option<i64>,
}

impl Users {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: i64, include_sensitive: bool) -> rusqlite::Result<Option<User>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            &format!(
                r#"
                SELECT
                    id,
                    {},
                    username,
                    display_name,
                    bio,
                    trusted_user,
                    admin,
                    '' AS password_hash
                FROM users
                WHERE id = ?
                "#,
                if include_sensitive { "email" } else { "'' as email" }
            )
        )?;

        let mut rows = statement.query(rusqlite::params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_user(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_username(&self, username: &str) -> rusqlite::Result<Option<User>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            r#"
            SELECT
                id,
                email,
                username,
                display_name,
                bio,
                trusted_user,
                admin,
                '' AS password_hash
            FROM users
            WHERE username = ?
            "#,
        )?;

        let mut rows = statement.query(rusqlite::params![username.to_lowercase()])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_user(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_email(&self, email: &str, include_sensitive: bool) -> rusqlite::Result<Option<User>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        let mut statement = connection.prepare(
            &format!(
                r#"
                SELECT
                    id,
                    {},
                    username,
                    display_name,
                    bio,
                    trusted_user,
                    admin,
                    {}
                FROM users
                WHERE email = ?
                "#,
                if include_sensitive { "email" } else { "'' AS email" },
                if include_sensitive { "password_hash" } else { "'' AS password_hash" }
            )
        )?;

        let mut rows = statement.query(rusqlite::params![email.to_lowercase()])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_user(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &UserQuery) -> rusqlite::Result<Vec<User>> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;
        
        let page_length = match query.page_length {
            Some(x) => if x > 1000 { 1000 } else { x },
            None => 50,
        };

        let mut sql = String::from(
            r#"
        SELECT
            id,
            '' AS email,
            username,
            display_name,
            bio,
            trusted_user,
            admin,
            '' AS password_hash
        FROM users
        WHERE 1 = 1
        "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(value) = &query.search {
            sql.push_str(
                r#"
            AND (
                username LIKE '%' || ? || '%' OR
                display_name LIKE '%' || ? || '%'
            )
            "#,
            );

            params.push(Box::new(value.clone()));
            params.push(Box::new(value.clone()));
        }

        sql.push_str(" ORDER BY username");

        sql.push_str(" LIMIT ?");
        params.push(Box::new(page_length));

        if let Some(offset) = query.offset {
            sql.push_str(" OFFSET ?");
            params.push(Box::new(offset));
        }

        let mut statement = connection.prepare(&sql)?;

        statement
            .query_map(
                rusqlite::params_from_iter(
                    params.iter().map(|param| param.as_ref())
                ),
                Self::row_to_user,
            )?
            .collect()
    }

    pub fn create(&self, email: &str, username: &str, password: &str) -> rusqlite::Result<i64> {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|err| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(err))
            })?;

        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            INSERT INTO users (
                email,
                username,
                password_hash
            )
            VALUES (?, ?, ?)
            "#,
            rusqlite::params![
                &email,
                &username,
                &password_hash,
            ],
        )?;

        Ok(connection.last_insert_rowid())
    }

    pub fn update(&self, user: &User) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            r#"
            UPDATE users SET
                email = ?,
                username = ?,
                display_name = ?,
                bio = ?,
                password_hash = ?,
                trusted_user = ?,
                admin = ?
            WHERE id = ?
            "#,
            rusqlite::params![
                &user.email,
                &user.username,
                &user.display_name,
                &user.bio,
                &user.password_hash,
                user.id,
                &user.trusted_user,
                &user.admin,
            ],
        )?;

        Ok(())
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<()> {
        let connection = self.pool.get().map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;

        connection.execute(
            "DELETE FROM users WHERE id = ?",
            rusqlite::params![id],
        )?;

        Ok(())
    }

    pub fn check_password(&self, email: &str, password: &str) -> rusqlite::Result<Option<User>> {
        let Some(mut user) = self.get_by_email(email, true)? else {
            return Ok(None);
        };

        let valid = bcrypt::verify(password, &user.password_hash)
            .map_err(|err| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(err))
            })?;

        if !valid {
            Ok(None)
        } else {
            user.password_hash = "".to_string();
            Ok(Some(user))
        }
    }

    fn row_to_user(row: &rusqlite::Row) -> rusqlite::Result<User> {
        Ok(User {
            id: row.get("id")?,
            email: row.get("email")?,
            username: row.get("username")?,
            display_name: row.get("display_name")?,
            bio: row.get("bio")?,
            password_hash: row.get("password_hash")?,
            trusted_user: row.get("trusted_user")?,
            admin: row.get("admin")?,
        })
    }
}