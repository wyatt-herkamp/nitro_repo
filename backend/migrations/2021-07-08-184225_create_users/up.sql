CREATE TABLE users
(
    id          BIGINT AUTO_INCREMENT PRIMARY KEY,
    name        TEXT,
    username    TEXT,
    email       TEXT,
    password    TEXT,
    permissions TEXT,
    created     BIGINT
)