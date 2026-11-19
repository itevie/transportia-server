
use sqlx::{
    Encode, Postgres, QueryBuilder, Type,
};

type EqBinder = Box<
    dyn for<'q> Fn(&mut QueryBuilder<Postgres>) + Send + Sync
>;

pub struct SearchQuery<'a> {
    pub select: &'a str,
    pub from: &'a str,

    pub distinct: Option<(&'a str, &'a [(&'a str, &'a str)])>,

    pub ilike: Vec<(&'a str, Option<&'a str>)>,

    pub eq: Vec<(&'a str, EqBinder)>,

    pub exists: Vec<(Option<bool>, &'a str)>,

    pub search: Option<&'a str>,
    pub search_columns: Vec<&'a str>,

    pub normalized_search: Option<&'a str>,
    pub normalized_columns: Vec<&'a str>,
    pub normalized_exists: Vec<NormalizedExists<'a>>,

    pub order: &'a str,
    pub limit: i64,
    pub offset: Option<i64>,
}

pub struct NormalizedExists<'a> {
    pub from: &'a str,
    pub condition: &'a str,
    pub columns: Vec<&'a str>,
}

impl<'a> SearchQuery<'a> {
    pub fn new(select: &'a str, from: &'a str) -> Self {
        Self {
            select,
            from,
            distinct: None,
            ilike: Vec::new(),
            eq: Vec::new(),
            exists: Vec::new(),
            search: None,
            search_columns: Vec::new(),
            normalized_search: None,
            normalized_columns: Vec::new(),
            normalized_exists: Vec::new(),
            order: "",
            limit: 50,
            offset: None,
        }
    }

    pub fn distinct(
        mut self,
        column: &'a str,
        allowed: &'a [(&'a str, &'a str)],
    ) -> Self {
        self.distinct = Some((column, allowed));
        self
    }

    pub fn ilike(
        mut self,
        column: &'a str,
        value: Option<&'a str>,
    ) -> Self {
        self.ilike.push((column, value));
        self
    }

    /// Adds a typed equality filter.
    ///
    /// Accepts any owned SQLx-compatible PostgreSQL value.
    pub fn eq<T>(
        mut self,
        column: &'a str,
        value: Option<T>,
    ) -> Self
    where
        T: for<'q> Encode<'q, Postgres>
        + Type<Postgres>
        + Clone
        + Send
        + Sync
        + 'static,
    {
        if let Some(value) = value {
            self.eq.push((
                column,
                Box::new(move |builder| {
                    builder.push_bind(value.clone());
                }),
            ));
        }

        self
    }

    pub fn exists(
        mut self,
        value: Option<bool>,
        sql: &'a str,
    ) -> Self {
        self.exists.push((value, sql));
        self
    }

    /// Searches multiple columns using ILIKE.
    pub fn search(
        mut self,
        value: Option<&'a str>,
        columns: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        self.search = value;
        self.search_columns = columns.into_iter().collect();
        self
    }

    /// Searches columns using the normalized search value.
    pub fn normalized_search(
        mut self,
        value: Option<&'a str>,
        columns: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        self.normalized_search = value;
        self.normalized_columns = columns.into_iter().collect();
        self
    }

    /// Adds a correlated EXISTS search to normalized search.
    pub fn normalized_exists(
        mut self,
        from: &'a str,
        condition: &'a str,
        columns: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        self.normalized_exists.push(NormalizedExists {
            from,
            condition,
            columns: columns.into_iter().collect(),
        });
        self
    }

    pub fn order(mut self, order: &'a str) -> Self {
        self.order = order;
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = limit;
        self
    }

    pub fn offset(mut self, offset: Option<i64>) -> Self {
        self.offset = offset;
        self
    }
}

pub fn search_query(
    builder: &mut QueryBuilder<Postgres>,
    query: &SearchQuery<'_>,
) -> sqlx::Result<()> {
    // Resolve DISTINCT ON column.
    let distinct_column = if let Some((unique, allowed)) = query.distinct {
        Some(
            allowed
                .iter()
                .find(|(name, _)| *name == unique)
                .map(|(_, column)| *column)
                .ok_or_else(|| {
                    sqlx::Error::Protocol(
                        format!("Invalid DISTINCT ON column: {unique}")
                    )
                })?,
        )
    } else {
        None
    };

    builder.push("SELECT ");

    if let Some(column) = distinct_column {
        builder
            .push("DISTINCT ON (")
            .push(column)
            .push(") ");
    }

    builder
        .push(query.select)
        .push(query.from);

    // ILIKE filters.
    for (column, value) in &query.ilike {
        if let Some(value) = value {
            builder
                .push(" AND ")
                .push(*column)
                .push(" ILIKE ")
                .push_bind(format!("%{value}%"));
        }
    }

    // Equality filters.
    for (column, bind) in &query.eq {
        builder
            .push(" AND ")
            .push(*column)
            .push(" = ");

        bind(builder);
    }

    // EXISTS filters.
    for (value, sql) in &query.exists {
        if let Some(value) = value {
            builder
                .push(if *value {
                    " AND EXISTS ("
                } else {
                    " AND NOT EXISTS ("
                })
                .push(*sql)
                .push(")");
        }
    }

    // Regular multi-column search.
    if let Some(search) = query.search {
        push_search_columns(
            builder,
            search,
            &query.search_columns,
        );
    }

    // Normalized multi-column search.
    if let Some(search) = query.normalized_search {
        push_search_columns(
            builder,
            search,
            &query.normalized_columns,
        );

        // Search related tables through correlated EXISTS.
        for exists in &query.normalized_exists {
            if exists.columns.is_empty() {
                continue;
            }

            builder
                .push(" AND EXISTS (SELECT 1 ")
                .push(exists.from)
                .push(" WHERE ")
                .push(exists.condition)
                .push(" AND (");

            push_search_conditions(
                builder,
                search,
                &exists.columns,
            );

            builder.push("))");
        }
    }

    // ORDER BY.
    if distinct_column.is_some() || !query.order.is_empty() {
        builder.push(" ORDER BY ");

        if let Some(column) = distinct_column {
            builder.push(column);

            if !query.order.is_empty() {
                builder.push(", ");
            }
        }

        if !query.order.is_empty() {
            builder.push(query.order);
        }
    }

    builder
        .push(" LIMIT ")
        .push_bind(query.limit);

    if let Some(offset) = query.offset {
        builder
            .push(" OFFSET ")
            .push_bind(offset);
    }

    Ok(())
}

/// Adds a parenthesized OR search across multiple columns.
fn push_search_columns(
    builder: &mut QueryBuilder<Postgres>,
    search: &str,
    columns: &[&str],
) {
    if columns.is_empty() {
        return;
    }

    builder.push(" AND (");

    push_search_conditions(builder, search, columns);

    builder.push(")");
}

/// Adds column ILIKE conditions without surrounding parentheses.
fn push_search_conditions(
    builder: &mut QueryBuilder<Postgres>,
    search: &str,
    columns: &[&str],
) {
    for (index, column) in columns.iter().enumerate() {
        if index > 0 {
            builder.push(" OR ");
        }

        builder
            .push(*column)
            .push(" ILIKE ")
            .push_bind(format!("%{search}%"));
    }
}
