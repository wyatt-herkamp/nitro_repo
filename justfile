build-release:
    cargo build --release
fmt:
    cargo fmt --all
    cd site && npm run format

# Backing services the storage tests need (Postgres + MinIO).
services-up:
    docker compose -f docker-compose.dev.yml up -d
services-down:
    docker compose -f docker-compose.dev.yml down

# Backends with no service running are skipped with a warning. Run `just services-up` first, or
# `just test-all` to make a skipped backend a failure instead.
test:
    cargo test --all
test-all:
    STORAGE_TESTS_REQUIRE_ALL=1 cargo test --all

lint:
    cargo clippy --all --all-targets -- -D warnings


release-dev-docker:
    docker build -t git.kingtux.dev/wherkamp/nitro_repo/nitro_repo:latest .
    docker push git.kingtux.dev/wherkamp/nitro_repo/nitro_repo:latest