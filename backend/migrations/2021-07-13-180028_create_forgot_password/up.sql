CREATE TABLE forgot_passwords
(
    id   BIGINT AUTO_INCREMENT PRIMARY KEY,
    user BIGINT,
    token  TEXT,
    expiration BIGINT,
    created BIGINT

)