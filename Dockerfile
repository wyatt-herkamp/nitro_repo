FROM rust:latest AS build
COPY . /home/build
WORKDIR /home/build

RUN apt-get update; apt-get install -y curl \
    && curl -sL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs  \
    && curl -L https://www.npmjs.com/install.sh | sh
RUN apt-get install -y libssl-dev pkg-config

# Build Frontend
WORKDIR /home/build/site
RUN npm install
RUN npm run build

WORKDIR /home/build/backend
RUN  cargo build --release --features frontend

LABEL org.label-schema.name="nitro_repo" \
    org.label-schema.vendor="wyatt-herkamp" \
    org.label-schema.schema-version="1.0" \
    org.label-schema.url="https://nitro-repo.kingtux.dev/" \
    org.label-schema.description="An open source artifact manager. Written in Rust back end and an Vue front end to create a fast and modern experience"

# The Final Image
FROM debian:bookworm-slim

RUN apt-get update -y && apt-get -y install libssl-dev openssl

RUN mkdir -p /opt/nitro-repo
RUN mkdir -p /app
COPY --from=build /home/build/target/release/nitro_repo /app/nitro-repo
COPY --from=build /home/build/entrypoint.sh /app/entrypoint.sh
WORKDIR /opt/nitro-repo
ENTRYPOINT ["/bin/sh", "entrypoint.sh"]
CMD []