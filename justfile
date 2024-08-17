build-release:
    cargo build --release
fmt:
    cargo fmt --all
    cd site && npm run format