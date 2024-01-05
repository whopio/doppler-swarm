# FROM --platform=linux/arm64 rust:1-slim-buster AS builder-arm64
FROM rust:1-slim-buster AS builder

WORKDIR /app

ADD . /app
RUN cargo build --release --target aarch64-unknown-linux-gnu


# FROM --platform=arm64 debian:buster-slim
FROM debian:buster-slim

WORKDIR /app

COPY --from=builder /app/target/release/doppler-swarm /app/doppler-swarm

ENV RUST_LOG=info

CMD ["/app/doppler-swarm"]
