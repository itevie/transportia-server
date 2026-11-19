
#[macro_export]
macro_rules! search_query {
    (
        $builder:expr,

        select: $select:literal,

        distinct: ($unique:expr, [$(($allowed:literal, $sql_column:literal)),+ $(,)?]),

        from: $from:expr,

        filters: {
            ilike: [
                $(($ilike_column:literal, $ilike_value:expr)),* $(,)?
            ],

            eq: [
                $(($eq_column:literal, $eq_value:expr)),* $(,)?
            ],

            exists: [
                $(($exists_value:expr, $exists_sql:expr)),* $(,)?
            ],

            search: {
                value: $search_value:expr,
                columns: [$($search_column:literal),+ $(,)?]
            },

            normalized_search: {
                value: $normalized_value:expr,
                columns: [$($normalized_column:literal),* $(,)?],
                exists: [
                    $(
                        (
                            $related_from:literal,
                            $related_condition:literal,
                            [$($related_column:literal),+ $(,)?]
                        )
                    ),* $(,)?
                ]
            },
        },

        order: $order:literal,
        limit: $limit:expr,
        offset: $offset:expr $(,)?
    ) => {{
        let builder = &mut $builder;

        // SELECT
        builder.push("SELECT ");

        // DISTINCT ON
        let unique_column: Option<&'static str> = match $unique {
            Some(column) => {
                let column = match column {
                    $(
                        $allowed => $sql_column,
                    )+
                    _ => return Err(sqlx::Error::Protocol(
                        format!("Invalid DISTINCT ON column: {}", column)
                    )),
                };

                builder
                    .push("DISTINCT ON (")
                    .push(column)
                    .push(") ");

                Some(column)
            }
            None => None,
        };

        builder
            .push($select)
            .push($from);

        // ILIKE filters
        $(
            if let Some(value) = $ilike_value {
                builder
                    .push(" AND ")
                    .push($ilike_column)
                    .push(" ILIKE ")
                    .push_bind(format!("%{}%", value));
            }
        )*

        // Equality filters
        $(
            if let Some(value) = $eq_value {
                builder
                    .push(" AND ")
                    .push($eq_column)
                    .push(" = ")
                    .push_bind(value);
            }
        )*

        // EXISTS filters
        $(
            if let Some(value) = $exists_value {
                builder
                    .push(if value {
                        " AND EXISTS ("
                    } else {
                        " AND NOT EXISTS ("
                    })
                    .push($exists_sql)
                    .push(")");
            }
        )*

        // Multi-column ILIKE search
        if let Some(search) = $search_value {
            let search = format!("%{}%", search);

            builder.push(" AND (");

            let mut count = 0;

            $(
                if count > 0 {
                    builder.push(" OR ");
                }

                builder
                    .push($search_column)
                    .push(" ILIKE ")
                    .push_bind(search.clone());

                count += 1;
            )+

            builder.push(")");
        }

        // Normalized search, including related tables
        if let Some(search) = $normalized_value {
            let search = format!(
                "%{}%",
                search.to_lowercase().replace(' ', "")
            );

            builder.push(" AND (");

            let mut count = 0;

            // Direct columns
            $(
                if count > 0 {
                    builder.push(" OR ");
                }

                builder
                    .push("REPLACE(LOWER(COALESCE(")
                    .push($normalized_column)
                    .push(", '')), ' ', '') LIKE ")
                    .push_bind(search.clone());

                count += 1;
            )*

            // Related table EXISTS searches
            $(
                if count > 0 {
                    builder.push(" OR ");
                }

                builder
                    .push("EXISTS (")
                    .push("SELECT 1 ")
                    .push($related_from)
                    .push(" WHERE ")
                    .push($related_condition)
                    .push(" AND (");

                let mut related_count = 0;

                $(
                    if related_count > 0 {
                        builder.push(" OR ");
                    }

                    builder
                        .push("REPLACE(LOWER(COALESCE(")
                        .push($related_column)
                        .push(", '')), ' ', '') LIKE ")
                        .push_bind(search.clone());

                    related_count += 1;
                )+

                builder.push("))");

                count += 1;
            )*

            builder.push(")");
        }

        // ORDER BY
        builder.push(" ORDER BY ");

        if let Some(column) = unique_column {
            builder
                .push(column)
                .push(", ");
        }

        builder.push($order);

        // LIMIT and OFFSET
        builder
            .push(" LIMIT ")
            .push_bind($limit);

        if let Some(offset) = $offset {
            builder
                .push(" OFFSET ")
                .push_bind(offset);
        }
    }};
}
