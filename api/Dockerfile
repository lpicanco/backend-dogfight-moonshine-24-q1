FROM rust:slim-bullseye as builder
WORKDIR /app
RUN USER=root cargo new backend-dogfight-moonshine-24-q1-api
WORKDIR /app/backend-dogfight-moonshine-24-q1-api
COPY Cargo.toml ./
RUN cargo build --release
RUN rm -rf src

COPY src src
COPY ../db ../db

RUN touch src/main.rs
RUN cargo build --release


FROM debian:bullseye-slim
COPY --from=builder /app/backend-dogfight-moonshine-24-q1-api/target/release/backend-dogfight-moonshine-24-q1-api /usr/local/bin/

CMD ["backend-dogfight-moonshine-24-q1-api"]