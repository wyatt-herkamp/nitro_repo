# Build Frontend
FROM node:22-bookworm-slim AS frontend
COPY ./site /home/frontend
WORKDIR /home/frontend
RUN npm install
RUN npm run build
RUN echo $(ls -1 /home/frontend/dist)
FROM rust:latest AS build

COPY . /home/build
WORKDIR /home/build
COPY --from=frontend /home/frontend/dist /home/build/site-dist
ENV FRONTEND_DIST=/home/build/site-dist/

# Build Backend
WORKDIR /home/build/backend
RUN  cargo build --release --features frontend

LABEL org.label-schema.name="nitro_repo" \
    org.label-schema.vendor="wyatt-herkamp" \
    org.label-schema.schema-version="2.0-BETA" \
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
ENTRYPOINT ["/bin/sh", "/app/entrypoint.sh"]
CMD []