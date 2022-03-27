table! {
    users (id) {
        id -> Bigint,
        name -> Text,
        username -> Text,
        email -> Text,
        password -> Text,
        permissions -> Text,
        created ->Bigint,

    }
}
table! {
    session_tokens (id) {
        id -> Bigint,
        user -> Bigint,
        token -> Text,
        expiration ->Bigint,
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
    forgot_passwords (id) {
        id -> Bigint,
        user -> Bigint,
        token -> Text,
        expiration ->Bigint,
        created ->Bigint,
    }
}
