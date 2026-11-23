use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

pub static ALLOWED_UNIQUE_FILTERS: LazyLock<HashMap<&'static str, Vec<(&'static str, &'static str)>>> =
    LazyLock::new(|| {
        HashMap::from([
            (
                "vehicles",
                vec![
                    ("operator_id", "v.operator_id"),
                    ("vehicle_type_id", "v.vehicle_type_id"),
                    ("livery_id", "v.livery_id"),
                ],
            ),
        ])
    });