FROM rust:slim-bullseye as builder
WORKDIR /app
RUN USER=root cargo new backend-dogfight-moonshine-24-q1
WORKDIR /app/backend-dogfight-moonshine-24-q1

RUN USER=root cargo new db
RUN USER=root cargo new api
RUN USER=root cargo new load-balancer

COPY Cargo.toml Cargo.lock ./
COPY db/Cargo.toml db/Cargo.toml
COPY api/Cargo.toml api/Cargo.toml

RUN cargo build --release
RUN rm -rf api db

COPY api api
COPY db db

RUN cargo build --release --bin moonshine-api


FROM debian:bullseye-slim
COPY --from=builder /app/backend-dogfight-moonshine-24-q1/target/release/moonshine-api /usr/local/bin/

CMD ["moonshine-api"]