
#[macro_export]
macro_rules! filter_eq {
    ($builder:expr, $column:expr, $value:expr) => {
        if let Some(value) = $value {
            $builder
                .push(" AND ")
                .push($column)
                .push(" = ")
                .push_bind(value);
        }
    };
}

#[macro_export]
macro_rules! filter_ilike {
    ($builder:expr, $column:expr, $value:expr) => {
        if let Some(value) = $value {
            $builder
                .push(" AND ")
                .push($column)
                .push(" ILIKE ")
                .push_bind(format!("%{}%", value));
        }
    };
}

#[macro_export]
macro_rules! filter_search {
    ($builder:expr, $search:expr, [$($column:expr),+ $(,)?]) => {
        if let Some(search) = $search {
            $builder.push(" AND (");
            let mut first = true;

            $(
                if !first {
                    $builder.push(" OR ");
                }

                $builder
                    .push($column)
                    .push(" ILIKE ")
                    .push_bind(format!("%{}%", search));

                first = false;
            )+

            $builder.push(")");
        }
    };
}

#[macro_export]
macro_rules! filter_exists {
    ($builder:expr, $table:expr, $column:expr, $where:expr, $value:expr) => {
        if let Some(value) = $value {
            $builder
                .push(" AND EXISTS (SELECT 1 FROM ")
                .push($table)
                .push(" WHERE ")
                .push($where)
                .push(" = ")
                .push_bind(value)
                .push(")");
        }
    };
}

#[macro_export]
macro_rules! distinct_on {
    ($builder:expr, $column:expr, [$($allowed:literal),+ $(,)?]) => {{
        let unique_column = if let Some(column) = $column {
            let column = match column.as_str() {
                $($allowed => $allowed,)+
                _ => panic!("Invalid DISTINCT ON column: {}", column),
            };

            $builder
                .push("DISTINCT ON (")
                .push(column)
                .push(") ");

            Some(column)
        } else {
            None
        };

        unique_column
    }};
}

#[macro_export]
macro_rules! filter_bool {
    ($builder:expr, $column:expr, $value:expr) => {
        $crate::filter_eq!($builder, $column, $value);
    };
}

#[macro_export]
macro_rules! filter_contains {
    ($builder:expr, $column:expr, $value:expr) => {
        $crate::filter_ilike!($builder, $column, $value);
    };
}
