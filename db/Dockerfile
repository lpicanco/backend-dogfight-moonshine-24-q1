FROM rust:slim-bullseye as builder
WORKDIR /app
RUN USER=root cargo new moonshine-db
WORKDIR /app/moonshine-db
#COPY Cargo.toml Cargo.lock ./
COPY Cargo.toml ./
RUN cargo build --release
RUN rm -rf src

COPY src src

RUN touch src/main.rs
RUN cargo build --release


FROM debian:bullseye-slim
COPY --from=builder /app/moonshine-db/target/release/moonshine-db /usr/local/bin/

CMD ["moonshine-db"]