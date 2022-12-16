#!/bin/bash

# shellcheck disable=SC2034
NITRO_CONFIG_DIR=/var/nitro_repo/config
NITRO_REPO_STORAGE_DIR=/var/nitro_repo/storage
NITRO_REPO_LOGS_DIR=/var/nitro_repo/logs
FRONTEND_PATH="/app/frontend"
if [ "$DATABASE_TYPE" == "sqlite" ]; then
   ./nitro_utils install --skip-if-file-exists true --log-dir $NITRO_REPO_LOGS_DIR --storage-path $NITRO_REPO_STORAGE_DIR --frontend-path $FRONTEND_PATH sqlite --database-path $DATABASE_PATH
else
   ./nitro_utils install --skip-if-file-exists true --log-dir $NITRO_REPO_LOGS_DIR --storage-path $NITRO_REPO_STORAGE_DIR --frontend-path $FRONTEND_PATH $DATABASE_TYPE --database-host $DATABASE_HOST --database-user $DATABASE_USER --database-password $DATABASE_PASSWORD --database-name $DATABASE_NAME
fi
exec ./nitro_repo_full
