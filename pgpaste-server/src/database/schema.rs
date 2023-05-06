// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "visibility"))]
    pub struct Visibility;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Visibility;

    pastes (id) {
        id -> Int4,
        public_key_id -> Int4,
        slug -> Varchar,
        visibility -> Visibility,
        content -> Bytea,
    }
}

diesel::table! {
    public_keys (id) {
        id -> Int4,
        fingerprint -> Bytea,
        key -> Bytea,
    }
}

diesel::joinable!(pastes -> public_keys (public_key_id));

diesel::allow_tables_to_appear_in_same_query!(
    pastes,
    public_keys,
);
