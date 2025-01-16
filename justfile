build-release:
    cargo build --release
fmt:
    cargo fmt --all
    cd site && npm run format


release-dev-docker:
    docker build -t git.kingtux.dev/wherkamp/nitro_repo/nitro_repo:latest .
    docker push git.kingtux.dev/wherkamp/nitro_repo/nitro_repo:latest