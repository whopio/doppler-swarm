# Builder
FROM rust:1-slim-buster AS builder
RUN apt-get update && apt-get upgrade -y && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
ADD . /app
RUN cargo build --release




# Runner
FROM debian:buster-slim
RUN apt-get update && apt-get upgrade -y && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/doppler-swarm /app/doppler-swarm
ENV RUST_LOG=info

CMD ["/app/doppler-swarm", "/app/config.json"]
