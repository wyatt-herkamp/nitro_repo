build-release:
    cargo build --release
# rustfmt.toml sets `imports_granularity` and `group_imports`, which are nightly-only. Stable
# rustfmt ignores them without saying so, so formatting on stable reorders imports differently to
# formatting on nightly and the two fight each other. Pinned to nightly so there is one answer.
fmt:
    cargo +nightly fmt --all
    cd site && npm run format
    cd docs && npm run format

fmt-check:
    cargo +nightly fmt --all --check

# MinIO, for the S3 storage tests. Postgres is behind a `postgres` profile because most
# developers already have one on 5432; add `--profile postgres` if you do not.
services-up:
    docker compose -f docker-compose.dev.yml up -d --wait minio
    docker compose -f docker-compose.dev.yml up --exit-code-from minio-bucket minio-bucket
services-down:
    docker compose -f docker-compose.dev.yml down

# Backends with no service running are skipped with a warning, and so are the integration tests
# when there is no database. Run `just services-up` first, or `just test-all` to make a skip a
# failure instead.
test:
    cargo test --all
test-all:
    STORAGE_TESTS_REQUIRE_ALL=1 NITRO_TESTS_REQUIRE_DB=1 cargo test --all

# The end-to-end suite on its own: real Maven and npm requests against the real router, each test
# in its own database.
test-integration:
    NITRO_TESTS_REQUIRE_DB=1 cargo test -p nitro_repo --test maven --test npm --test authorization

# Fills a running instance with a suite of artifacts, over the real protocols.
seed config="seed.example.toml":
    cargo run -p nitro_repo -- seed --config {{config}}

# The documentation site. `docs-build` is what CI runs; it fails on a broken internal link.
docs:
    cd docs && npm run dev
docs-build:
    cd docs && npm run build

lint:
    cargo clippy --all --all-targets -- -D warnings


release-dev-docker:
    docker build -t git.kingtux.dev/wherkamp/nitro_repo/nitro_repo:latest .
    docker push git.kingtux.dev/wherkamp/nitro_repo/nitro_repo:latest