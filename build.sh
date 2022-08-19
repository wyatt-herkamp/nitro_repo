#!/bin/sh
(
  cd frontend
  nvm install
  npm run full-build
)
if [ "$1" = "ssl" ]; then
  echo "Compiling with SSL"

  cargo build --features ssl --release --manifest-path backend/Cargo.toml
else
  cargo build --release --manifest-path backend/Cargo.toml
fi
mkdir -p build/frontend
cp -R frontend/dist/* build/frontend
cp backend/target/release/nitro_repo_full build
cp -R other build
cp -R LICENSE build
(
  cd build
  tar -czvf nitro_repo.tar.gz  *
)
cp build/nitro_repo.tar.gz .
rm -R build