table! {
    settings (id) {
        id -> Bigint,
        setting -> Text,
        value ->Text,
        updated ->Bigint,

    }
}
table! {
    users (id) {
        id -> Bigint,
        username -> Text,
        email -> Text,
        password -> Text,
        created ->Bigint,

    }
}
