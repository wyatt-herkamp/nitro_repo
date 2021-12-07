#!/bin/sh
(
  cd site
  npm run build
)
if [ "$1" = "ssl" ]
then
  echo "Compiling with SSL"
  cargo build --features ssl --release
else
    cargo build --release
fi
