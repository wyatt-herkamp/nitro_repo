CREATE TABLE IF NOT EXISTS nr_test_environment (
    id              SERIAL PRIMARY KEY,
    function_path   TEXT                NOT NULL,
        constraint function_path_unique UNIQUE (function_path),
    run_successfully BOOLEAN,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);