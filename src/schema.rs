table! {
    settings (id) {
        id -> Bigint,
        setting -> Text,
        value ->Text,
        updated ->Bigint,

    }
}
table! {
    servers (id) {
        id -> Bigint,
        server_id -> Text,
        server_token ->Text,
        settings ->Text,
        created ->Bigint,

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
table! {
    items (id) {
        id -> Bigint,
        s_id -> Text,
        name -> Text,
        description -> Text,
        product -> Text,
        settings -> Text,
        category ->Bigint,
        server ->Bigint,
        created ->Bigint,

    }
}
table! {
    auth_tokens (id) {
        id -> Bigint,
        user -> Bigint,
        token -> Text,
        expiration ->Bigint,
        created ->Bigint,
    }
}
table! {
    sales (id) {
        id -> Bigint,
        s_id -> Text,
        username -> Text,
        uuid -> Text,
        first_name -> Text,
        last_name -> Text,
        country -> Text,
        payment -> Text,
        cart -> Text,
        created -> Bigint,
    }
}

table! {
    shopping_carts (id) {
        id -> Bigint,
        s_id -> Text,
        username -> Text,
        cart -> Text,
        created -> Bigint,
    }
}
table! {
    forgot_passwords (id) {
        id -> Bigint,
        user -> Bigint,
        token -> Text,
        expiration ->Bigint,
        created ->Bigint,
    }
}
