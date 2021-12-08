#!/bin/sh
(
  cd site
  npx prettier --write .
)
cargo fmt
