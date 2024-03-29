FROM rust:latest AS build
COPY . /home/build
WORKDIR /home/build

RUN apt-get update; apt-get install -y curl \
    && curl -sL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs  \
    && curl -L https://www.npmjs.com/install.sh | sh

WORKDIR /home/build/backend
RUN  cargo build --release --features multi_storage,ssl,clap
# Build Frontend
WORKDIR /home/build/frontend
RUN npm install
RUN npm run build


LABEL org.label-schema.name="nitro_repo" \
      org.label-schema.vendor="wyatt-herkamp" \
      org.label-schema.schema-version="1.0" \
      org.label-schema.url="https://nitro-repo.kingtux.dev/" \
      org.label-schema.description="An open source artifact manager. Written in Rust back end and an Vue front end to create a fast and modern experience"

# The Final Image
FROM debian:bullseye-slim

RUN apt-get install libssl1.1

RUN mkdir -p /etc/nitro_repo
RUN mkdir -p /var/nitro_repo && mkdir -p /var/log/nitro_repo


COPY --from=build /home/build/backend/target/release/nitro_repo_full nitro_repo_full
COPY --from=build /home/build/backend/target/release/nitro_utils nitro_utils
COPY --from=build /home/build/frontend/dist frontend
COPY --from=build /home/build/entrypoint.sh entrypoint.sh


ENTRYPOINT ["/bin/sh", "entrypoint.sh"]
CMD []