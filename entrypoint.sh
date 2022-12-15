#!/bin/bash

NITRO_CONFIG_DIR=/etc/nitro_repo

./nitro_utils install --skip-if-file-exists true

exec ./nitro_repo_full
