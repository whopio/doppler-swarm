# Builder
FROM rust:1-alpine AS builder

WORKDIR /app
ADD . /app
RUN cargo build --release



# Runner
FROM alpine:latest
RUN apk add --no-cache docker-cli
# COPY --from=docker:25.0-cli /usr/local/bin/docker /usr/local/bin/docker

WORKDIR /app
COPY --from=builder /app/target/release/doppler-swarm /app/doppler-swarm
ENV RUST_LOG=info

CMD ["/app/doppler-swarm", "/app/config.json"]
