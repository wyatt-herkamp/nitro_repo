FROM rust:latest AS build
COPY . /home/build
WORKDIR /home/build

LABEL  org.label-schema.name="nitro_repo" \
      org.label-schema.vendor="wyatt-herkamp" \
      org.label-schema.schema-version="1.0"


RUN apt-get update; apt-get install -y curl \
    && curl -sL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs  \
    && curl -L https://www.npmjs.com/install.sh | sh

WORKDIR /home/build/backend
RUN  cargo build --release --features multi_storage,ssl,clap,whoami
# Build Frontend
WORKDIR /home/build/frontend
RUN npm install
RUN npm run build

# The Final Image
FROM debian:bullseye-slim
RUN apt-get install libssl1.1
RUN mkdir -p /app/data/storages && mkdir -p /var/log/nitro_repo
VOLUME /app/data
WORKDIR /app
COPY --from=build /home/build/backend/target/release/nitro_repo_full nitro_repo_full
COPY --from=build /home/build/backend/target/release/nitro_utils nitro_utils
COPY --from=build /home/build/frontend/dist frontend
COPY --from=build /home/build/entrypoint.sh entrypoint.sh
RUN addgroup nitro_repo &&  adduser --system --ingroup nitro_repo --shell /bin/sh nitro_repo

ENTRYPOINT ["/bin/sh", "entrypoint.sh"]
#ENTRYPOINT ["tail", "-f", "/dev/null"]
CMD []