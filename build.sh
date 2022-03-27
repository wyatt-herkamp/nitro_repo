#!/bin/sh
(
  cd frontend
  npm run build
)
if [ "$1" = "ssl" ]; then
  echo "Compiling with SSL"
  cargo build --features ssl --release --manifest-path backend/Cargo.toml
else
  cargo build --release --manifest-path backend/Cargo.toml
fi
