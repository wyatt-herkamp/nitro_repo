#!/bin/bash
nitro_repo
chown -R nitro_repo:nitro_repo /app
chown -R nitro_repo:nitro_repo /var/log/nitro_repo
# shellcheck disable=SC2093
chmod +x ./nitro_utils
chmod +x ./nitro_repo_full

./nitro_utils install --log-dir /var/log/nitro_repo --storage-path /app/data/storages --frontend-path /app/frontend sqlite --database-file /app/data/database.db --skip-if-file-exists true
exec ./nitro_repo_full
