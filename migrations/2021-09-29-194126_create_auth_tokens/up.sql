CREATE TABLE auth_tokens
(
    id   BIGINT AUTO_INCREMENT PRIMARY KEY,
    user BIGINT,
    token  TEXT,
    expiration BIGINT,
    created BIGINT

)