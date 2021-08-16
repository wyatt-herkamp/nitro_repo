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
table! {
    categories (id) {
        id -> Bigint,
        s_id -> Text,
        name -> Text,
        settings -> Text,
        server -> Bigint,
        created ->Bigint,

    }
}