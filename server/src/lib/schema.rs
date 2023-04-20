// @generated automatically by Diesel CLI.

diesel::table! {
    groups (id) {
        id -> Int4,
        title -> Text,
        colour -> Nullable<Text>,
        vocab -> Bool,
        user_id -> Int4,
    }
}

diesel::table! {
    kanji (id) {
        id -> Int4,
        symbol -> Text,
        meaning -> Text,
        onyomi -> Array<Nullable<Text>>,
        kunyomi -> Array<Nullable<Text>>,
        description -> Nullable<Text>,
        vocab_refs -> Array<Nullable<Text>>,
        user_id -> Int4,
        group_id -> Nullable<Int4>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Text,
        hash -> Bytea,
        salt -> Bytea,
    }
}

diesel::table! {
    vocab (id) {
        id -> Int4,
        phrase -> Text,
        meaning -> Text,
        reading -> Array<Nullable<Text>>,
        description -> Nullable<Text>,
        kanji_refs -> Array<Nullable<Text>>,
        user_id -> Int4,
        group_id -> Nullable<Int4>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    groups,
    kanji,
    users,
    vocab,
);
