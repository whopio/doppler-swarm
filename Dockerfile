# # Builder
# FROM rust:1-slim-bookworm AS builder
# RUN apt-get update && apt-get upgrade -y && apt-get clean && rm -rf /var/lib/apt/lists/*

# WORKDIR /app
# ADD . /app
# RUN cargo build --release




# # Runner
# FROM debian:bookworm-slim
# RUN apt-get update && apt-get upgrade -y && apt-get clean && rm -rf /var/lib/apt/lists/*

# WORKDIR /app
# COPY --from=builder /app/target/release/doppler-swarm /app/doppler-swarm
# ENV RUST_LOG=info

# CMD ["/app/doppler-swarm", "/app/config.json"]


# Builder
FROM rust:1-alpine AS builder
RUN apt-get update && apt-get upgrade -y && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
ADD . /app
RUN cargo build --release




# Runner
FROM alpine:latest
# RUN apt-get update && apt-get upgrade -y && apt-get clean && rm -rf /var/lib/apt/lists/*
RUN apk add --no-cache docker-cli
# COPY --from=docker:25.0-cli /usr/local/bin/docker /usr/local/bin/docker

WORKDIR /app
COPY --from=builder /app/target/release/doppler-swarm /app/doppler-swarm
ENV RUST_LOG=info

CMD ["/app/doppler-swarm", "/app/config.json"]
