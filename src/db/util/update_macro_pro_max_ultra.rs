
#[macro_export]
macro_rules! define_update {
    (
        $update:ident, $normalised:ident => $model:ty {
            $(
                $field:ident : $kind:ident $type:ty => $validation:expr
            ),* $(,)?
        }
    ) => {
        $crate::define_update! {
            @generate
            $update, $normalised => $model {
                $(
                    $field : $kind $type => $validation
                ),*
            }
        }
    };

    (@generate $update:ident, $normalised:ident => $model:ty {
        $( $field:ident : $kind:ident $type:ty => $validation:expr ),*
    }) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
        pub struct $update {
            $(
                $crate::define_update!(@field $field, $kind, $type);
            )*
        }
    };
}
