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
table! {
     storages(id) {
        id -> Bigint,
        public_name -> Text,
        name -> Text,
        created ->Bigint,
    }
}
table! {
     repositories(id) {
        id -> Bigint,
        name -> Text,
        repo_type -> Text,
        storage -> Bigint,
        settings -> Text,
        deploy_settings -> Text,
        security -> Text,
        created ->Bigint,
    }
}
