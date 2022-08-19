chown -R nitro_repo:nitro_repo /app
chown -R nitro_repo:nitro_repo /var/log/nitro_repo

cd data
# shellcheck disable=SC2093
exec runuser -u nitro_repo -- ../simple_installer --log-dir /var/log/nitro_repo  --storage-path /app/data/storages --frontend-path /app/frontend sqlite --database-file /app/data/database.db
exec runuser -u nitro_repo -- ../nitro_repo_full
