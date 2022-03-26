#!/bin/sh
(
  cd frontend
  npm run build
)
cd backend
if [ "$1" = "ssl" ]; then
  echo "Compiling with SSL"
  cargo build --features ssl --release
else
  cargo build --release
fi
