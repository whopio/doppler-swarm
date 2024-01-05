FROM rust:1-slim-buster AS builder
WORKDIR /app
ADD . /app
RUN cargo build --release


FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /app/target/release/doppler-swarm /app/doppler-swarm
ENV RUST_LOG=info
CMD ["/app/doppler-swarm"]
