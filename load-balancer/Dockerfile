FROM rust:slim-bullseye as builder
WORKDIR /app
RUN USER=root cargo new moonshine-lb
WORKDIR /app/moonshine-lb
RUN apt update && apt install libfindbin-libs-perl cmake make g++ openssl libssl-dev pkg-config -y
COPY Cargo.toml ./
RUN cargo build --release
RUN rm -rf src

COPY src src

RUN touch src/main.rs
RUN cargo build --release


FROM debian:bullseye-slim
COPY --from=builder /app/moonshine-lb/target/release/moonshine-lb /usr/local/bin/

CMD ["moonshine-lb"]